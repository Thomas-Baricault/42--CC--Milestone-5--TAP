use clap::Parser;
use std::str::FromStr;

#[derive(Parser)]
#[command(about = "A Multi-User Dungeon server which use the TAP protocol")]
struct Args {
    world: String,

    /// The server binding ip address
    #[arg(long, short)]
    ip: Option<String>,

    /// The server binding port
    #[arg(long, short)]
    port: Option<String>,
}

struct Cli {
    server: tap::network::Server,
    game: tap::utils::Shared<tap::game::GameState>,
}

impl Cli {
    pub async fn start() -> Result<(), std::io::Error> {
        let args = Args::parse();
        let game = match tap::game::GameState::new(&args.world) {
            Ok(v) => v,
            Err(e) => return Err(std::io::Error::new(
                e.kind(),
                format!("failed to load world data: {e}"),
            )),
        };
        let ip = match args.ip {
            Some(v) => v,
            None => "127.0.0.1".to_string(),
        };
        let port = match args.port {
            Some(v) => v,
            None => "7373".to_string(),
        };
        let mut cli = Cli {
            server: tap::network::Server::new(&format!("{ip}:{port}")),
            game: tap::utils::Shared::new(game),
        };
        cli.server.bind().await?;
        tap::cli::Logger::info(&format!("Server listening at \x1b[36m{}\x1b[0m", cli.server.addr)).await;
        cli.run().await
    }

    async fn run(&mut self) -> Result<(), std::io::Error> {
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    tap::cli::Logger::error("Interrupted").await;
                    self.server.close();
                    break;
                }
                res = self.server.accept() => {
                    match res {
                        Ok(client) => {
                            let game = self.game.clone();
                            tokio::spawn(async move {
                                Self::run_client(client, game).await;
                            });
                        }
                        Err(e) => return Err(e),
                    };
                }
            };
        }
        tap::cli::Logger::info("Server disconnected").await;
        Ok(())
    }

    async fn run_client(mut client: tap::network::Client, game: tap::utils::Shared<tap::game::GameState>) {
        let mut flood = tap::utils::FloodDetector::new(20, 1);
        let mut username = String::new();
        tap::cli::Logger::info(&format!(
            "New client connected (\x1b[36m{}\x1b[0m)",
            client.addr,
        )).await;
        tap::cli::Logger::to(
            &client,
            &username,
            &tap::messages::Message::Response(tap::messages::Response {
                payload: tap::messages::Payload::new(&[
                    tap::messages::PayloadKind::String("hello".to_string()),
                    tap::messages::PayloadKind::KeyValue {
                        key: "proto".to_string(),
                        value: "1".to_string(),
                    }
                ])
            }),
        ).await;
        loop {
            match client.reader.read().await {
                Ok(Some(message)) => {
                    tap::cli::Logger::from(&client, &username, &message).await;
                    if flood.record() {
                        tap::cli::Logger::warning(&format!(
                            "\x1b[0;35m{}\x1b[0;0m (\x1b[0;36m{}\x1b[0;0m): command flooding",
                            if username.is_empty() { "?" } else { &username },
                            client.addr,
                        )).await;
                    }
                    match tap::messages::Message::from_str(&message) {
                        Ok(tap::messages::Message::Command(command)) => {
                            if command.kind.requires_auth() && !matches!(client.state, tap::network::ClientState::Authenticated) {
                                tap::cli::Logger::to(
                                    &client,
                                    &username,
                                    &tap::messages::Message::Error(tap::messages::Error::NotAuthenticated),
                                ).await;
                            } else if matches!(command.kind, tap::messages::CommandKind::Connect) && matches!(client.state, tap::network::ClientState::Authenticated) {
                                tap::cli::Logger::to(
                                    &client,
                                    &username,
                                    &tap::messages::Message::Error(tap::messages::Error::AlreadyAuthenticated),
                                ).await;
                            } else {
                                let mut game = game.lock().await;
                                let message = Self::process_command(&mut client, &mut username, &mut game, &command).await;
                                tap::cli::Logger::to(&client, &username, &message).await;
                                if let tap::messages::Message::Response(response) = message && (
                                    client.is_open() &&
                                    response.payload.args.len() == 1 &&
                                    if let tap::messages::PayloadKind::String(s) = &response.payload.args[0] {
                                        s == "bye"
                                    } else {
                                        false
                                    }
                                ) {
                                    client.close();
                                }
                            }
                        }
                        _ => tap::cli::Logger::to(
                            &client,
                            &username,
                            &tap::messages::Message::Error(tap::messages::Error::NotACommand),
                        ).await,
                    };
                }
                Ok(None) => break,
                Err(e) => {
                    tap::cli::Logger::error(&format!(
                        "client \x1b[36m{}\x1b[0m: {}",
                        client.addr,
                        e,
                    )).await;
                    if client.is_open() {
                        tap::cli::Logger::to(
                            &client,
                            &username,
                            &tap::messages::Message::Error(tap::messages::Error::NotACommand),
                        ).await;
                    } else {
                        break;
                    }
                }
            };
            if !client.is_open() {
                break;
            }
        }
        if !username.is_empty() {
            let mut game = game.lock().await;
            tap::game::Group::leave(&mut game, &username).await;
            tap::game::RoomState::leave(&mut game, &username).await;
            game.players.remove(&username);
            tap::cli::Logger::player_count(&game).await;
        }
        tap::cli::Logger::info(&format!(
            "Client \x1b[36m{}\x1b[0m disconnected",
            client.addr,
        )).await;
    }

    async fn process_command(client: &mut tap::network::Client, username: &mut String, game: &mut tap::game::GameState, command: &tap::messages::Command) -> tap::messages::Message {
        match command.kind {
            tap::messages::CommandKind::AbandonQuest => {
                let mut quest = String::new();
                if command.payload.extract(&mut [
                    tap::messages::PayloadExtractor::String(&mut quest),
                ]).is_err() {
                    tap::messages::Message::Error(tap::messages::Error::InvalidArguments)
                } else {
                    tap::game::Player::abandon_quest(game, username, &quest)
                }
            }
            tap::messages::CommandKind::Attack => {
                let mut npc = String::new();
                if command.payload.extract(&mut [
                    tap::messages::PayloadExtractor::String(&mut npc),
                ]).is_err() {
                    tap::messages::Message::Error(tap::messages::Error::InvalidArguments)
                } else {
                    tap::game::Player::attack(game, username, &npc).await
                }
            }
            tap::messages::CommandKind::Chat => {
                let mut scope = tap::messages::EventScope::Global;
                let mut message = String::new();
                if command.payload.extract(&mut [
                    tap::messages::PayloadExtractor::Keyword(&mut scope),
                    tap::messages::PayloadExtractor::String(&mut message),
                ]).is_err() || !matches!(scope, tap::messages::EventScope::Global | tap::messages::EventScope::Group | tap::messages::EventScope::Room) {
                    tap::messages::Message::Error(tap::messages::Error::InvalidArguments)
                } else {
                    tap::game::Player::chat(game, username, &scope, &message).await
                }
            }
            tap::messages::CommandKind::Connect => {
                username.clear();
                if command.payload.extract(&mut [
                    tap::messages::PayloadExtractor::String(username),
                ]).is_err() {
                    tap::messages::Message::Error(tap::messages::Error::InvalidArguments)
                } else {
                    tap::game::Player::connect(game, client, username).await
                }
            }
            tap::messages::CommandKind::Consume => {
                let mut item = String::new();
                if command.payload.extract(&mut [
                    tap::messages::PayloadExtractor::String(&mut item),
                ]).is_err() {
                    tap::messages::Message::Error(tap::messages::Error::InvalidArguments)
                } else {
                    tap::game::Player::consume(game, username, &item).await
                }
            }
            tap::messages::CommandKind::Describe => {
                let mut id = String::new();
                if command.payload.extract(&mut [
                    tap::messages::PayloadExtractor::String(&mut id),
                ]).is_err() {
                    tap::messages::Message::Error(tap::messages::Error::InvalidArguments)
                } else {
                    game.describe(&id)
                }
            }
            tap::messages::CommandKind::Drop => {
                let mut item = String::new();
                if command.payload.extract(&mut [
                    tap::messages::PayloadExtractor::String(&mut item),
                ]).is_err() {
                    tap::messages::Message::Error(tap::messages::Error::InvalidArguments)
                } else {
                    tap::game::Player::drop(game, username, &item).await
                }
            }
            tap::messages::CommandKind::Equip => {
                let mut item = String::new();
                if command.payload.extract(&mut [
                    tap::messages::PayloadExtractor::String(&mut item),
                ]).is_err() {
                    tap::messages::Message::Error(tap::messages::Error::InvalidArguments)
                } else {
                    tap::game::Player::equip(game, username, &item).await
                }
            }
            tap::messages::CommandKind::GroupDescribe => {
                if command.payload.is_empty() {
                    tap::game::Player::describe_group(game, username)
                } else {
                    tap::messages::Message::Error(tap::messages::Error::InvalidArguments)
                }
            }
            tap::messages::CommandKind::GroupCreate => {
                let mut name = String::new();
                if command.payload.extract(&mut [
                    tap::messages::PayloadExtractor::String(&mut name),
                ]).is_err() {
                    tap::messages::Message::Error(tap::messages::Error::InvalidArguments)
                } else {
                    tap::game::Group::create(game, username, &name)
                }
            }
            tap::messages::CommandKind::GroupInvite => {
                let mut invited = String::new();
                if command.payload.extract(&mut [
                    tap::messages::PayloadExtractor::String(&mut invited),
                ]).is_err() {
                    tap::messages::Message::Error(tap::messages::Error::InvalidArguments)
                } else {
                    tap::game::Group::invite(game, username, &invited).await
                }
            }
            tap::messages::CommandKind::GroupJoin => {
                let mut group = String::new();
                if command.payload.extract(&mut [
                    tap::messages::PayloadExtractor::String(&mut group),
                ]).is_err() {
                    tap::messages::Message::Error(tap::messages::Error::InvalidArguments)
                } else {
                    tap::game::Group::join(game, username, &group).await
                }
            }
            tap::messages::CommandKind::GroupLeave => {
                if command.payload.is_empty() {
                    tap::game::Group::leave(game, username).await
                } else {
                    tap::messages::Message::Error(tap::messages::Error::InvalidArguments)
                }
            }
            tap::messages::CommandKind::Inventory => {
                if command.payload.is_empty() {
                    tap::game::Player::inventory(game, username)
                } else {
                    tap::messages::Message::Error(tap::messages::Error::InvalidArguments)
                }
            }
            tap::messages::CommandKind::Look => {
                if command.payload.is_empty() {
                    tap::game::Player::look(game, username)
                } else {
                    tap::messages::Message::Error(tap::messages::Error::InvalidArguments)
                }
            }
            tap::messages::CommandKind::Move => {
                let mut direction = tap::game::Direction::East;
                if command.payload.extract(&mut [
                    tap::messages::PayloadExtractor::Keyword(&mut direction),
                ]).is_err() {
                    tap::messages::Message::Error(tap::messages::Error::InvalidArguments)
                } else {
                    tap::game::Player::move_to(game, username, &direction).await
                }
            }
            tap::messages::CommandKind::Quest => {
                let mut npc = String::new();
                if command.payload.extract(&mut [
                    tap::messages::PayloadExtractor::String(&mut npc),
                ]).is_err() {
                    tap::messages::Message::Error(tap::messages::Error::InvalidArguments)
                } else {
                    tap::game::Npc::quest(game, username, &npc).await
                }
            }
            tap::messages::CommandKind::Quests => {
                if command.payload.is_empty() {
                    tap::game::Player::quests(game, username)
                } else {
                    tap::messages::Message::Error(tap::messages::Error::InvalidArguments)
                }
            }
            tap::messages::CommandKind::Quit => {
                if command.payload.is_empty() {
                    tap::messages::Message::Response(tap::messages::Response {
                        payload: tap::messages::Payload::new(&[
                            tap::messages::PayloadKind::String("bye".to_string()),
                        ]),
                    })
                } else {
                    tap::messages::Message::Error(tap::messages::Error::InvalidArguments)
                }
            }
            tap::messages::CommandKind::Status => {
                if command.payload.is_empty() {
                    tap::game::Player::status(game, username)
                } else {
                    tap::messages::Message::Error(tap::messages::Error::InvalidArguments)
                }
            }
            tap::messages::CommandKind::Take => {
                let mut item = String::new();
                if command.payload.extract(&mut [
                    tap::messages::PayloadExtractor::String(&mut item),
                ]).is_err() {
                    tap::messages::Message::Error(tap::messages::Error::InvalidArguments)
                } else {
                    tap::game::Player::take(game, username, &item).await
                }
            }
            tap::messages::CommandKind::Talk => {
                let mut npc = String::new();
                if command.payload.extract(&mut [
                    tap::messages::PayloadExtractor::String(&mut npc),
                ]).is_err() {
                    tap::messages::Message::Error(tap::messages::Error::InvalidArguments)
                } else {
                    tap::game::Npc::talk(game, username, &npc).await
                }
            }
            tap::messages::CommandKind::Unequip => {
                let mut item = String::new();
                if command.payload.extract(&mut [
                    tap::messages::PayloadExtractor::String(&mut item),
                ]).is_err() {
                    tap::messages::Message::Error(tap::messages::Error::InvalidArguments)
                } else {
                    tap::game::Player::unequip(game, username, &item).await
                }
            }
            tap::messages::CommandKind::Who => {
                if command.payload.is_empty() {
                    tap::game::Player::count(game)
                } else {
                    tap::messages::Message::Error(tap::messages::Error::InvalidArguments)
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    match Cli::start().await {
        Ok(_) => (),
        Err(e) => tap::cli::Logger::error(&format!("{e}")).await,
    };
}

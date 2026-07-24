enum LogKind {
    Error,
    Info,
    Warning,
}

impl std::fmt::Display for LogKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "\x1b[31mERROR\x1b[0m"),
            Self::Info => write!(f, "\x1b[34mINFO\x1b[0m"),
            Self::Warning => write!(f, "\x1b[33mWARN\x1b[0m"),
        }
    }
}

static LOCK: std::sync::LazyLock<tokio::sync::Mutex<()>> = std::sync::LazyLock::new(|| tokio::sync::Mutex::new(()));

pub struct Logger;

impl Logger {
    async fn log(kind: LogKind, s: &str) {
        let _guard = LOCK.lock().await;
        let s = format!(
            "\x1b[90m[{}]\x1b[0m {}: {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            kind,
            s,
        );
        if matches!(kind, LogKind::Error) {
            eprintln!("{s}");
        } else {
            println!("{s}");
        }
    }

    pub async fn error(s: &str) {
        Self::log(LogKind::Error, s).await;
    }

    pub async fn info(s: &str) {
        Self::log(LogKind::Info, s).await;
    }

    pub async fn warning(s: &str) {
        Self::log(LogKind::Warning, s).await;
    }

    async fn message(prefix: &str, client: &crate::network::Client, username: &String, message: &String) {
        Logger::info(&format!(
            "{} \x1b[0;35m{}\x1b[0;0m (\x1b[0;36m{}\x1b[0;0m): {}",
            prefix,
            if username.is_empty() { "?" } else { &username },
            client.addr,
            message,
        )).await;
    }

    pub async fn from(client: &crate::network::Client, username: &String, message: &String) {
        Self::message("From", client, username, message).await;
    }

    pub async fn to(client: &crate::network::Client, username: &String, message: &crate::messages::Message) {
        Self::message("To", client, username, &message.to_string()).await;
        if let Some(writer) = &client.writer {
            let _ = writer.write_message(message).await;
        }
    }

    pub async fn event<F: FnMut(&crate::game::Player) -> bool>(details: &str, game: &crate::game::GameState, event: &crate::messages::Event, mut filter: F) {
        Logger::info(&format!(
            "{} event: {}",
            match event.scope {
                crate::messages::EventScope::Global => "Global".to_string(),
                crate::messages::EventScope::Group => format!("Group \x1b[0;35m{}\x1b[0;0m", details),
                crate::messages::EventScope::Player => format!("Player \x1b[0;35m{}\x1b[0;0m", details),
                crate::messages::EventScope::Room => format!("Room \x1b[0;35m{}\x1b[0;0m", details),
                crate::messages::EventScope::Stats => "Stats".to_string(),
            },
            event,
        )).await;
        for (_, player) in game.players.iter().filter(|(_, player)| filter(player)) {
            if let Some(writer) = &player.writer {
                let _ = writer.write_message(&crate::messages::Message::Event(event.clone())).await;
            }
        }
    }

    pub async fn player_count(game: &crate::game::GameState) {
        Self::event(
            "",
            game,
            &crate::messages::Event {
                scope: crate::messages::EventScope::Stats,
                kind: crate::messages::EventKind::Players,
                payload: crate::messages::Payload::new(&[
                    crate::messages::PayloadKind::KeyValue {
                        key: "players".to_string(),
                        value: game.players.len().to_string(),
                    },
                ]),
            },
            |_| true,
        ).await;
    }
}

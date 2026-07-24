use clap::Parser;

#[derive(Parser)]
#[command(about = "A Multi-User Dungeon client which use the TAP protocol")]
struct Args {
    /// The server binding ip address
    #[arg(long, short)]
    ip: Option<String>,

    /// The server binding port
    #[arg(long, short)]
    port: Option<String>,

    /// If enable, enter into raw client
    #[arg(long, short, action = clap::ArgAction::SetTrue)]
    raw: bool,

    /// If enable, enter into gui client
    #[arg(long, short, action = clap::ArgAction::SetTrue)]
    gui: bool,
}

async fn start() -> Option<tap::messages::Error> {
    let args = Args::parse();
    let ip = match args.ip {
        Some(v) => v,
        None => "127.0.0.1".to_string(),
    };
    let port = match args.port {
        Some(v) => v,
        None => "7373".to_string(),
    };
    let mut client = tap::network::Client {
        addr: format!("{ip}:{port}"),
        ..Default::default()
    };
    if client.connect().await.is_err() {
        return Some(tap::messages::Error::ConnectionFailed);
    }
    if args.raw {
        tap::cli::RawCli::default().start(client).await
    } else if args.gui {
        tap::gui::MyApp::default().start(client)
    } else {
        tap::cli::FriendlyCli::default().start(client).await
    }
}

#[tokio::main]
async fn main() {
    if let Some(e) = start().await {
        eprintln!("{e}");
    };
}

#[derive(Default)]
pub enum ServerState {
    #[default]
    Disconnected,
    Binded,
    Terminated,
}

impl std::fmt::Display for ServerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Binded => write!(f, "Binded"),
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Terminated => write!(f, "Terminated"),
        }
    }
}

#[derive(Default)]
pub struct Server {
    pub state: ServerState,
    pub addr: String,
    listener: Option<tokio::net::TcpListener>,
}

impl Server {
    pub fn new(addr: &str) -> Self {
        Self {
            state: ServerState::Disconnected,
            addr: addr.to_string(),
            listener: None,
        }
    }

    pub fn close(&mut self) {
        self.state = ServerState::Terminated;
        self.listener = None;
    }

    pub async fn bind(&mut self) -> Result<(), std::io::Error> {
        if self.listener.is_some() {
            return Err(std::io::Error::other("already connected"));
        }
        self.listener = match tokio::net::TcpListener::bind(&self.addr).await {
            Ok(v) => Some(v),
            Err(e) => return Err(std::io::Error::new(
                e.kind(),
                format!("bind to '{}' failed: {}", self.addr, e),
            ))
        };
        self.state = ServerState::Binded;
        Ok(())
    }

    pub async fn accept(&mut self) -> Result<crate::network::Client, std::io::Error> {
        match &self.listener {
            Some(listener) => match listener.accept().await {
                Ok((stream, addr)) => Ok(crate::network::Client::new(addr, stream)),
                Err(e) => {
                    self.close();
                    Err(std::io::Error::new(
                        e.kind(),
                        format!("accept failed: {e}"),
                    ))
                },
            }
            None => Err(std::io::Error::other("not binded")),
        }
    }
}

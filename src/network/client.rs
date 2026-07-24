use tokio::io::{
    AsyncReadExt,
    AsyncWriteExt,
};

#[derive(Default)]
pub enum ClientState {
    #[default]
    Disconnected,
    Authenticated,
    Connected,
    Terminated,
}

impl std::fmt::Display for ClientState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Authenticated => write!(f, "Authenticated"),
            Self::Connected => write!(f, "Connected"),
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Terminated => write!(f, "Terminated"),
        }
    }
}

#[derive(Default)]
pub struct Client {
    pub state: ClientState,
    pub addr: String,
    pub proto: String,
    pub reader: Reader,
    pub writer: Option<std::sync::Arc<Writer>>,
}

impl Client {
    pub fn new(addr: std::net::SocketAddr, stream: tokio::net::TcpStream) -> Self {
        let (reader, writer) = stream.into_split();
        Self {
            state: ClientState::Connected,
            addr: addr.to_string(),
            proto: "1".to_string(),
            reader: Reader::new(reader),
            writer: Some(std::sync::Arc::new(Writer::new(writer))),
        }
    }

    pub fn is_open(&self) -> bool {
        self.reader.is_open()
    }

    pub fn close(&mut self) {
        self.state = ClientState::Terminated;
        self.reader.close();
    }

    pub async fn connect(&mut self) -> Result<(), std::io::Error> {
        if self.is_open() {
            return Err(std::io::Error::other("already connected"));
        }
        match tokio::net::TcpStream::connect(&self.addr).await {
            Ok(stream) => {
                let (reader, writer) = stream.into_split();
                self.reader = Reader::new(reader);
                self.writer = Some(std::sync::Arc::new(Writer::new(writer)));
            },
            Err(e) => return Err(std::io::Error::new(
                e.kind(),
                format!("connection to '{}' failed: {}", &self.addr, e),
            )),
        };
        self.state = ClientState::Connected;
        Ok(())
    }
}

#[derive(Default)]
pub struct Reader {
    reader: Option<crate::utils::Shared<tokio::net::tcp::OwnedReadHalf>>,
    buffer: String,
}

impl Reader {
    pub fn new(reader: tokio::net::tcp::OwnedReadHalf) -> Self {
        Self {
            reader: Some(
                crate::utils::Shared::new(
                    reader
                )
            ),
            buffer: String::new(),
        }
    }

    pub fn is_open(&self) -> bool {
        self.reader.is_some()
    }

    pub fn close(&mut self) {
        self.reader = None;
    }

    pub async fn read(&mut self) -> Result<Option<String>, std::io::Error> {
        loop {
            if let Some(i) = self.buffer.find('\n') {
                let message = self.buffer.drain(..i).collect::<String>();
                self.buffer.remove(0);
                return Ok(Some(message));
            }
            let mut buffer = [0u8; 1024];
            match &mut self.reader {
                Some(reader) => {
                    let n = reader.lock().await.read(&mut buffer).await;
                    match n {
                        Ok(0) => {
                            self.close();
                            return Ok(None);
                        }
                        Ok(v) => {
                            self.buffer += &String::from_utf8_lossy(&buffer[..v]);
                            if self.buffer.chars().count() > 4096 {
                                self.buffer = self.buffer
                                    .chars()
                                    .rev()
                                    .take(4096)
                                    .collect::<Vec<_>>()
                                    .into_iter()
                                    .rev()
                                    .collect();
                            }
                        },
                        Err(e) => {
                            self.close();
                            return Err(std::io::Error::new(
                                e.kind(),
                                format!("read failed: {e}"),
                            ))
                        }
                    }
                }
                None => return Err(std::io::Error::other("not connected"))
            }
        }
    }
}

pub struct Writer {
    writer: crate::utils::Shared<tokio::net::tcp::OwnedWriteHalf>,
}

impl Writer {
    pub fn new(writer: tokio::net::tcp::OwnedWriteHalf) -> Self {
        Self {
            writer: crate::utils::Shared::new(writer),
        }
    }

    pub async fn write(&self, s: &str) -> Result<(), std::io::Error> {
        let r = self.writer.lock().await.write_all(s.as_bytes()).await;
        match r {
            Ok(_) => Ok(()),
            Err(e) => Err(std::io::Error::other(format!("write failed: {e}"))),
        }
    }

    pub async fn write_message(&self, message: &crate::messages::Message) -> Result<(), std::io::Error> {
        self.write(&format!("{message}\n")).await
    }
}

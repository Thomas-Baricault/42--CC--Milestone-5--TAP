#[derive(Clone)]
pub enum Message {
    Command(crate::messages::Command),
    Error(crate::messages::Error),
    Event(crate::messages::Event),
    Response(crate::messages::Response),
}

impl std::str::FromStr for Message {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(v) = crate::messages::Command::from_str(s) {
            return Ok(Self::Command(v));
        }
        if let Ok(v) = crate::messages::Error::from_str(s) {
            return Ok(Self::Error(v));
        }
        if let Ok(v) = crate::messages::Event::from_str(s) {
            return Ok(Self::Event(v));
        }
        if let Ok(v) = crate::messages::Response::from_str(s) {
            return Ok(Self::Response(v));
        }
        Err(crate::utils::invalid_input(&format!("invalid message '{s}'")))
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Command(v) => write!(f, "{v}"),
            Self::Error(v) => write!(f, "{v}"),
            Self::Event(v) => write!(f, "{v}"),
            Self::Response(v) => write!(f, "{v}"),
        }
    }
}

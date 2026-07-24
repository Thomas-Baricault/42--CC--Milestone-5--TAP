use strum::IntoEnumIterator;

#[derive(
    Clone,
    strum_macros::EnumIter,
)]
pub enum EventScope {
    Global,
    Group,
    Player,
    Stats,
    Room,
}

impl std::str::FromStr for EventScope {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GLOBAL" => Ok(Self::Global),
            "GROUP" => Ok(Self::Group),
            "PLAYER" => Ok(Self::Player),
            "STATS" => Ok(Self::Stats),
            "ROOM" => Ok(Self::Room),
            _ => Err(crate::utils::invalid_input(&format!("invalid scope '{s}'"))),
        }
    }
}

impl std::fmt::Display for EventScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Global => write!(f, "GLOBAL"),
            Self::Group => write!(f, "GROUP"),
            Self::Player => write!(f, "PLAYER"),
            Self::Stats => write!(f, "STATS"),
            Self::Room => write!(f, "ROOM"),
        }
    }
}

#[derive(
    Clone,
    strum_macros::EnumIter,
)]
pub enum EventKind {
    Chat,
    CombatEnd,
    CombatStats,
    Die,
    Invite,
    Join,
    Leave,
    Players,
    PresenceEnter,
    PresenceLeave,
    QuestComplete,
}

impl std::fmt::Display for EventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Chat => write!(f, "CHAT"),
            Self::CombatEnd => write!(f, "COMBAT END"),
            Self::CombatStats => write!(f, "COMBAT STATS"),
            Self::Die => write!(f, "DIE"),
            Self::Invite => write!(f, "INVITE"),
            Self::Join => write!(f, "JOIN"),
            Self::Leave => write!(f, "LEAVE"),
            Self::Players => write!(f, "PLAYERS"),
            Self::PresenceEnter => write!(f, "PRESENCE ENTER"),
            Self::PresenceLeave => write!(f, "PRESENCE LEAVE"),
            Self::QuestComplete => write!(f, "QUEST COMPLETE"),
        }
    }
}

#[derive(Clone)]
pub struct Event {
    pub scope: EventScope,
    pub kind: EventKind,
    pub payload: crate::messages::Payload,
}

impl std::str::FromStr for Event {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut message = s.to_string();
        let err = Err(crate::utils::invalid_input(&format!("invalid event '{s}'")));
        if !crate::messages::utils::parse_begin(&mut message, "EVT") {
            return err;
        }
        if matches!(crate::messages::utils::skip_space(&mut message), Ok(false) | Err(_)) {
            return err;
        }
        for scope in EventScope::iter() {
            if crate::messages::utils::parse_begin(&mut message, &scope.to_string()) {
                if matches!(crate::messages::utils::skip_space(&mut message), Ok(false) | Err(_)) {
                    return err;
                }
                for kind in EventKind::iter() {
                    if crate::messages::utils::parse_begin(&mut message, &kind.to_string()) {
                        return Ok(Event {
                            scope,
                            kind,
                            payload: match crate::messages::utils::parse_payload(&mut message) {
                                Ok(v) => v,
                                Err(_) => return err,
                            }
                        });
                    }
                }
                return err;
            }
        }
        err
    }
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::messages::utils::write_vec(f, vec![
            "EVT".to_string(),
            self.scope.to_string(),
            self.kind.to_string(),
            self.payload.to_string(),
        ])
    }
}

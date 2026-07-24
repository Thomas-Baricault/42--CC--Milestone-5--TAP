use strum::IntoEnumIterator;

#[derive(
    Clone,
    strum_macros::EnumIter,
)]
pub enum Error {
    UnknownError,
    NotACommand,
    UnknownCommand,
    InvalidArguments,
    AlreadyAuthenticated,
    NotAuthenticated,
    NameInUse,
    InvalidName,
    NoExit,
    NotInGroup,
    AlreadyInGroup,
    NotInvited,
    PlayerNotFound,
    GroupNotFound,
    ItemNotFound,
    ItemNotInInventory,
    ItemNotEquiped,
    NPCNotFound,
    QuestNotFound,
    NPCNotHostile,
    NoQuestAvailable,
    NPCNotNeutral,
    ItemNotConsumable,
    ItemNotEquipable,
    QuestNotActive,
    InCombat,
    ConnectionFailed,
    SendFailed,
    ConnectionClosed,
    UnexpectedServerResponse,
    ServerTimeOut,
    ServerError,
}

impl Error {
    pub fn code(&self) -> u16 {
        match self {
            Self::UnknownError => 1,
            Self::NotACommand => 100,
            Self::UnknownCommand => 101,
            Self::InvalidArguments => 102,
            Self::AlreadyAuthenticated => 103,
            Self::NotAuthenticated => 104,
            Self::NameInUse => 201,
            Self::InvalidName => 202,
            Self::NoExit => 301,
            Self::NotInGroup => 401,
            Self::AlreadyInGroup => 402,
            Self::NotInvited => 403,
            Self::PlayerNotFound => 404,
            Self::GroupNotFound => 404,
            Self::ItemNotFound => 404,
            Self::ItemNotInInventory => 404,
            Self::ItemNotEquiped => 404,
            Self::NPCNotFound => 404,
            Self::QuestNotFound => 404,
            Self::NPCNotHostile => 405,
            Self::NoQuestAvailable => 406,
            Self::NPCNotNeutral => 407,
            Self::ItemNotConsumable => 408,
            Self::ItemNotEquipable => 409,
            Self::QuestNotActive => 410,
            Self::InCombat => 411,
            Self::ConnectionFailed => 900,
            Self::SendFailed => 901,
            Self::ConnectionClosed => 902,
            Self::UnexpectedServerResponse => 903,
            Self::ServerTimeOut => 904,
            Self::ServerError => 905,
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            Self::UnknownError => "UNKNOWN_ERROR",
            Self::NotACommand => "NOT_A_COMMAND",
            Self::UnknownCommand => "UNKNOWN_COMMAND",
            Self::InvalidArguments => "INVALID_ARGUMENTS",
            Self::AlreadyAuthenticated => "ALREADY_AUTHENTICATED",
            Self::NotAuthenticated => "NOT_AUTHENTICATED",
            Self::NameInUse => "NAME_IN_USE",
            Self::InvalidName => "INVALID_NAME",
            Self::NoExit => "NO_EXIT",
            Self::NotInGroup => "NOT_IN_GROUP",
            Self::AlreadyInGroup => "ALREADY_IN_GROUP",
            Self::NotInvited => "NOT_INVITED",
            Self::PlayerNotFound => "PLAYER_NOT_FOUND",
            Self::GroupNotFound => "GROUP_NOT_FOUND",
            Self::ItemNotFound => "ITEM_NOT_FOUND",
            Self::ItemNotInInventory => "ITEM_NOT_IN_INVENTORY",
            Self::ItemNotEquiped => "ITEM_NOT_EQUIPED",
            Self::NPCNotFound => "NPC_NOT_FOUND",
            Self::QuestNotFound => "QUEST_NOT_FOUND",
            Self::NPCNotHostile => "NPC_NOT_HOSTILE",
            Self::NoQuestAvailable => "NO_QUEST_AVAILABLE",
            Self::NPCNotNeutral => "NPC_NOT_NEUTRAL",
            Self::ItemNotConsumable => "ITEM_NOT_CONSUMABLE",
            Self::ItemNotEquipable => "ITEM_NOT_EQUIPABLE",
            Self::QuestNotActive => "QUEST_NOT_ACTIVE",
            Self::InCombat => "IN_COMBAT",
            Self::ConnectionFailed => "CONNECTION_FAILED",
            Self::SendFailed => "SEND_FAILED",
            Self::ConnectionClosed => "CONNECTION_CLOSED",
            Self::UnexpectedServerResponse => "UNEXPECTED_SERVER_RESPONSE",
            Self::ServerTimeOut => "SERVER_TIME_OUT",
            Self::ServerError => "SERVER_ERROR",
        }
    }

    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            Self::ConnectionFailed |
            Self::SendFailed |
            Self::ConnectionClosed |
            Self::UnexpectedServerResponse |
            Self::ServerTimeOut |
            Self::ServerError,
        )
    }
}

impl std::str::FromStr for Error {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("ERR ") {
            return Err(crate::utils::invalid_input(&format!("invalid error '{s}'")));
        }
        for kind in Error::iter() {
            if s == kind.to_string() {
                return Ok(kind);
            }
        }
        Ok(Error::UnknownError)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::messages::utils::write_vec(f, vec![
            "ERR".to_string(),
            self.code().to_string(),
            self.message().to_string(),
        ])
    }
}

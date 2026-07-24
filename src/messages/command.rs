use strum::IntoEnumIterator;

#[derive(
    Clone,
    strum_macros::EnumIter,
)]
pub enum CommandKind {
    AbandonQuest,
    Attack,
    Chat,
    Connect,
    Consume,
    Describe,
    Drop,
    Equip,
    GroupDescribe,
    GroupCreate,
    GroupInvite,
    GroupJoin,
    GroupLeave,
    Inventory,
    Look,
    Move,
    Quests,
    Quest,
    Quit,
    Status,
    Take,
    Talk,
    Unequip,
    Who,
}

impl CommandKind {
    pub fn requires_auth(&self) -> bool {
        !matches!(
            self,
            Self::Connect |
            Self::Quit,
        )
    }
}

impl std::fmt::Display for CommandKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AbandonQuest => write!(f, "ABANDON QUEST"),
            Self::Attack => write!(f, "ATTACK"),
            Self::Chat => write!(f, "CHAT"),
            Self::Connect => write!(f, "CONNECT"),
            Self::Consume => write!(f, "CONSUME"),
            Self::Describe => write!(f, "DESCRIBE"),
            Self::Drop => write!(f, "DROP"),
            Self::Equip => write!(f, "EQUIP"),
            Self::GroupDescribe => write!(f, "GROUP DESCRIBE"),
            Self::GroupCreate => write!(f, "GROUP CREATE"),
            Self::GroupInvite => write!(f, "GROUP INVITE"),
            Self::GroupJoin => write!(f, "GROUP JOIN"),
            Self::GroupLeave => write!(f, "GROUP LEAVE"),
            Self::Inventory => write!(f, "INVENTORY"),
            Self::Look => write!(f, "LOOK"),
            Self::Move => write!(f, "MOVE"),
            Self::Quest => write!(f, "QUEST"),
            Self::Quests => write!(f, "QUESTS"),
            Self::Quit => write!(f, "QUIT"),
            Self::Status => write!(f, "STATUS"),
            Self::Take => write!(f, "TAKE"),
            Self::Talk => write!(f, "TALK"),
            Self::Unequip => write!(f, "UNEQUIP"),
            Self::Who => write!(f, "WHO"),
        }
    }
}

#[derive(Clone)]
pub struct Command {
    pub kind: CommandKind,
    pub payload: crate::messages::Payload,
}

impl Command {
    pub fn new(kind: CommandKind) -> Self {
        Self {
            kind,
            payload: crate::messages::Payload::default(),
        }
    }
}

impl std::str::FromStr for Command {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut message = s.to_string();
        for kind in CommandKind::iter() {
            if crate::messages::utils::parse_begin(&mut message, &kind.to_string()) {
                return Ok(Command {
                    kind,
                    payload: match crate::messages::utils::parse_payload(&mut message) {
                        Ok(v) => v,
                        Err(_) => return Err(crate::utils::invalid_input(&format!("invalid command '{s}'"))),
                    }
                });
            }
        }
        Err(crate::utils::invalid_input(&format!("invalid command '{s}'")))
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::messages::utils::write_vec(f, vec![
            self.kind.to_string(),
            self.payload.to_string(),
        ])
    }
}

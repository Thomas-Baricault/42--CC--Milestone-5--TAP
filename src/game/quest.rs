#[derive(
    Clone,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum QuestKind {
    Bring {
        item: String,
        count: u32,
    },
    Goto {
        room: String,
    },
    Kill {
        enemy: String,
        count: u32,
    },
    Talk {
        npc: String,
    }
}

impl Default for QuestKind {
    fn default() -> Self {
        QuestKind::Goto {
            room: String::new(),
        }
    }
}

#[derive(
    Clone,
    Default,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
pub struct Quest {
    pub id: String,
    pub name: String,
    pub description: String,

    #[serde(default)]
    pub thanks: String,

    #[serde(default)]
    pub autocomplete: bool,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub requirements: Vec<String>,

    pub task: QuestKind,
    pub reward: Vec<String>,
}

#[derive(
    Clone,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(rename_all = "lowercase")]
pub enum QuestStatus {
    Abandoned,
    Active,
    Completed,
}

impl std::fmt::Display for QuestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Abandoned => write!(f, "Abandoned"),
            Self::Active => write!(f, "Active"),
            Self::Completed => write!(f, "Completed"),
        }
    }
}

#[derive(
    Clone,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct QuestProgress {
    pub giver: String,
    pub quest: String,
    pub status: QuestStatus,
    pub progress: u32,
}

impl QuestProgress {
    pub fn new(giver: String, quest: &Quest) -> Self {
        Self {
            giver,
            quest: quest.id.clone(),
            status: QuestStatus::Active,
            progress: 0,
        }
    }

    pub fn is_complete(&self, quests: &std::collections::HashMap<String, crate::game::Quest>) -> bool {
        let reference = &quests[&self.quest];
        match &reference.task {
            crate::game::QuestKind::Bring {
                item: _,
                count,
            } => self.progress == *count,
            crate::game::QuestKind::Goto {
                room: _,
            } => self.progress == 1,
            crate::game::QuestKind::Kill {
                enemy: _,
                count,
            } => self.progress == *count,
            crate::game::QuestKind::Talk {
                npc: _,
            } => self.progress == 1,
        }
    }
}

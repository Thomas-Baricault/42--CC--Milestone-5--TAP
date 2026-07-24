#[derive(
    Clone,
    Default,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct EnemyStatus {
    pub id: String,
    pub hp: u32,
    pub max_hp: u32,
    pub armor: u32,
    pub attack: u32,
}

#[derive(
    Clone,
    Default,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct Combat {
    pub players: Vec<crate::game::PlayerStatus>,
    pub enemies: Vec<EnemyStatus>,
}

impl Combat {
    pub fn index(&self, username: &String) -> Option<usize> {
        (0..self.players.len()).find(|&i| self.players[i].username == *username)
    }
}

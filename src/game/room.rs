#[derive(
    Clone,
    Default,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub description: String,
    pub exits: std::collections::HashMap<crate::game::Direction, String>,
}

#[derive(
    Default,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
pub struct RoomState {
    pub room: Room,
    pub players: std::collections::HashSet<String>,
    pub items: Vec<String>,
    pub npcs: Vec<String>,

    #[serde(skip)]
    pub dead_npcs: Vec<String>,

    #[serde(skip)]
    pub combat: crate::game::Combat,
}

impl RoomState {
    pub fn new(room: Room) -> Self {
        Self {
            room,
            players: std::collections::HashSet::new(),
            items: Vec::new(),
            npcs: Vec::new(),
            dead_npcs: Vec::new(),
            combat: crate::game::Combat::default(),
        }
    }

    pub async fn enter(game: &mut crate::game::GameState, player: &String, room: &String) {
        if let Some(player) = game.players.get_mut(player) {
            player.room = room.clone();
        }
        if let Some(room) = game.rooms.get_mut(room) {
            room.players.insert(player.clone());
        }
        crate::cli::Logger::event(
            room,
            game,
            &crate::messages::Event {
                scope: crate::messages::EventScope::Room,
                kind: crate::messages::EventKind::PresenceEnter,
                payload: crate::messages::Payload::new(&[
                    crate::messages::PayloadKind::String(player.clone()),
                ]),
            },
            |to| to.username != *player && to.room == *room,
        ).await;
    }

    pub async fn leave(game: &mut crate::game::GameState, player: &String) {
        let mut combat_update = false;
        if let Some(player) = game.players.get(player) {
            if let Some(room) = game.rooms.get_mut(&player.room) {
                room.players.remove(&player.username);
                if let Some(i) = room.combat.players.iter().position(|i| i.username == player.username) {
                    combat_update = true;
                    room.combat.players.remove(i);
                }
                if room.combat.players.is_empty() {
                    room.combat.enemies.clear();
                }
                if room.players.is_empty() {
                    room.npcs.append(&mut room.dead_npcs);
                }
            }
            crate::cli::Logger::event(
                &player.room,
                game,
                &crate::messages::Event {
                    scope: crate::messages::EventScope::Room,
                    kind: crate::messages::EventKind::PresenceLeave,
                    payload: crate::messages::Payload::new(&[
                        crate::messages::PayloadKind::String(player.username.clone()),
                    ]),
                },
                |to| to.username != player.username && to.room == player.room,
            ).await;
        }
        if combat_update {
            let room = &game.players[player].room;
            crate::cli::Logger::event(
                room,
                game,
                &crate::messages::Event {
                    scope: crate::messages::EventScope::Room,
                    kind: crate::messages::EventKind::CombatStats,
                    payload: crate::messages::Payload::new(&[
                        crate::messages::PayloadKind::new_json(&game.rooms[room].combat),
                    ]),
                },
                |to| to.username != *player && to.room == *room,
            ).await;
        }
    }
}

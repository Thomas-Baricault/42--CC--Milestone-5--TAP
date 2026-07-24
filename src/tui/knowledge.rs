#[derive(Default)]
pub struct Knowledge {
    pub addr: String,
    pub proto: String,
    pub players: usize,
    pub describes: std::collections::HashSet<String>,
    pub items: std::collections::HashMap<String, crate::game::Item>,
    pub npcs: std::collections::HashMap<String, crate::game::Npc>,
    pub quests: std::collections::HashMap<String, crate::game::Quest>,
    pub rooms: std::collections::HashMap<String, crate::game::Room>,
    pub positions: std::collections::HashMap<String, (i32, i32)>,
    pub rpositions: std::collections::HashMap<(i32, i32), String>,
    pub connections: std::collections::HashSet<((i32, i32), (i32, i32))>,
    pub room: crate::game::RoomState,
    pub group: crate::game::Group,
    pub invitations: std::collections::HashSet<String>,
    pub player: crate::game::Player,

	pub ui_popup: Option<(String, String)>,
	pub ui_popup_show: bool,
	pub chat: Vec<crate::tui::ChatMessage>,
	pub last_error: Option<crate::messages::Error>
}

impl Knowledge {
    pub fn change_room(&mut self, room: crate::game::RoomState) {
        if !self.positions.contains_key(&room.room.id) {
            let mut pos = (0, 0);
            for (direction, id) in &room.room.exits {
                if let Some(other) = self.positions.get(id) {
                    match direction {
                        crate::game::Direction::East => pos = (other.0 - 1, other.1),
                        crate::game::Direction::North => pos = (other.0, other.1 + 1),
                        crate::game::Direction::South => pos = (other.0, other.1 - 1),
                        crate::game::Direction::West => pos = (other.0 + 1, other.1),
                    }
                    break;
                }
            }
            self.rooms.insert(room.room.id.clone(), room.room.clone());
            self.positions.insert(room.room.id.clone(), pos);
            self.rpositions.insert(pos, room.room.id.clone());
            for (direction, id) in &room.room.exits {
                let (other, connection) = match direction {
                    crate::game::Direction::East => (
                        (pos.0 + 1, pos.1),
                        (pos, (pos.0 + 1, pos.1)),
                    ),
                    crate::game::Direction::North => (
                        (pos.0, pos.1 - 1),
                        ((pos.0, pos.1 - 1), pos),
                    ),
                    crate::game::Direction::South => (
                        (pos.0, pos.1 + 1),
                        (pos, (pos.0, pos.1 + 1)),
                    ),
                    crate::game::Direction::West => (
                        (pos.0 - 1, pos.1),
                        ((pos.0 - 1, pos.1), pos),
                    ),
                };
                self.rpositions.insert(other, id.clone());
                self.connections.insert(connection);
            }
        }
        self.player.room = room.room.id.clone();
        let combat = self.room.combat.clone();
        self.room = room;
        self.room.combat = combat;
    }

    pub fn change_group(&mut self, group: Option<crate::game::Group>) {
        if let Some(group) = group {
            self.player.group = group.name.clone();
            self.group = group;
        } else {
            self.player.group.clear();
        }
    }

    pub fn update(&mut self, data: crate::game::WorldData) -> Result<(), std::io::Error> {
        match data {
            crate::game::WorldData::Item(item) => {
                self.describes.remove(&item.id);
                self.items.insert(item.id.clone(), item);
            },
            crate::game::WorldData::Npc(npc) => {
                self.describes.remove(&npc.id);
                self.npcs.insert(npc.id.clone(), npc);
            },
            crate::game::WorldData::Quest(quest) => {
                self.describes.remove(&quest.id);
                self.quests.insert(quest.id.clone(), quest);
            },
            crate::game::WorldData::Room(room) => {
                self.describes.remove(&room.id);
                self.rooms.insert(room.id.clone(), room);
            },
            _ => return Err(std::io::Error::other("invalid data kind")),
        };
        Ok(())
    }

    pub fn need(&mut self) -> Option<String> {
        if let Some(v) = self.describes.iter().next().cloned() {
            self.describes.take(&v)
        } else {
            None
        }
    }

    pub fn item_name(&mut self, id: &str) -> String {
        if let Some(item) = self.items.get(id) {
            item.name.clone()
        } else {
            self.describes.insert(id.to_string());
            format!("{{{}}}", id)
        }
    }

    pub fn npc_name(&mut self, id: &str) -> String {
        if let Some(npc) = self.npcs.get(id) {
            npc.name.clone()
        } else {
            self.describes.insert(id.to_string());
            format!("{{{}}}", id)
        }
    }

    pub fn room_name(&mut self, id: &str) -> String {
        if let Some(room) = self.rooms.get(id) {
            room.name.clone()
        } else {
            self.describes.insert(id.to_string());
            format!("{{{}}}", id)
        }
    }
	pub fn show_popup(&mut self, title: impl Into<String>, content: impl Into<String>) {
        self.ui_popup = Some((title.into(), content.into()));
        self.ui_popup_show = true;
    }
}

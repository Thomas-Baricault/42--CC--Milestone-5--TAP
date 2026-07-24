#[derive(
    Default,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum WorldData {
    #[default]
    None,
    Group(crate::game::Group),
    Item(crate::game::Item),
    Npc(crate::game::Npc),
    Quest(crate::game::Quest),
    Room(crate::game::Room),
}

fn default_count() -> usize {
    1
}

#[derive(
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
enum SpawnKind {
    Item {
        room: String,
        item: String,

        #[serde(default = "default_count")]
        count: usize,
    },
    Npc {
        room: String,
        npc: String,
    },
}

#[derive(
    Default,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
struct World {
    pub rooms: Vec<crate::game::Room>,
    pub items: Vec<crate::game::Item>,
    pub npcs: Vec<crate::game::Npc>,
    pub quests: Vec<crate::game::Quest>,
    pub spawns: Vec<SpawnKind>,
}

#[derive(Default)]
pub struct GameState {
    pub players: std::collections::HashMap<String, crate::game::Player>,
    pub groups: std::collections::HashMap<String, crate::game::Group>,
    pub rooms: std::collections::HashMap<String, crate::game::RoomState>,
    pub items: std::collections::HashMap<String, crate::game::Item>,
    pub npcs: std::collections::HashMap<String, crate::game::Npc>,
    pub quests: std::collections::HashMap<String, crate::game::Quest>,
}

impl GameState {
    pub fn new(path: &str) -> Result<Self, std::io::Error> {
        fn err_no_identified(kind: &str, id: &str) -> Result<GameState, std::io::Error> {
            Err(std::io::Error::other(format!("there is no {kind} identified by '{id}'")))
        }

        fn err_duplicated_id(kind: &str, id: &str) -> Result<GameState, std::io::Error> {
            Err(std::io::Error::other(format!("duplicated {kind} id '{id}'")))
        }

        let world: World = serde_json::from_str(
            &std::fs::read_to_string(path)?
        )?;
        if world.rooms.is_empty() {
            return Err(std::io::Error::other("cannot instantiate a world without any room data"));
        }
        let mut game = Self::default();
        for mut room in world.rooms {
            room.id = format!("room.{}", room.id);
            if game.rooms.contains_key(&room.id) {
                return Err(std::io::Error::other(format!(
                    "duplicated room id '{}'",
                    room.id,
                )));
            }
            for (_, id) in room.exits.iter_mut() {
                *id = format!("room.{id}");
            }
            game.rooms.insert(
                room.id.clone(),
                crate::game::RoomState::new(room),
            );
        }
        if !game.rooms.contains_key("room.start") {
            return Err(std::io::Error::other("missing room 'start' used as spawn point"));
        }
        let mut positions: std::collections::HashMap<String, (i32, i32)> = std::collections::HashMap::new();
        positions.insert("room.start".to_string(), (0, 0));
        while positions.len() != game.rooms.len() {
            for (id, room) in &game.rooms {
                if !positions.contains_key(id) {
                    let (mut x, mut y) = (None, None);
                    for (direction, other) in &room.room.exits {
                        if let Some(other) = game.rooms.get(other) {
                            if !other.room.exits.contains_key(&direction.opposite()) || other.room.exits[&direction.opposite()] != room.room.id {
                                return Err(std::io::Error::other(format!(
                                    "the path between '{}' and '{}' is not reversible",
                                    id,
                                    other.room.id,
                                )));
                            }
                        } else {
                            return err_no_identified("room", other);
                        }
                        if let Some(dest) = positions.get(other) {
                            if let Some(x) = x && let Some(y) = y {
                                if match direction {
                                    crate::game::Direction::East => x != dest.0 - 1 || y != dest.1,
                                    crate::game::Direction::North => x != dest.0 || y != dest.1 + 1,
                                    crate::game::Direction::South => x != dest.0 || y != dest.1 - 1,
                                    crate::game::Direction::West => x != dest.0 + 1 || y != dest.1,
                                } {
                                    return Err(std::io::Error::other("the rooms are not arranged in a geometrically correct way"));
                                }
                            } else {
                                match direction {
                                    crate::game::Direction::East => { x = Some(dest.0 - 1); y = Some(dest.1); }
                                    crate::game::Direction::North => { x = Some(dest.0); y = Some(dest.1 + 1); }
                                    crate::game::Direction::South => { x = Some(dest.0); y = Some(dest.1 - 1); }
                                    crate::game::Direction::West => { x = Some(dest.0 + 1); y = Some(dest.1); }
                                }
                            }
                        }
                    }
                    if let Some(x) = x && let Some(y) = y {
                        positions.insert(id.clone(), (x, y));
                    }
                }
            }
        }
        for mut item in world.items {
            item.id = format!("item.{}", item.id);
            if game.items.contains_key(&item.id) {
                return err_duplicated_id("item", &item.id);
            }
            game.items.insert(
                item.id.clone(),
                item,
            );
        }
        for mut quest in world.quests {
            quest.id = format!("quest.{}", quest.id);
            if game.quests.contains_key(&quest.id) {
                return err_duplicated_id("quest", &quest.id);
            }
            for require in quest.requirements.iter_mut() {
                *require = format!("quest.{require}");
            }
            match &mut quest.task {
                crate::game::QuestKind::Bring {
                    item,
                    count: _,
                } => *item = format!("item.{item}"),
                crate::game::QuestKind::Kill {
                    enemy,
                    count: _,
                } => *enemy = format!("npc.{enemy}"),
                crate::game::QuestKind::Talk {
                    npc,
                } => *npc = format!("npc.{npc}"),
                _ => (),
            }
            for item in quest.reward.iter_mut() {
                *item = format!("item.{item}");
            }
            game.quests.insert(
                quest.id.clone(),
                quest,
            );
        }
        for mut npc in world.npcs {
            npc.id = format!("npc.{}", npc.id);
            if game.npcs.contains_key(&npc.id) {
                return err_duplicated_id("Npc", &npc.id);
            }
            if let crate::game::NpcKind::Neutral {
                dialogues: _,
                quests,
            } = &mut npc.data {
                for quest in quests {
                    *quest = format!("quest.{quest}");
                    if !game.quests.contains_key(quest) {
                        return err_no_identified("quest", quest);
                    }
                }
            }
            game.npcs.insert(
                npc.id.clone(),
                npc,
            );
        }
        for quest in game.quests.values() {
            for require in &quest.requirements {
                if !game.quests.contains_key(require) {
                    return err_no_identified("quest", require);
                }
            }
            for item in &quest.reward {
                if !game.items.contains_key(item) {
                    return err_no_identified("item", item);
                }
            }
            match &quest.task {
                crate::game::QuestKind::Bring {
                    item,
                    count: _,
                } => {
                    if !game.items.contains_key(item) {
                        return err_no_identified("item", item);
                    }
                }
                crate::game::QuestKind::Kill {
                    enemy,
                    count: _,
                } => {
                    if let Some(npc) = game.npcs.get(enemy) {
                        if !npc.is_enemy() {
                            return Err(std::io::Error::other(format!(
                                "players will not be able to kill NPC '{}' as it's not an enemy",
                                npc.id,
                            )));
                        }
                    } else {
                        return err_no_identified("NPC", enemy);
                    }
                }
                crate::game::QuestKind::Goto {
                    room,
                } => {
                    if !game.rooms.contains_key(room) {
                        return err_no_identified("room", room);
                    }
                }
                crate::game::QuestKind::Talk {
                    npc,
                } => {
                    if let Some(npc) = game.npcs.get(npc) {
                        if npc.is_enemy() {
                            return Err(std::io::Error::other(format!(
                                "players will not be able to talk to NPC '{}' as it's an enemy",
                                npc.id,
                            )));
                        }
                    } else {
                        return err_no_identified("NPC", npc);
                    }
                }
            }
        }
        for spawn in world.spawns {
            match spawn {
                SpawnKind::Item {
                    mut room,
                    mut item,
                    count
                } => {
                    room = format!("room.{room}");
                    item = format!("item.{item}");
                    if !game.items.contains_key(&item) {
                        return Err(std::io::Error::other(format!("there is no item identified by '{item}'")));
                    }
                    if let Some(room) = game.rooms.get_mut(&room) {
                        room.items.extend(std::iter::repeat_n(item, count));
                    } else {
                        return Err(std::io::Error::other(format!("there is no room identified by '{room}'")));
                    }
                }
                SpawnKind::Npc {
                    mut room,
                    mut npc,
                } => {
                    room = format!("room.{room}");
                    npc = format!("npc.{npc}");
                    if !game.npcs.contains_key(&npc) {
                        return Err(std::io::Error::other(format!("there is no npc identified by '{npc}'")));
                    }
                    if room == "room.start" && game.npcs[&npc].is_enemy() {
                        return Err(std::io::Error::other("enemies cannot spawn in start room"));
                    }
                    if let Some(room) = game.rooms.get_mut(&room) {
                        room.npcs.push(npc);
                    } else {
                        return Err(std::io::Error::other(format!("there is no room identified by '{room}'")));
                    }
                }
            }
        }
        Ok(game)
    }

    pub fn describe(&self, id: &String) -> crate::messages::Message {
        if id.starts_with("item.") {
            if let Some(item) = self.items.get(id) {
                crate::messages::Message::Response(crate::messages::Response {
                    payload: crate::messages::Payload::new(&[
                        crate::messages::PayloadKind::new_json(item),
                    ]),
                })
            } else {
                crate::messages::Message::Error(crate::messages::Error::ItemNotFound)
            }
        } else if id.starts_with("npc.") {
            if let Some(npc) = self.npcs.get(id) {
                crate::messages::Message::Response(crate::messages::Response {
                    payload: crate::messages::Payload::new(&[
                        crate::messages::PayloadKind::new_json(npc),
                    ]),
                })
            } else {
                crate::messages::Message::Error(crate::messages::Error::NPCNotFound)
            }
        } else if id.starts_with("quest.") {
            if let Some(quest) = self.quests.get(id) {
                crate::messages::Message::Response(crate::messages::Response {
                    payload: crate::messages::Payload::new(&[
                        crate::messages::PayloadKind::new_json(quest),
                    ]),
                })
            } else {
                crate::messages::Message::Error(crate::messages::Error::QuestNotFound)
            }
        } else if id.starts_with("room.") {
            if let Some(room) = self.rooms.get(id) {
                crate::messages::Message::Response(crate::messages::Response {
                    payload: crate::messages::Payload::new(&[
                        crate::messages::PayloadKind::new_json(&room.room),
                    ]),
                })
            } else {
                crate::messages::Message::Error(crate::messages::Error::QuestNotFound)
            }
        } else {
            crate::messages::Message::Error(crate::messages::Error::InvalidArguments)
        }
    }
}

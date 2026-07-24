#[derive(
    Clone,
    Default,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct PlayerStatus
{
    pub username: String,
    pub hp: u32,
    pub max_hp: u32,
    pub armor: String,
    pub weapon: String,
}

impl PlayerStatus {
    pub fn new(username: &str) -> Self {
        Self {
            username: username.to_string(),
            hp: 100,
            max_hp: 100,
            armor: String::new(),
            weapon: String::new(),
        }
    }
}

#[derive(Default)]
pub struct Player {
    pub username: String,
    pub status: PlayerStatus,
    pub group: String,
    pub room: String,
    pub items: Vec<String>,
    pub quests: std::collections::HashMap<String, crate::game::QuestProgress>,
    pub writer: Option<std::sync::Arc<crate::network::Writer>>,
}

impl Player {
    pub fn new(username: String, writer: std::sync::Arc<crate::network::Writer>) -> Self {
        Self {
            username: username.clone(),
            status: PlayerStatus::new(&username),
            group: String::new(),
            room: String::new(),
            items: Vec::new(),
            quests: std::collections::HashMap::new(),
            writer: Some(writer),
        }
    }

    pub fn count(game: &crate::game::GameState) -> crate::messages::Message {
        crate::messages::Message::Response(crate::messages::Response {
            payload: crate::messages::Payload::new(&[
                crate::messages::PayloadKind::KeyValue {
                    key: "players".to_string(),
                    value: game.players.len().to_string(),
                },
            ]),
        })
    }

    pub async fn connect(game: &mut crate::game::GameState, client: &mut crate::network::Client, username: &String) -> crate::messages::Message {
        if game.players.contains_key(username) {
            crate::messages::Message::Error(crate::messages::Error::NameInUse)
        } else if username.is_empty() || username.chars().count() > 256 {
            crate::messages::Message::Error(crate::messages::Error::InvalidName)
        } else {
            if let Some(writer) = &client.writer {
                client.state = crate::network::ClientState::Authenticated;
                game.players.insert(
                    username.clone(),
                    crate::game::Player::new(
                        username.clone(),
                        writer.clone(),
                    ),
                );
                crate::cli::Logger::player_count(game).await;
            }
            crate::game::RoomState::enter(game, username, &"room.start".to_string()).await;
            crate::messages::Message::Response(crate::messages::Response {
                payload: crate::messages::Payload::new(&[
                    crate::messages::PayloadKind::String("connected".to_string()),
                ]),
            })
        }
    }

    pub fn status(game: &crate::game::GameState, player: &String) -> crate::messages::Message {
        crate::messages::Message::Response(crate::messages::Response {
            payload: crate::messages::Payload::new(&[
                crate::messages::PayloadKind::new_json(&game.players[player].status),
            ]),
        })
    }

    pub async fn chat(game: &crate::game::GameState, player: &String, scope: &crate::messages::EventScope, message: &str) -> crate::messages::Message {
        let player = &game.players[player];
        crate::cli::Logger::event(
            match scope {
                crate::messages::EventScope::Group => &player.group,
                crate::messages::EventScope::Room => &player.room,
                _ => "",
            },
            game,
            &crate::messages::Event {
                scope: scope.clone(),
                kind: crate::messages::EventKind::Chat,
                payload: crate::messages::Payload::new(&[
                    crate::messages::PayloadKind::String(player.username.clone()),
                    crate::messages::PayloadKind::String(message.to_string()),
                ]),
            },
            |to| match scope {
                crate::messages::EventScope::Group => to.group == player.group,
                crate::messages::EventScope::Room => to.room == player.room,
                _ => true,
            },
        ).await;
        crate::messages::Message::Response(crate::messages::Response::default())
    }

    pub fn inventory(game: &crate::game::GameState, player: &String) -> crate::messages::Message {
        crate::messages::Message::Response(crate::messages::Response {
            payload: crate::messages::Payload::new(&[
                crate::messages::PayloadKind::new_json(&game.players[player].items),
            ]),
        })
    }

    pub fn quests(game: &crate::game::GameState, player: &String) -> crate::messages::Message {
        crate::messages::Message::Response(crate::messages::Response {
            payload: crate::messages::Payload::new(&[
                crate::messages::PayloadKind::new_json(&game.players[player].quests.values().collect::<Vec<&crate::game::QuestProgress>>()),
            ]),
        })
    }

    pub fn look(game: &crate::game::GameState, player: &String) -> crate::messages::Message {
        crate::messages::Message::Response(crate::messages::Response {
            payload: crate::messages::Payload::new(&[
                crate::messages::PayloadKind::new_json(&game.rooms[&game.players[player].room]),
            ]),
        })
    }

    pub fn describe_group(game: &crate::game::GameState, player: &String) -> crate::messages::Message {
        if game.players[player].group.is_empty() {
            crate::messages::Message::Error(crate::messages::Error::NotInGroup)
        } else {
            crate::messages::Message::Response(crate::messages::Response {
                payload: crate::messages::Payload::new(&[
                    crate::messages::PayloadKind::new_json(&game.groups[&game.players[player].group]),
                ]),
            })
        }
    }

    pub async fn move_to(game: &mut crate::game::GameState, player: &String, direction: &crate::game::Direction) -> crate::messages::Message {
        let room = &game.rooms[&game.players[player].room];
        if room.combat.index(player).is_some() {
            return crate::messages::Message::Error(crate::messages::Error::InCombat);
        }
        let name: String;
        if let Some(s) = room.room.exits.get(direction) {
            name = s.clone();
        } else {
            return crate::messages::Message::Error(crate::messages::Error::NoExit);
        }
        crate::game::RoomState::leave(game, player).await;
        crate::game::RoomState::enter(game, player, &name).await;
        Self::update_quests(game, player, "room", &game.players[player].room.clone()).await;
        crate::messages::Message::Response(crate::messages::Response {
            payload: crate::messages::Payload::new(&[
                crate::messages::PayloadKind::KeyValue {
                    key: "room".to_string(),
                    value: name,
                },
            ]),
        })
    }

    pub async fn take(game: &mut crate::game::GameState, player: &String, item: &String) -> crate::messages::Message {
        let id: String;
        if let Some(player) = game.players.get_mut(player) && let Some(room) = game.rooms.get_mut(&player.room) {
            if let Some(i) = room.items.iter().position(|i| *i == *item || game.items[i].name == *item) {
                id = room.items.remove(i);
                player.items.push(id.clone());
            } else {
                return crate::messages::Message::Error(crate::messages::Error::ItemNotFound);
            }
        } else {
            return crate::messages::Message::Error(crate::messages::Error::ServerError);
        }
        Self::update_quests(game, player, "item", &id).await;
        crate::messages::Message::Response(crate::messages::Response {
            payload: crate::messages::Payload::new(&[
                crate::messages::PayloadKind::KeyValue {
                    key: "taken".to_string(),
                    value: id,
                },
            ]),
        })
    }

    pub async fn drop(game: &mut crate::game::GameState, player: &String, item: &String) -> crate::messages::Message {
        let id: String;
        if let Some(player) = game.players.get_mut(player) && let Some(room) = game.rooms.get_mut(&player.room) {
            if let Some(i) = player.items.iter().position(|i| *i == *item || game.items[i].name == *item) {
                id = player.items.remove(i);
                room.items.push(id.clone());
            } else {
                return crate::messages::Message::Error(crate::messages::Error::ItemNotInInventory);
            }
        } else {
            return crate::messages::Message::Error(crate::messages::Error::ServerError)
        }
        Self::update_quests(game, player, "item", &id).await;
        crate::messages::Message::Response(crate::messages::Response {
            payload: crate::messages::Payload::new(&[
                crate::messages::PayloadKind::KeyValue {
                    key: "dropped".to_string(),
                    value: id,
                },
            ]),
        })
    }

    pub async fn consume(game: &mut crate::game::GameState, player: &String, item: &String) -> crate::messages::Message {
        let id: String;
        if let Some(player) = game.players.get_mut(player) {
            if let Some(i) = player.items.iter().position(|i| *i == *item || game.items[i].name == *item) {
                if let crate::game::ItemKind::Consumable { heal } = game.items[item].data {
                    id = player.items.remove(i);
                    player.status.hp = std::cmp::min(player.status.hp + heal, player.status.max_hp);
                } else {
                    return crate::messages::Message::Error(crate::messages::Error::ItemNotConsumable);
                }
            } else {
                return crate::messages::Message::Error(crate::messages::Error::ItemNotInInventory);
            }
        } else {
            return crate::messages::Message::Error(crate::messages::Error::ServerError)
        }
        let mut combat_update = false;
        let room = game.rooms[&game.players[player].room].room.id.clone();
        if let Some(room) = game.rooms.get_mut(&room) && let Some(i) = room.combat.index(player) {
            combat_update = true;
            room.combat.players[i] = game.players[player].status.clone();
        }
        if combat_update {
            crate::cli::Logger::event(
                &room,
                game,
                &crate::messages::Event {
                    scope: crate::messages::EventScope::Room,
                    kind: crate::messages::EventKind::CombatStats,
                    payload: crate::messages::Payload::new(&[
                        crate::messages::PayloadKind::new_json(&game.rooms[&room].combat),
                    ]),
                },
                |to| to.room == room,
            ).await;
        }
        Self::update_quests(game, player, "item", &id).await;
        crate::messages::Message::Response(crate::messages::Response {
            payload: crate::messages::Payload::new(&[
                crate::messages::PayloadKind::KeyValue {
                    key: "consumed".to_string(),
                    value: id,
                },
            ]),
        })
    }

    pub async fn equip(game: &mut crate::game::GameState, player: &String, item: &String) -> crate::messages::Message {
        let id: String;
        let mut removed = String::new();
        if let Some(player) = game.players.get_mut(player) {
            if let Some(i) = player.items.iter().position(|i| *i == *item || game.items[i].name == *item) {
                if let crate::game::ItemKind::Armor { armor: _ } = game.items[item].data {
                    id = player.items.remove(i);
                    if !player.status.armor.is_empty() {
                        removed = player.status.armor.clone();
                        player.items.push(player.status.armor.clone());
                    }
                    player.status.armor = id.clone();
                } else if let crate::game::ItemKind::Weapon { damage: _ } = game.items[item].data {
                    id = player.items.remove(i);
                    if !player.status.weapon.is_empty() {
                        removed = player.status.weapon.clone();
                        player.items.push(player.status.weapon.clone());
                    }
                    player.status.weapon = id.clone();
                } else {
                    return crate::messages::Message::Error(crate::messages::Error::ItemNotEquipable);
                }
            } else {
                return crate::messages::Message::Error(crate::messages::Error::ItemNotInInventory);
            }
        } else {
            return crate::messages::Message::Error(crate::messages::Error::ServerError)
        }
        let mut combat_update = false;
        let room = game.rooms[&game.players[player].room].room.id.clone();
        if let Some(room) = game.rooms.get_mut(&room) && let Some(i) = room.combat.index(player) {
            combat_update = true;
            room.combat.players[i] = game.players[player].status.clone();
        }
        if combat_update {
            crate::cli::Logger::event(
                &room,
                game,
                &crate::messages::Event {
                    scope: crate::messages::EventScope::Room,
                    kind: crate::messages::EventKind::CombatStats,
                    payload: crate::messages::Payload::new(&[
                        crate::messages::PayloadKind::new_json(&game.rooms[&room].combat),
                    ]),
                },
                |to| to.room == room,
            ).await;
        }
        Self::update_quests(game, player, "item", &id).await;
        if !removed.is_empty() {
            Self::update_quests(game, player, "item", &removed).await;
        }
        crate::messages::Message::Response(crate::messages::Response {
            payload: crate::messages::Payload::new(&[
                crate::messages::PayloadKind::KeyValue {
                    key: "equiped".to_string(),
                    value: id,
                },
            ]),
        })
    }

    pub async fn unequip(game: &mut crate::game::GameState, player: &String, item: &String) -> crate::messages::Message {
        let id: String;
        if let Some(player) = game.players.get_mut(player) {
            if !player.status.armor.is_empty() && (*item == player.status.armor || *item == game.items[&player.status.armor].name) {
                id = player.status.armor.clone();
                player.items.push(player.status.armor.clone());
                player.status.armor.clear();
            } else if !player.status.weapon.is_empty() && (*item == player.status.weapon || *item == game.items[&player.status.weapon].name) {
                id = player.status.weapon.clone();
                player.items.push(player.status.weapon.clone());
                player.status.weapon.clear();
            } else {
                return crate::messages::Message::Error(crate::messages::Error::ItemNotEquipable);
            }
        } else {
            return crate::messages::Message::Error(crate::messages::Error::ServerError);
        }
        let mut combat_update = false;
        let room = game.rooms[&game.players[player].room].room.id.clone();
        if let Some(room) = game.rooms.get_mut(&room) && let Some(i) = room.combat.index(player) {
            combat_update = true;
            room.combat.players[i] = game.players[player].status.clone();
        }
        if combat_update {
            crate::cli::Logger::event(
                &room,
                game,
                &crate::messages::Event {
                    scope: crate::messages::EventScope::Room,
                    kind: crate::messages::EventKind::CombatStats,
                    payload: crate::messages::Payload::new(&[
                        crate::messages::PayloadKind::new_json(&game.rooms[&room].combat),
                    ]),
                },
                |to| to.room == room,
            ).await;
        }
        Self::update_quests(game, player, "item", &id).await;
        crate::messages::Message::Response(crate::messages::Response {
            payload: crate::messages::Payload::new(&[
                crate::messages::PayloadKind::KeyValue {
                    key: "unequiped".to_string(),
                    value: id,
                },
            ]),
        })
    }

    pub async fn attack(game: &mut crate::game::GameState, player: &String, npc: &String) -> crate::messages::Message {
        let name: String;
        let mut killed = false;
        if let Some(room) = game.rooms.get_mut(&game.players[player].room) {
            name = room.room.id.clone();
            if room.npcs.contains(npc) {
                if game.npcs[npc].is_enemy() {
                    if room.combat.index(player).is_some() {
                        for i in 0..room.combat.enemies.len() {
                            if room.combat.enemies[i].id == *npc && let Some(player) = game.players.get_mut(player) {
                                room.combat.enemies[i].hp = room.combat.enemies[i].hp.saturating_sub(
                                    if !player.status.weapon.is_empty() && let crate::game::ItemKind::Weapon { damage } = &game.items[&player.status.weapon].data {
                                        *damage
                                    } else {
                                        10
                                    }.saturating_sub(room.combat.enemies[i].armor)
                                );
                                if room.combat.enemies[i].hp == 0 {
                                    killed = true;
                                    room.combat.enemies.remove(i);
                                    room.dead_npcs.push(room.npcs.remove(room.npcs.iter().position(|i| i == npc).unwrap()));
                                    break;
                                }
                                player.status.hp = player.status.hp.saturating_sub(
                                    room.combat.enemies[i].attack.saturating_sub(
                                        if !player.status.armor.is_empty() && let crate::game::ItemKind::Armor { armor } = &game.items[&player.status.armor].data {
                                            *armor
                                        } else {
                                            0
                                        }
                                    )
                                );
                                if let Some(room) = game.rooms.get_mut(&player.room) && let Some(i) = room.combat.index(&player.username) {
                                    room.combat.players[i] = player.status.clone();
                                }
                                break;
                            }
                        }
                    } else if room.combat.players.is_empty() {
                        for npc in &room.npcs {
                            if let crate::game::NpcKind::Enemy { hp, attack, armor } = &game.npcs[npc].data {
                                room.combat.enemies.push(crate::game::EnemyStatus {
                                    id: npc.clone(),
                                    hp: *hp,
                                    max_hp: *hp,
                                    armor: *armor,
                                    attack: *attack,
                                });
                            }
                        }
                        room.combat.players.push(game.players[player].status.clone());
                    } else if game.players[player].group.is_empty() || game.players[player].group != game.players[&room.combat.players[0].username].group {
                        return crate::messages::Message::Error(crate::messages::Error::NotInGroup);
                    } else {
                        room.combat.players.push(game.players[player].status.clone());
                    }
                } else {
                    return crate::messages::Message::Error(crate::messages::Error::NPCNotHostile);
                }
            } else {
                return crate::messages::Message::Error(crate::messages::Error::NPCNotFound);
            }
        } else {
            return crate::messages::Message::Error(crate::messages::Error::ServerError);
        }
        if killed {
            for player in &game.rooms[&name].players.clone() {
                Self::update_quests(game, player, "kill", npc).await;
            }
        }
        if game.players[player].status.hp == 0 {
            crate::game::RoomState::leave(game, player).await;
            crate::game::RoomState::enter(game, player, &"room.start".to_string()).await;
            Self::update_quests(game, player, "room", "room.start").await;
            if let Some(player) = game.players.get_mut(player) {
                if let Some(room) = game.rooms.get_mut(&name) {
                    if !player.status.armor.is_empty() {
                        player.items.push(player.status.armor.clone());
                        player.status.armor.clear();
                    }
                    if !player.status.weapon.is_empty() {
                        player.items.push(player.status.weapon.clone());
                        player.status.weapon.clear();
                    }
                    room.items.append(&mut player.items);
                }
                player.status.hp = std::cmp::max(1, player.status.max_hp / 2);
            }
            crate::cli::Logger::event(
                player,
                game,
                &crate::messages::Event {
                    scope: crate::messages::EventScope::Player,
                    kind: crate::messages::EventKind::Die,
                    payload: crate::messages::Payload::default(),
                },
                |to| to.username == *player,
            ).await;
        }
        if game.rooms[&name].combat.enemies.is_empty() {
            if let Some(room) = game.rooms.get_mut(&name) {
                room.combat = crate::game::Combat::default();
            }
            crate::cli::Logger::event(
                &name,
                game,
                &crate::messages::Event {
                    scope: crate::messages::EventScope::Room,
                    kind: crate::messages::EventKind::CombatEnd,
                    payload: crate::messages::Payload::default(),
                },
                |to| to.room == name,
            ).await;
        } else {
            crate::cli::Logger::event(
                &name,
                game,
                &crate::messages::Event {
                    scope: crate::messages::EventScope::Room,
                    kind: crate::messages::EventKind::CombatStats,
                    payload: crate::messages::Payload::new(&[
                        crate::messages::PayloadKind::new_json(&game.rooms[&name].combat),
                    ]),
                },
                |to| to.room == name,
            ).await;
        }
        crate::messages::Message::Response(crate::messages::Response::default())
    }

    pub fn abandon_quest(game: &mut crate::game::GameState, player: &String, quest: &String) -> crate::messages::Message {
        if let Some(player) = game.players.get_mut(player) {
            if let Some(quest) = player.quests.get_mut(quest) {
                if matches!(quest.status, crate::game::QuestStatus::Active) {
                    quest.status = crate::game::QuestStatus::Abandoned;
                    crate::messages::Message::Response(crate::messages::Response::default())
                } else {
                    crate::messages::Message::Error(crate::messages::Error::QuestNotActive)
                }
            } else {
                crate::messages::Message::Error(crate::messages::Error::QuestNotFound)
            }
        } else {
            crate::messages::Message::Error(crate::messages::Error::ServerError)
        }
    }

    pub async fn update_quests(game: &mut crate::game::GameState, player: &String, kind: &str, value: &str) {
        let mut completed = Vec::new();
        if let Some(player) = game.players.get_mut(player) {
            for (id, quest) in player.quests.iter_mut() {
                if matches!(quest.status, crate::game::QuestStatus::Active) {
                    let reference = &game.quests[id];
                    match &reference.task {
                        crate::game::QuestKind::Bring {
                            item,
                            count,
                        } if kind.is_empty() || kind == "item" && item == value => quest.progress = std::cmp::min(*count, player.items
                            .iter()
                            .filter(|s| **s == *item)
                            .count() as u32),
                        crate::game::QuestKind::Goto {
                            room
                        } if kind.is_empty() || kind == "room" && room == value => quest.progress = (player.room == *room) as u32,
                        crate::game::QuestKind::Kill {
                            enemy,
                            count,
                        } if kind == "kill" && enemy == value => quest.progress = std::cmp::min(*count, quest.progress + 1),
                        crate::game::QuestKind::Talk {
                            npc
                        } if kind == "talk" && npc == value => quest.progress = 1,
                        _ => (),
                    }
                    if reference.autocomplete && quest.is_complete(&game.quests) {
                        if let crate::game::QuestKind::Bring {
                            item,
                            count,
                        } = &reference.task {
                            for _ in 0..*count {
                                player.items.remove(player.items.iter().position(|i| i == item).unwrap());
                            }
                        }
                        player.items.append(&mut reference.reward.clone());
                        quest.status = crate::game::QuestStatus::Completed;
                        completed.push(id.clone());
                    }
                }
            }
        }
        for quest in completed {
            crate::cli::Logger::event(
                player,
                game,
                &crate::messages::Event {
                    scope: crate::messages::EventScope::Player,
                    kind: crate::messages::EventKind::QuestComplete,
                    payload: crate::messages::Payload::new(&[
                        crate::messages::PayloadKind::String(quest),
                    ]),
                },
                |to| to.username == *player,
            ).await;
        }
    }
}

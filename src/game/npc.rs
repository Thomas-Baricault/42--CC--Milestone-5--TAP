use rand::seq::IndexedRandom;

#[derive(
    Clone,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum NpcKind {
    Enemy {
        hp: u32,
        attack: u32,
        armor: u32,
    },
    Neutral {
        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        dialogues: Vec<String>,

        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        quests: Vec<String>,
    },
}

impl NpcKind {
    pub fn is_enemy(&self) -> bool {
        matches!(self, Self::Enemy { hp: _, attack: _, armor: _ })
    }
}

#[derive(
    Clone,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
pub struct Npc {
    pub id: String,
    pub name: String,
    pub description: String,
    pub data: NpcKind,
}

impl Npc {
    pub fn is_enemy(&self) -> bool {
        self.data.is_enemy()
    }

    pub async fn talk(game: &mut crate::game::GameState, player: &String, npc: &String) -> crate::messages::Message {
        let mut dialogue = "...".to_string();
        if let Some(npc) = game.npcs.get(npc) {
            if let NpcKind::Neutral {
                dialogues,
                quests,
            } = &npc.data {
                let mut event = None;
                if let Some(player) = game.players.get_mut(player) {
                    for quest in quests {
                        if let Some(quest) = player.quests.get_mut(quest) && matches!(quest.status, crate::game::QuestStatus::Active) && quest.is_complete(&game.quests) {
                            let reference = &game.quests[&quest.quest];
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
                            if !reference.thanks.is_empty() {
                                dialogue = reference.thanks.clone();
                            }
                            event = Some(crate::messages::Event {
                                scope: crate::messages::EventScope::Player,
                                kind: crate::messages::EventKind::QuestComplete,
                                payload: crate::messages::Payload::new(&[
                                    crate::messages::PayloadKind::String(quest.quest.clone()),
                                ]),
                            });
                            break;
                        }
                    }
                }
                if let Some(event) = event {
                    crate::cli::Logger::event(
                        player,
                        game,
                        &event,
                        |to| to.username == *player,
                    ).await;
                } else {
                    let mut rng = rand::rng();
                    if let Some(choosed) = dialogues.choose(&mut rng) {
                        dialogue = choosed.clone();
                    }
                }
            } else {
                return crate::messages::Message::Error(crate::messages::Error::NPCNotNeutral);
            }
        } else {
            return crate::messages::Message::Error(crate::messages::Error::NPCNotFound);
        }
        crate::game::Player::update_quests(game, player, "talk", npc).await;
        crate::messages::Message::Response(crate::messages::Response {
            payload: crate::messages::Payload::new(&[
                crate::messages::PayloadKind::String(dialogue)
            ]),
        })
    }

    pub async fn quest(game: &mut crate::game::GameState, player: &String, npc: &String) -> crate::messages::Message {
        let mut new_quest = String::new();
        if let Some(npc) = game.npcs.get(npc) {
            if let NpcKind::Neutral {
                dialogues: _,
                quests,
            } = &npc.data {
                if let Some(player) = game.players.get_mut(player) {
                    let mut ok = false;
                    for quest in quests {
                        ok = true;
                        if let Some(quest) = player.quests.get_mut(quest) {
                            if matches!(quest.status, crate::game::QuestStatus::Abandoned) {
                                quest.status = crate::game::QuestStatus::Active;
                            } else {
                                ok = false;
                            }
                        } else {
                            let quest = &game.quests[quest];
                            for require in &quest.requirements {
                                if !player.quests.contains_key(require) || !matches!(player.quests[require].status, crate::game::QuestStatus::Completed) {
                                    ok = false;
                                    break;
                                }
                            }
                            if ok {
                                player.quests.insert(
                                    quest.id.clone(),
                                    crate::game::QuestProgress::new(
                                        npc.id.clone(),
                                        quest,
                                    ),
                                );
                            }
                        }
                        if ok {
                            new_quest = quest.clone();
                            break;
                        }
                    }
                    if !ok {
                        return crate::messages::Message::Error(crate::messages::Error::NoQuestAvailable);
                    }
                } else {
                    return crate::messages::Message::Error(crate::messages::Error::ServerError);
                }
            } else {
                return crate::messages::Message::Error(crate::messages::Error::NPCNotNeutral);
            }
        } else {
            return crate::messages::Message::Error(crate::messages::Error::NPCNotFound);
        }
        crate::game::Player::update_quests(game, player, "", "").await;
        crate::messages::Message::Response(crate::messages::Response {
            payload: crate::messages::Payload::new(&[
                crate::messages::PayloadKind::new_json(&game.quests[&new_quest]),
            ]),
        })
    }
}

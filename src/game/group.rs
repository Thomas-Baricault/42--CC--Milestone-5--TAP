#[derive(
    Default,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct Group {
    pub name: String,
    pub players: std::collections::HashSet<String>,
    pub invited: std::collections::HashSet<String>,
}

impl Group {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            players: std::collections::HashSet::new(),
            invited: std::collections::HashSet::new(),
        }
    }

    pub fn create(game: &mut crate::game::GameState, player: &String, name: &String) -> crate::messages::Message {
        if game.groups.contains_key(name) {
            return crate::messages::Message::Error(crate::messages::Error::NameInUse);
        } else if name.is_empty() || name.len() > 256 {
			return crate::messages::Message::Error(crate::messages::Error::InvalidName);
		}
        if let Some(player) = game.players.get_mut(player) {
            if !player.group.is_empty() {
                return crate::messages::Message::Error(crate::messages::Error::AlreadyInGroup);
            }
            player.group = name.clone();
        }
        let mut group = Self::new(name);
        group.players.insert(player.clone());
        game.groups.insert(group.name.clone(), group);
        crate::messages::Message::Response(crate::messages::Response {
            payload: crate::messages::Payload::new(&[
                crate::messages::PayloadKind::KeyValue {
                    key: "group".to_string(),
                    value: name.clone(),
                },
            ]),
        })
    }

    pub async fn invite(game: &mut crate::game::GameState, player: &String, invited: &String) -> crate::messages::Message {
        if player == invited {
            return crate::messages::Message::Error(crate::messages::Error::PlayerNotFound);
        }
        if !game.players.contains_key(invited) {
            return crate::messages::Message::Error(crate::messages::Error::PlayerNotFound);
        }
        let group = game.players[player].group.clone();
        if group.is_empty() {
            return crate::messages::Message::Error(crate::messages::Error::NotInGroup);
        }
        if let Some(group) = game.groups.get_mut(&group) {
            group.invited.insert(invited.clone());
        }
        crate::cli::Logger::event(
            invited,
            game,
            &crate::messages::Event {
                scope: crate::messages::EventScope::Group,
                kind: crate::messages::EventKind::Invite,
                payload: crate::messages::Payload::new(&[
                    crate::messages::PayloadKind::String(group),
                ]),
            },
            |to| to.username == *invited,
        ).await;
        crate::messages::Message::Response(crate::messages::Response::default())
    }

    pub async fn join(game: &mut crate::game::GameState, player: &String, group: &String) -> crate::messages::Message {
        if let Some(group) = game.groups.get_mut(group) {
            if !group.invited.contains(player) {
                return crate::messages::Message::Error(crate::messages::Error::NotInvited);
            }
            group.players.insert(player.clone());
            group.invited.remove(player);
        } else {
            return crate::messages::Message::Error(crate::messages::Error::GroupNotFound);
        }
        Self::leave(game, player).await;
        if let Some(player) = game.players.get_mut(player) {
            player.group = group.clone();
        }
        crate::cli::Logger::event(
            group,
            game,
            &crate::messages::Event {
                scope: crate::messages::EventScope::Group,
                kind: crate::messages::EventKind::Join,
                payload: crate::messages::Payload::new(&[
                    crate::messages::PayloadKind::String(player.clone()),
                ]),
            },
            |to| to.username != *player && to.group == *group,
        ).await;
        crate::messages::Message::Response(crate::messages::Response {
            payload: crate::messages::Payload::new(&[
                crate::messages::PayloadKind::KeyValue {
                    key: "group".to_string(),
                    value: group.clone(),
                },
            ]),
        })
    }

    pub async fn leave(game: &mut crate::game::GameState, player: &String) -> crate::messages::Message {
        let group: String;
        if let Some(player) = game.players.get_mut(player) {
            if player.group.is_empty() {
                return crate::messages::Message::Error(crate::messages::Error::NotInGroup);
            }
            group = player.group.clone();
            player.group.clear();
        } else {
            return crate::messages::Message::Error(crate::messages::Error::ServerError);
        }
        if game.groups[&group].players.len() == 1 {
            game.groups.remove(&group);
        } else if let Some(group) = game.groups.get_mut(&group) {
            group.players.remove(player);
        }
        crate::cli::Logger::event(
            &group,
            game,
            &crate::messages::Event {
                scope: crate::messages::EventScope::Group,
                kind: crate::messages::EventKind::Leave,
                payload: crate::messages::Payload::new(&[
                    crate::messages::PayloadKind::String(player.clone()),
                ]),
            },
            |to| to.group == group,
        ).await;
        crate::messages::Message::Response(crate::messages::Response::default())
    }
}

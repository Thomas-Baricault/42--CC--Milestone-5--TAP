use std::str::FromStr;

use crate::cli::HandleEvent;
use crate::tui::Widget;

enum Page {
    Welcome = 0,
    Room = 1,
    Stats = 2,
    Quests = 3,
    Group = 4,
    Chat = 5,
}

#[derive(Default)]
enum Focus {
    #[default]
    WelcomeExit,

    RoomReload,
    RoomMove,
    RoomFighters,
    RoomEnemies,
    RoomPlayers,
    RoomNPCs,
    RoomItems,
    StatsArmorUnequip,
    StatsWeaponUnequip,
    StatsInventory,
    QuestsQuests,
    GroupCreate,
    GroupPlayers,
    GroupLeave,
    GroupInvitations,
    Chat,
}

#[derive(Default)]
pub struct FriendlyCli {
    waiter: crate::utils::Waiter,
    client: crate::network::Client,
    commands: Vec<crate::messages::Command>,
    focused: Focus,
    popup: Option<crate::tui::Popup>,
    notebook: crate::tui::Notebook,
    knowledge: crate::tui::Knowledge,
}

impl FriendlyCli {
    pub async fn start(&mut self, client: crate::network::Client) -> Option<crate::messages::Error> {
        self.waiter.begin(3);
        self.client = client;
        let mut terminal = match crate::tui::Terminal::new() {
            Ok(v) => v,
            Err(_) => return Some(crate::messages::Error::UnknownError),
        };
        self.notebook = crate::tui::Notebook::new(vec![
            Box::new(crate::tui::WelcomePage::default()),
            Box::new(crate::tui::RoomPage::default()),
            Box::new(crate::tui::StatsPage::default()),
            Box::new(crate::tui::QuestsPage::default()),
            Box::new(crate::tui::GroupPage::default()),
            Box::new(crate::tui::ChatPage::default()),
        ]);
        self.change_focus(Focus::WelcomeExit);
        crate::cli::Handler::init();
        let r = self.run(&mut terminal).await;
        crate::cli::Handler::cleanup();
        let _ = terminal.close();
        r
    }

    async fn run(&mut self, terminal: &mut crate::tui::Terminal) -> Option<crate::messages::Error> {
        loop {
            if !self.client.is_open() {
                return None;
            }
            terminal.update(&mut *self, Self::render);
            if let Some(writer) = &self.client.writer {
                while let Some(id) = self.knowledge.need() {
                    self.commands.push(crate::messages::Command {
                        kind: crate::messages::CommandKind::Describe,
                        payload: crate::messages::Payload::new(&[
                            crate::messages::PayloadKind::String(id),
                        ]),
                    });
                }
                if !self.waiter.is_waiting() && !self.commands.is_empty() {
                    if writer.write_message(&crate::messages::Message::Command(self.commands[0].clone())).await.is_err() {
                        return Some(crate::messages::Error::SendFailed);
                    }
                    self.waiter.begin(3);
                }
            }
            match &self.popup {
                Some(crate::tui::Popup::Input(popup)) if popup.id == "auth" && matches!(self.client.state, crate::network::ClientState::Authenticated) => self.popup = None,
                None if !matches!(self.client.state, crate::network::ClientState::Authenticated) => self.popup = Some(crate::tui::Popup::input("auth", "Enter your username")),
                _ => (),
            }
            if let Some(e) = tokio::select! {
                _ = self.waiter.wait() => Some(crate::messages::Error::ServerTimeOut),
                event = async {
                    if !self.commands.is_empty() {
                        crate::utils::Waiter::block().await;
                    }
                    if let Some(popup) = &mut self.popup {
                        popup.handle_event().await
                    } else {
                        match &self.focused {
                            Focus::WelcomeExit => self.notebook.page::<crate::tui::WelcomePage>(Page::Welcome as usize).exit.handle_event().await,
                            Focus::RoomReload => self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).refresh.handle_event().await,
                            Focus::RoomMove => self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).move_to.handle_event().await,
                            Focus::RoomFighters => self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).fighters.handle_event().await,
                            Focus::RoomEnemies => self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).enemies.handle_event().await,
                            Focus::RoomPlayers => self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).players.handle_event().await,
                            Focus::RoomNPCs => self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).npcs.handle_event().await,
                            Focus::RoomItems => self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).items.handle_event().await,
                            Focus::StatsArmorUnequip => self.notebook.page::<crate::tui::StatsPage>(Page::Stats as usize).armor_unequip.handle_event().await,
                            Focus::StatsWeaponUnequip => self.notebook.page::<crate::tui::StatsPage>(Page::Stats as usize).weapon_unequip.handle_event().await,
                            Focus::StatsInventory => self.notebook.page::<crate::tui::StatsPage>(Page::Stats as usize).inventory.handle_event().await,
                            Focus::QuestsQuests => self.notebook.page::<crate::tui::QuestsPage>(Page::Quests as usize).quests.handle_event().await,
                            Focus::GroupCreate => self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).create.handle_event().await,
                            Focus::GroupPlayers => self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).players.handle_event().await,
                            Focus::GroupLeave => self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).leave.handle_event().await,
                            Focus::GroupInvitations => self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).invitations.handle_event().await,
                            Focus::Chat => self.notebook.page::<crate::tui::ChatPage>(Page::Chat as usize).chat.handle_event().await,
                        }
                    }
                } => {
                    if let Some(event) = event {
                        if matches!(event, crate::cli::Event::Interrupted) {
                            return None;
                        }
                        self.process_input(event);
                    }
                    None
                }
                message = self.client.reader.read() => {
                    match message {
                        Ok(Some(message)) => match crate::messages::Message::from_str(&message) {
                            Ok(message) => {
                                match message {
                                    crate::messages::Message::Error(error) => {
                                        self.waiter.end();
                                        self.commands.remove(0);
                                        Some(error)
                                    },
                                    crate::messages::Message::Response(response) => self.process_response(response).await,
                                    crate::messages::Message::Event(event) => self.process_event(event).await,
                                    crate::messages::Message::Command(_) => Some(crate::messages::Error::UnexpectedServerResponse),
                                }
                            }
                            Err(_) => Some(crate::messages::Error::UnexpectedServerResponse),
                        }
                        _ => Some(crate::messages::Error::ConnectionClosed),
                    }
                }
            } {
                if e.is_fatal() {
                    return Some(e);
                } else {
                    self.popup = Some(crate::tui::Popup::error(e));
                }
            }
        }
    }

    fn focus_info(&mut self) -> (Focus, Focus, &mut bool) {
        let in_combat = self.knowledge.room.combat.index(&self.knowledge.player.username).is_some();
        let in_group = !self.knowledge.player.group.is_empty();
        match &self.focused {
            Focus::WelcomeExit => (Focus::Chat, Focus::RoomReload, &mut self.notebook.page::<crate::tui::WelcomePage>(Page::Welcome as usize).exit.focus),
            Focus::RoomReload => (Focus::WelcomeExit, if in_combat { Focus::RoomFighters } else { Focus::RoomMove }, &mut self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).refresh.focus),
            Focus::RoomMove => (Focus::RoomReload, Focus::RoomPlayers, &mut self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).move_to.focus),
            Focus::RoomFighters => (Focus::RoomReload, Focus::RoomEnemies, &mut self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).fighters.focus),
            Focus::RoomEnemies => (Focus::RoomFighters, Focus::RoomPlayers, &mut self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).enemies.focus),
            Focus::RoomPlayers => (if in_combat { Focus::RoomEnemies } else { Focus::RoomMove }, Focus::RoomNPCs, &mut self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).players.focus),
            Focus::RoomNPCs => (Focus::RoomPlayers, Focus::RoomItems, &mut self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).npcs.focus),
            Focus::RoomItems => (Focus::RoomNPCs, Focus::StatsArmorUnequip, &mut self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).items.focus),
            Focus::StatsArmorUnequip => (Focus::RoomItems, Focus::StatsWeaponUnequip, &mut self.notebook.page::<crate::tui::StatsPage>(Page::Stats as usize).armor_unequip.focus),
            Focus::StatsWeaponUnequip => (Focus::StatsArmorUnequip, Focus::StatsInventory, &mut self.notebook.page::<crate::tui::StatsPage>(Page::Stats as usize).weapon_unequip.focus),
            Focus::StatsInventory => (Focus::StatsWeaponUnequip, Focus::QuestsQuests, &mut self.notebook.page::<crate::tui::StatsPage>(Page::Stats as usize).inventory.focus),
            Focus::QuestsQuests => (Focus::StatsInventory, if in_group { Focus::GroupPlayers } else { Focus::GroupCreate }, &mut self.notebook.page::<crate::tui::QuestsPage>(Page::Quests as usize).quests.focus),
            Focus::GroupCreate => (Focus::QuestsQuests, Focus::GroupInvitations, &mut self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).create.focus),
            Focus::GroupPlayers => (Focus::QuestsQuests, Focus::GroupLeave, &mut self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).players.focus),
            Focus::GroupLeave => (Focus::GroupPlayers, Focus::GroupInvitations, &mut self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).leave.focus),
            Focus::GroupInvitations => (if in_group { Focus::GroupLeave } else { Focus::GroupCreate }, Focus::Chat, &mut self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).invitations.focus),
            Focus::Chat => (Focus::GroupInvitations, Focus::WelcomeExit, &mut self.notebook.page::<crate::tui::ChatPage>(Page::Chat as usize).chat.focus),
        }
    }

    fn change_focus(&mut self, focus: Focus) {
        *self.focus_info().2 = false;
        self.focused = focus;
        *self.focus_info().2 = true;
        self.notebook.current = match &self.focused {
            Focus::WelcomeExit => Page::Welcome,
            Focus::RoomReload | Focus::RoomMove | Focus::RoomFighters | Focus::RoomEnemies | Focus::RoomPlayers | Focus::RoomNPCs | Focus::RoomItems  => Page::Room,
            Focus::StatsArmorUnequip | Focus::StatsWeaponUnequip | Focus::StatsInventory => Page::Stats,
            Focus::QuestsQuests => Page::Quests,
            Focus::GroupCreate | Focus::GroupPlayers | Focus::GroupLeave | Focus::GroupInvitations => Page::Group,
            Focus::Chat => Page::Chat,
        } as usize;
    }

    fn process_input(&mut self, event: crate::cli::Event) {
        if self.popup.is_some() {
            if let crate::cli::Event::Validate = event {
                let mut popup = self.popup.take().unwrap();
                match &mut popup {
                    crate::tui::Popup::Ask(popup) if let Some(selected) = popup.selected() && selected => match popup.id {
                        "exit" => self.client.close(),
                        "group_leave" => self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::GroupLeave)),
                        _ => (),
                    }
                    crate::tui::Popup::Action(popup) if let Some(selected) = popup.buttons.selected() => match popup.id {
                        "invite" => match selected.as_str() {
                            "Dismiss" => {
                                self.knowledge.invitations.remove(&popup.title);
                                self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).update(&self.knowledge);
                            }
                            "Accept" => {
                                self.knowledge.invitations.remove(&popup.title);
                                self.commands.push(crate::messages::Command {
                                    kind: crate::messages::CommandKind::GroupJoin,
                                    payload: crate::messages::Payload::new(&[
                                        crate::messages::PayloadKind::String(popup.title.clone()),
                                    ]),
                                });
                            }
                            _ => (),
                        }
                        "move" if let Ok(direction) = crate::game::Direction::from_str(selected) => self.commands.push(crate::messages::Command {
                            kind: crate::messages::CommandKind::Move,
                            payload: crate::messages::Payload::new(&[
                                crate::messages::PayloadKind::String(direction.to_string()),
                            ]),
                        }),
                        "player" if selected == "Invite" => self.commands.push(crate::messages::Command {
                            kind: crate::messages::CommandKind::GroupInvite,
                            payload: crate::messages::Payload::new(&[
                                crate::messages::PayloadKind::String(popup.title.clone()),
                            ]),
                        }),
                        _ => (),
                    }
                    crate::tui::Popup::Describe(popup) if let Some(selected) = popup.buttons.selected() => match popup.id {
                        "enemy_status" if selected == "Attack" =>  self.commands.push(crate::messages::Command {
                            kind: crate::messages::CommandKind::Attack,
                            payload: crate::messages::Payload::new(&[
                                crate::messages::PayloadKind::String(popup.data::<crate::game::EnemyStatus>().id.clone()),
                            ]),
                        }),
                        "item" => match selected.as_str() {
                            "Consume" => self.commands.push(crate::messages::Command {
                                kind: crate::messages::CommandKind::Consume,
                                payload: crate::messages::Payload::new(&[
                                    crate::messages::PayloadKind::String(popup.data::<crate::game::Item>().id.clone()),
                                ]),
                            }),
                            "Drop" => self.commands.push(crate::messages::Command {
                                kind: crate::messages::CommandKind::Drop,
                                payload: crate::messages::Payload::new(&[
                                    crate::messages::PayloadKind::String(popup.data::<crate::game::Item>().id.clone()),
                                ]),
                            }),
                            "Equip" => self.commands.push(crate::messages::Command {
                                kind: crate::messages::CommandKind::Equip,
                                payload: crate::messages::Payload::new(&[
                                    crate::messages::PayloadKind::String(popup.data::<crate::game::Item>().id.clone()),
                                ]),
                            }),
                            "Take" => self.commands.push(crate::messages::Command {
                                kind: crate::messages::CommandKind::Take,
                                payload: crate::messages::Payload::new(&[
                                    crate::messages::PayloadKind::String(popup.data::<crate::game::Item>().id.clone()),
                                ]),
                            }),
                            _ => (),
                        }
                        "npc" => match selected.as_str() {
                            "Attack" => self.commands.push(crate::messages::Command {
                                kind: crate::messages::CommandKind::Attack,
                                payload: crate::messages::Payload::new(&[
                                    crate::messages::PayloadKind::String(popup.data::<crate::game::Npc>().id.clone()),
                                ]),
                            }),
                            "Talk" => self.commands.push(crate::messages::Command {
                                kind: crate::messages::CommandKind::Talk,
                                payload: crate::messages::Payload::new(&[
                                    crate::messages::PayloadKind::String(popup.data::<crate::game::Npc>().id.clone()),
                                ]),
                            }),
                            "Ask for a quest" => self.commands.push(crate::messages::Command {
                                kind: crate::messages::CommandKind::Quest,
                                payload: crate::messages::Payload::new(&[
                                    crate::messages::PayloadKind::String(popup.data::<crate::game::Npc>().id.clone()),
                                ]),
                            }),
                            _ => (),
                        }
                        "quest" if selected == "Abandon" => self.commands.push(crate::messages::Command {
                            kind: crate::messages::CommandKind::AbandonQuest,
                            payload: crate::messages::Payload::new(&[
                                crate::messages::PayloadKind::String(popup.data::<crate::game::Quest>().id.clone()),
                            ]),
                        }),
                        _ => (),
                    }
                    crate::tui::Popup::Input(popup) => match popup.id {
                        "auth" => self.commands.push(crate::messages::Command {
                            kind: crate::messages::CommandKind::Connect,
                            payload: crate::messages::Payload::new(&[
                                crate::messages::PayloadKind::String(popup.input.input.consume()),
                            ]),
                        }),
                        "group_create" => self.commands.push(crate::messages::Command {
                            kind: crate::messages::CommandKind::GroupCreate,
                            payload: crate::messages::Payload::new(&[
                                crate::messages::PayloadKind::String(popup.input.input.consume()),
                            ]),
                        }),
                        _ => (),
                    }
                    _ => (),
                }
            }
        } else {
            match event {
                crate::cli::Event::Key {
                    code: crate::cli::KeyCode::Tab,
                    modifiers: crate::cli::KeyModifiers::NONE,
                } if matches!(self.client.state, crate::network::ClientState::Authenticated) => {
                    let focus = self.focus_info().1;
                    self.change_focus(focus);
                }
                crate::cli::Event::Key {
                    code: crate::cli::KeyCode::BackTab,
                    modifiers: crate::cli::KeyModifiers::SHIFT,
                } if matches!(self.client.state, crate::network::ClientState::Authenticated) => {
                    let focus = self.focus_info().0;
                    self.change_focus(focus);
                }
                crate::cli::Event::Validate => {
                    match &self.focused {
                        Focus::WelcomeExit => {
                            self.popup = Some(crate::tui::Popup::ask(
                                "exit",
                                "Exit",
                                "Do you really want to exit ?",
                            ));
                        }
                        Focus::RoomReload => self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Look)),
                        Focus::RoomMove => {
                            let mut actions = vec!["Return".to_string()];
                            let mut directions = self.knowledge.room.room.exits
                                .keys()
                                .map(|i| i.to_string())
                                .collect();
                            actions.append(&mut directions);
                            self.popup = Some(crate::tui::Popup::action(
                                "move",
                                "Move to",
                                actions,
                            ));
                        }
                        Focus::RoomFighters => {
                            if let Some(i) = self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).fighters.index() {
                                self.popup = Some(crate::tui::Popup::describe(
                                    "player_status",
                                    Box::new(self.knowledge.room.combat.players[i].clone()),
                                    vec![],
                                ));
                            }
                        }
                        Focus::RoomEnemies => {
                            if let Some(i) = self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).enemies.index() {
                                self.popup = Some(crate::tui::Popup::describe(
                                    "enemy_status",
                                    Box::new(self.knowledge.room.combat.enemies[i].clone()),
                                    vec!["Attack".to_string()],
                                ));
                            }
                        }
                        Focus::RoomPlayers => {
                            if let Some(player) = self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).players.selected() {
                                self.popup = Some(crate::tui::Popup::action(
                                    "player",
                                    player,
                                    vec![
                                        "Return".to_string(),
                                        "Invite".to_string(),
                                    ],
                                ));
                            }
                        }
                        Focus::RoomNPCs => {
                            if let Some(npc) = self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).npcs.selected() {
                                self.popup = Some(crate::tui::Popup::describe(
                                    "npc",
                                    Box::new(npc.clone()),
                                    if npc.is_enemy() {
                                        vec!["Attack".to_string()]
                                    } else {
                                        vec![
                                            "Talk".to_string(),
                                            "Ask for a quest".to_string(),
                                        ]
                                    },
                                ));
                            }
                        }
                        Focus::RoomItems => {
                            if let Some(item) = self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).items.selected() {
                                self.popup = Some(crate::tui::Popup::describe(
                                    "item",
                                    Box::new(item.clone()),
                                    vec!["Take".to_string()],
                                ));
                            }
                        }
                        Focus::StatsArmorUnequip => {
                            if !self.knowledge.player.status.armor.is_empty() {
                                self.commands.push(crate::messages::Command {
                                    kind: crate::messages::CommandKind::Unequip,
                                    payload: crate::messages::Payload::new(&[
                                        crate::messages::PayloadKind::String(self.knowledge.player.status.armor.clone()),
                                    ]),
                                });
                            }
                        }
                        Focus::StatsWeaponUnequip => {
                            if !self.knowledge.player.status.weapon.is_empty() {
                                self.commands.push(crate::messages::Command {
                                    kind: crate::messages::CommandKind::Unequip,
                                    payload: crate::messages::Payload::new(&[
                                        crate::messages::PayloadKind::String(self.knowledge.player.status.weapon.clone()),
                                    ]),
                                });
                            }
                        }
                        Focus::StatsInventory => {
                            if let Some(item) = self.notebook.page::<crate::tui::StatsPage>(Page::Stats as usize).inventory.selected() {
                                self.popup = Some(crate::tui::Popup::describe(
                                    "item",
                                    Box::new(item.clone()),
                                    vec![
                                        "Consume".to_string(),
                                        "Equip".to_string(),
                                        "Drop".to_string(),
                                    ],
                                ));
                            }
                        }
                        Focus::QuestsQuests => {
                            if let Some(quest) = self.notebook.page::<crate::tui::QuestsPage>(Page::Quests as usize).quests.selected() {
                                self.popup = Some(crate::tui::Popup::describe(
                                    "quest",
                                    Box::new(quest.clone()),
                                    vec!["Abandon".to_string()],
                                ));
                            }
                        }
                        Focus::GroupCreate => {
                            self.popup = Some(crate::tui::Popup::input(
                                "group_create",
                                "Enter the group name",
                            ));
                        }
                        Focus::GroupPlayers => (),
                        Focus::GroupLeave => {
                            self.popup = Some(crate::tui::Popup::ask(
                                "group_leave",
                                "Leave Group",
                                "Do you really want to leave the group ?",
                            ));
                        }
                        Focus::GroupInvitations => {
                            if let Some(group) = self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).invitations.selected() {
                                self.popup = Some(crate::tui::Popup::action(
                                    "invite",
                                    group,
                                    vec![
                                        "Return".to_string(),
                                        "Accept".to_string(),
                                        "Dismiss".to_string(),
                                    ],
                                ));
                            }
                        }
                        Focus::Chat => {
                            let scope = self.notebook.page::<crate::tui::ChatPage>(Page::Chat as usize).chat.scope.to_string();
                            let input = self.notebook.page::<crate::tui::ChatPage>(Page::Chat as usize).chat.input.input.consume();
                            self.commands.push(crate::messages::Command {
                                kind: crate::messages::CommandKind::Chat,
                                payload: crate::messages::Payload::new(&[
                                    crate::messages::PayloadKind::String(scope),
                                    crate::messages::PayloadKind::String(input),
                                ]),
                            });
                        }
                    }
                }
                _ => (),
            }
        }
    }

    async fn process_response(&mut self, response: crate::messages::Response) -> Option<crate::messages::Error> {
        self.waiter.end();
        match if self.commands.is_empty() {
            None
        } else {
            Some(self.commands.remove(0))
        } {
            Some(command) => match command.kind {
                crate::messages::CommandKind::AbandonQuest => {
                    let mut quest = String::new();
                    if response.payload.is_empty() && command.payload.extract(&mut [
                        crate::messages::PayloadExtractor::String(&mut quest),
                    ]).is_ok() && let Some(quest) = self.knowledge.player.quests.get_mut(&quest) {
                        quest.status = crate::game::QuestStatus::Abandoned;
                        self.notebook.page::<crate::tui::QuestsPage>(Page::Quests as usize).update(&self.knowledge);
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::Attack => None,
                crate::messages::CommandKind::Chat => {
                    if response.payload.is_empty() {
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::Connect => if response.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut "connected".to_string()),
                ]).is_ok() {
                    self.client.state = crate::network::ClientState::Authenticated;
                    self.knowledge.player.username.clear();
                    if command.payload.extract(&mut [
                        crate::messages::PayloadExtractor::String(&mut self.knowledge.player.username),
                    ]).is_err() {
                        return Some(crate::messages::Error::UnexpectedServerResponse);
                    }
                    self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Look));
                    None
                } else {
                    Some(crate::messages::Error::UnexpectedServerResponse)
                }
                crate::messages::CommandKind::Consume => {
                    if let crate::messages::PayloadKind::String(mut s) = command.payload.args[0].clone() {
                        if response.payload.extract(&mut [
                            crate::messages::PayloadExtractor::KeyValue {
                                key: &mut "consumed".to_string(),
                                value: &mut s,
                            },
                        ]).is_ok() {
                            self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Inventory));
                            self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Status));
                            self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Quests));
                            None
                        } else {
                            Some(crate::messages::Error::UnexpectedServerResponse)
                        }
                    } else {
                        None
                    }
                }
                crate::messages::CommandKind::Describe => {
                    let mut data = crate::game::WorldData::default();
                    if response.payload.extract(&mut [
                        crate::messages::PayloadExtractor::Json(&mut data),
                    ]).is_ok() && self.knowledge.update(data).is_ok() {
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::Drop => {
                    if let crate::messages::PayloadKind::String(mut s) = command.payload.args[0].clone() {
                        if response.payload.extract(&mut [
                            crate::messages::PayloadExtractor::KeyValue {
                                key: &mut "dropped".to_string(),
                                value: &mut s,
                            },
                        ]).is_ok() {
                            self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Inventory));
                            self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Quests));
                            self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Look));
                            None
                        } else {
                            Some(crate::messages::Error::UnexpectedServerResponse)
                        }
                    } else {
                        None
                    }
                }
                crate::messages::CommandKind::Equip => {
                    if let crate::messages::PayloadKind::String(mut s) = command.payload.args[0].clone() {
                        if response.payload.extract(&mut [
                            crate::messages::PayloadExtractor::KeyValue {
                                key: &mut "equiped".to_string(),
                                value: &mut s,
                            },
                        ]).is_ok() {
                            self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Inventory));
                            self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Status));
                            self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Quests));
                            None
                        } else {
                            Some(crate::messages::Error::UnexpectedServerResponse)
                        }
                    } else {
                        None
                    }
                }
                crate::messages::CommandKind::GroupDescribe => {
                    let mut group = crate::game::Group::default();
                    if response.payload.extract(&mut [
                        crate::messages::PayloadExtractor::Json(&mut group),
                    ]).is_ok() {
                        self.knowledge.change_group(Some(group));
                        self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).update(&self.knowledge);
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::GroupCreate | crate::messages::CommandKind::GroupJoin => {
                    let mut group = String::new();
                    if response.payload.extract(&mut [
                        crate::messages::PayloadExtractor::KeyValue {
                            key: &mut "group".to_string(),
                            value: &mut group,
                        },
                    ]).is_ok() {
                        self.change_focus(Focus::GroupPlayers);
                        self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::GroupDescribe));
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::GroupInvite => {
                    if response.payload.is_empty() {
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::GroupLeave => {
                    if response.payload.is_empty() {
                        self.knowledge.change_group(None);
                        self.change_focus(Focus::GroupCreate);
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::Inventory => {
                    if response.payload.extract(&mut [
                        crate::messages::PayloadExtractor::Json(&mut self.knowledge.player.items),
                    ]).is_ok() {
                        self.notebook.page::<crate::tui::StatsPage>(Page::Stats as usize).update(&self.knowledge);
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::Look => {
                    let mut room = crate::game::RoomState::default();
                    if response.payload.extract(&mut [
                        crate::messages::PayloadExtractor::Json(&mut room),
                    ]).is_ok() {
                        self.knowledge.change_room(room);
                        self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).update(&self.knowledge);
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::Move => {
                    let mut room = String::new();
                    if response.payload.extract(&mut [
                        crate::messages::PayloadExtractor::KeyValue {
                            key: &mut "room".to_string(),
                            value: &mut room,
                        },
                    ]).is_ok() {
                        self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Look));
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::Quest => {
                    let mut quest = crate::game::Quest::default();
                    if response.payload.extract(&mut [
                        crate::messages::PayloadExtractor::Json(&mut quest),
                    ]).is_ok() {
                        self.popup = Some(crate::tui::Popup::info("New quest", "New quest obtained!"));
                        self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Quests));
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::Quests => {
                    let mut quests: Vec<crate::game::QuestProgress> = Vec::new();
                    if response.payload.extract(&mut [
                        crate::messages::PayloadExtractor::Json(&mut quests),
                    ]).is_ok() {
                        self.knowledge.player.quests.clear();
                        for quest in quests.into_iter() {
                            self.knowledge.player.quests.insert(quest.quest.clone(), quest);
                        }
                        self.notebook.page::<crate::tui::QuestsPage>(Page::Quests as usize).update(&self.knowledge);
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::Quit => if response.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut "bye".to_string()),
                ]).is_ok() {
                    self.client.close();
                    None
                } else {
                    Some(crate::messages::Error::UnexpectedServerResponse)
                }
                crate::messages::CommandKind::Status => {
                    let mut status = crate::game::PlayerStatus::default();
                    if response.payload.extract(&mut [
                        crate::messages::PayloadExtractor::Json(&mut status),
                    ]).is_ok() {
                        self.knowledge.player.status = status;
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                },
                crate::messages::CommandKind::Take => {
                    if response.payload.extract(&mut [
                        crate::messages::PayloadExtractor::KeyValue {
                            key: &mut "taken".to_string(),
                            value: &mut "".to_string(),
                        },
                    ]).is_ok() {
                        self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Look));
                        self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Quests));
                        self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Inventory));
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::Talk => {
                    let mut dialogue = String::new();
                    if response.payload.extract(&mut [
                        crate::messages::PayloadExtractor::String(&mut dialogue),
                    ]).is_ok() {
                        if let crate::messages::PayloadKind::String(id) = &command.payload.args[0] && let Some(npc) = self.knowledge.npcs.get(id) {
                            self.popup = Some(crate::tui::Popup::info(
                                &npc.name,
                                &dialogue,
                            ));
                        }
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
                crate::messages::CommandKind::Unequip => {
                    if let crate::messages::PayloadKind::String(mut s) = command.payload.args[0].clone() {
                        if response.payload.extract(&mut [
                            crate::messages::PayloadExtractor::KeyValue {
                                key: &mut "unequiped".to_string(),
                                value: &mut s,
                            },
                        ]).is_ok() {
                            self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Inventory));
                            self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Status));
                            self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Quests));
                            None
                        } else {
                            Some(crate::messages::Error::UnexpectedServerResponse)
                        }
                    } else {
                        None
                    }
                }
                crate::messages::CommandKind::Who => {
                    let mut players = String::new();
                    if response.payload.extract(&mut [
                        crate::messages::PayloadExtractor::KeyValue {
                            key: &mut "players".to_string(),
                            value: &mut players,
                        },
                    ]).is_ok() && let Ok(v) = players.parse::<usize>() {
                        self.knowledge.players = v;
                        None
                    } else {
                        Some(crate::messages::Error::UnexpectedServerResponse)
                    }
                }
            }
            None => {
                self.client.proto.clear();
                if response.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut "hello".to_string()),
                    crate::messages::PayloadExtractor::KeyValue {
                        key: &mut "proto".to_string(),
                        value: &mut self.client.proto,
                    }
                ]).is_err() {
                    Some(crate::messages::Error::UnexpectedServerResponse)
                } else {
                    self.knowledge.addr = self.client.addr.clone();
                    self.knowledge.proto = self.client.proto.clone();
                    None
                }
            }
        }
    }

    async fn process_event(&mut self, event: crate::messages::Event) -> Option<crate::messages::Error> {
        match (&event.scope, &event.kind) {
            (crate::messages::EventScope::Global | crate::messages::EventScope::Group | crate::messages::EventScope::Room,
            crate::messages::EventKind::Chat) => {
                let mut message = crate::tui::ChatMessage {
                    scope: event.scope,
                    author: String::new(),
                    content: String::new(),
                };
                if matches!(message.scope, crate::messages::EventScope::Player | crate::messages::EventScope::Stats) || event.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut message.author),
                    crate::messages::PayloadExtractor::String(&mut message.content),
                ]).is_err() {
                    return Some(crate::messages::Error::UnexpectedServerResponse);
                }
                self.notebook.page::<crate::tui::ChatPage>(Page::Chat as usize).chat.push(message);
            }
            (crate::messages::EventScope::Group, crate::messages::EventKind::Invite) => {
                let mut group = String::new();
                if event.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut group),
                ]).is_err() {
                    return Some(crate::messages::Error::UnexpectedServerResponse);
                }
                self.knowledge.invitations.insert(group);
                self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).update(&self.knowledge);
            }
            (crate::messages::EventScope::Group, crate::messages::EventKind::Join) => {
                let mut player = String::new();
                if event.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut player),
                ]).is_err() || self.knowledge.player.group.is_empty() || !self.knowledge.group.players.insert(player) {
                    return Some(crate::messages::Error::UnexpectedServerResponse);
                }
                self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).update(&self.knowledge);
            }
            (crate::messages::EventScope::Group, crate::messages::EventKind::Leave) => {
                let mut player = String::new();
                if event.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut player),
                ]).is_err() || self.knowledge.player.group.is_empty() || !self.knowledge.group.players.remove(&player) {
                    return Some(crate::messages::Error::UnexpectedServerResponse);
                }
                self.notebook.page::<crate::tui::GroupPage>(Page::Group as usize).update(&self.knowledge);
            }
            (crate::messages::EventScope::Stats, crate::messages::EventKind::Players) => {
                let mut n = String::new();
                if event.payload.extract(&mut [
                    crate::messages::PayloadExtractor::KeyValue {
                        key: &mut "players".to_string(),
                        value: &mut n,
                    },
                ]).is_err() {
                    return Some(crate::messages::Error::UnexpectedServerResponse);
                }
                match n.parse::<usize>() {
                    Ok(v) => self.knowledge.players = v,
                    Err(_) => return Some(crate::messages::Error::UnexpectedServerResponse),
                }
            }
            (crate::messages::EventScope::Room, crate::messages::EventKind::PresenceEnter) => {
                let mut player = String::new();
                if event.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut player),
                ]).is_err() || !self.knowledge.room.players.insert(player) {
                    return Some(crate::messages::Error::UnexpectedServerResponse);
                }
                self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).update(&self.knowledge);
            }
            (crate::messages::EventScope::Room, crate::messages::EventKind::PresenceLeave) => {
                let mut player = String::new();
                if event.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut player),
                ]).is_err() || !self.knowledge.room.players.remove(&player) {
                    return Some(crate::messages::Error::UnexpectedServerResponse);
                }
                self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).update(&self.knowledge);
            }
            (crate::messages::EventScope::Room, crate::messages::EventKind::CombatEnd) => {
                if !event.payload.is_empty() {
                    return Some(crate::messages::Error::UnexpectedServerResponse);
                }
                self.focused = Focus::RoomMove;
                self.knowledge.room.combat = crate::game::Combat::default();
                self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).update(&self.knowledge);
                self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Look));
            }
            (crate::messages::EventScope::Room, crate::messages::EventKind::CombatStats) => {
                let mut combat = crate::game::Combat::default();
                if event.payload.extract(&mut [
                    crate::messages::PayloadExtractor::Json(&mut combat),
                ]).is_err() {
                    return Some(crate::messages::Error::UnexpectedServerResponse);
                }
                self.knowledge.room.combat = combat;
                self.notebook.page::<crate::tui::RoomPage>(Page::Room as usize).update(&self.knowledge);
                self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Status));
            }
            (crate::messages::EventScope::Player, crate::messages::EventKind::Die) => {
                if !event.payload.is_empty() {
                    return Some(crate::messages::Error::UnexpectedServerResponse);
                }
                self.knowledge.room.combat = crate::game::Combat::default();
                self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Status));
                self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Inventory));
                self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Quests));
                self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Look));
            }
            (crate::messages::EventScope::Player, crate::messages::EventKind::QuestComplete) => {
                let mut quest = String::new();
                if event.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut quest),
                ]).is_ok() && let Some(quest) = self.knowledge.player.quests.get_mut(&quest) {
                    quest.status = crate::game::QuestStatus::Completed;
                    self.notebook.page::<crate::tui::QuestsPage>(Page::Quests as usize).update(&self.knowledge);
                    self.commands.push(crate::messages::Command::new(crate::messages::CommandKind::Inventory));
                } else {
                    return Some(crate::messages::Error::UnexpectedServerResponse);
                }
            }
            (_, _) => return Some(crate::messages::Error::UnexpectedServerResponse),
        }
        None
    }

    fn render(&mut self, frame: &mut ratatui::Frame) {
        let [header, body] = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Length(3),
            ratatui::layout::Constraint::Fill(1),
        ])
            .areas(frame.area());
        crate::tui::Header.render_with_data(&mut self.knowledge, header, frame.buffer_mut());
        self.notebook.render_with_data(&mut self.knowledge, body, frame.buffer_mut());
        if let Some(popup) = &mut self.popup {
            popup.render_with_data(&mut self.knowledge, body, frame.buffer_mut());
        }
    }
}

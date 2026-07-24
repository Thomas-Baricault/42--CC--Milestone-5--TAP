use eframe::egui;
use egui_extras::{TableBuilder, Column};
use std::fmt;
use std::cmp;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use std::str::FromStr;

pub enum UiAction {
	Command(crate::messages::Command),
}
enum GuiPopup {
	Item { id: String, in_room: bool },
	Npc { id: String },
	Player { name: String },
	Enemy { id: String },
	GroupLeave,
}

#[derive(Default, PartialEq)]
enum AppState {
	#[default]
	AskUsername,
	Connect,
	Rooms,
	Stats,
	Quests,
	Group,
	Chat,
}

impl fmt::Display for AppState{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::AskUsername => write!(f, "Username"),
			Self::Connect => write!(f, "Connect"),
			Self::Rooms => write!(f, "Rooms"),
			Self::Stats => write!(f, "Stats"),
			Self::Quests => write!(f, "Quests"),
			Self::Group => write!(f, "Group"),
			Self::Chat => write!(f, "Chat"),
		}
	}
}

#[derive(Default)]
pub struct MyApp {
	state: AppState,
	username_input: String,
	username: String,

	knowledge: Arc<Mutex<crate::tui::Knowledge>>,
	action_tx: Option<mpsc::UnboundedSender<UiAction>>,
	popup: Option<GuiPopup>,
	selected_quest: Option<String>,

	group_name: String,
	message: String,
	chat_scope: String,

	egui_ctx: egui::Context,
}

impl eframe::App for MyApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {

		self.egui_ctx = ui.ctx().clone();
		let mut data = self.knowledge.lock().unwrap();
		let mut open_popup: Option<GuiPopup> = None;

		if self.state != AppState::AskUsername {
			egui::Panel::top("server_data")
				.resizable(false)
				.default_size(100.0)
				.show(ui, |ui|{
					ui.centered_and_justified( |ui| {
						ui.horizontal( |ui| {
							ui.add_space(50.0);
							ui.label(format!("Connected at {} proto = 1 as {}", data.addr, self.username));
							ui.add_space(400.0);
							ui.label(format!("{} players connected", data.players));
							ui.add_space(400.0);
							ui.heading(format!("{}", self.state));
						});
					});
				});
		}

		if let Some(err) = data.last_error.as_ref().map(|e| e.to_string()) {
			egui::Panel::top("error_banner")
				.resizable(false)
				.default_size(40.0)
				.show(ui, |ui| {
					ui.horizontal(|ui| {
						ui.colored_label(egui::Color32::RED, format!("Erreur: {}", err));
						if ui.button("OK").clicked() {
							data.last_error = None;
						}
					});
				});
		}

		if self.state != AppState::AskUsername && self.state != AppState::Connect {
			egui::Panel::bottom("state_data")
				.resizable(false)
				.max_size(350.0)
				.min_size(150.0)
				.show(ui, |ui| {
					 if self.state == AppState::Rooms {
						TableBuilder::new(ui)
							.column(Column::remainder())
							.column(Column::remainder())
							.column(Column::remainder())
							.header(30.0, |mut header| {
								header.col(|ui| { ui.strong("Player"); });
								header.col(|ui| { ui.strong("NPCs"); });
								header.col(|ui| { ui.strong("Items"); });
							})
							.body(|mut body| {
								for n in 0..cmp::max(
									cmp::max(data.room.players.len(), data.room.items.len()), 
									cmp::max(data.room.items.len(), data.room.npcs.len())
								) {
									let p = data.room.players.iter().nth(n).cloned();
									let npc = data.room.npcs.get(n).cloned();
									let item = data.room.items.get(n).cloned();

									body.row(20.0, |mut row| {
										row.col(|ui| {
											if let Some(p) = &p && ui.button(p).clicked() {
                                                open_popup = Some(GuiPopup::Player { name: p.clone() });
											}
										});
										row.col(|ui| {
											if let Some(id) = &npc {
												let name = data.npcs.get(id).map(|npc| npc.name.clone()).unwrap_or_else(|| format!("{{{}}}", id));
												if ui.button(name).clicked() {
													open_popup = Some(GuiPopup::Npc { id: id.clone() });
												}
											}
										});
										row.col(|ui| {
											if let Some(id) = &item {
												let name = data.items.get(id).map(|item| item.name.clone()).unwrap_or_else(|| format!("{{{}}}", id));
												if ui.button(name).clicked() {
													open_popup = Some(GuiPopup::Item { id: id.clone(), in_room: true });
												}
											}
										});
									});
								}
							});
					 }

					 if self.state == AppState::Stats {
						ui.add_space(20.0);
						ui.horizontal( |ui| {
							ui.strong("HP:");
							ui.label(format!("{}/{}", data.player.status.hp, data.player.status.max_hp));
						});
						let mut armor = data.player.status.armor.clone();
						if !armor.is_empty() {
							armor = data.item_name(&armor);
						}
						let mut weapon = data.player.status.weapon.clone();
						if !weapon.is_empty() {
							weapon = data.item_name(&weapon);
						}
						ui.horizontal( |ui| {
							ui.strong("Armor:");
							ui.label(armor);
							ui.add_space(200.0);
							if ui.button("Unequip").clicked() && !data.player.status.armor.is_empty() && let Some(tx) = &self.action_tx {
                                let cmd = crate::messages::Command {
                                    kind: crate::messages::CommandKind::Unequip,
                                    payload: crate::messages::Payload::new(&[
                                        crate::messages::PayloadKind::String(data.player.status.armor.clone()),
                                    ]),
                                };
                                let _ = tx.send(UiAction::Command(cmd));
							}
						});
						ui.horizontal( |ui| {
							ui.strong("Weapon:");
							ui.label(weapon);
							ui.add_space(200.0);
							if ui.button("Unequip").clicked() && !data.player.status.weapon.is_empty() && let Some(tx) = &self.action_tx {
                                let cmd = crate::messages::Command {
                                    kind: crate::messages::CommandKind::Unequip,
                                    payload: crate::messages::Payload::new(&[
                                        crate::messages::PayloadKind::String(data.player.status.weapon.clone()),
                                    ]),
                                };
                                let _ = tx.send(UiAction::Command(cmd));
							}
						});
					 }

					 if self.state == AppState::Quests {
						match self.selected_quest.as_ref().and_then(|id| data.player.quests.get(id)).cloned() {
							Some(qp) => {
								let giver = data.npcs.get(&qp.giver).map(|npc| npc.name.clone()).unwrap_or_else(|| qp.giver.clone());
								let (name, progression, rewards) = match data.quests.get(&qp.quest) {
									Some(quest) => (
										quest.name.clone(),
										quest_progression(&data, quest, &qp),
										quest_rewards(&data, quest),
									),
									None => (format!("{{{}}}", qp.quest), format!("{}", qp.progress), String::new()),
								};
								let status = match qp.status {
									crate::game::QuestStatus::Active => "Active",
									crate::game::QuestStatus::Completed => "Completed",
									crate::game::QuestStatus::Abandoned => "Abandoned",
								};

								TableBuilder::new(ui)
									.column(Column::remainder())
									.column(Column::remainder())
									.column(Column::remainder())
									.column(Column::remainder())
									.column(Column::remainder())
									.header(30.0, |mut header| {
										header.col(|ui| { ui.strong("Quest Name"); });
										header.col(|ui| { ui.strong("From"); });
										header.col(|ui| { ui.strong("Status"); });
										header.col(|ui| { ui.strong("Progression"); });
										header.col(|ui| { ui.strong("Rewards"); });
									})
									.body(|mut body| {
										body.row(18.0, |mut row| {
											row.col(|ui| { ui.label(&name); });
											row.col(|ui| { ui.label(&giver); });
											row.col(|ui| { ui.label(status); });
											row.col(|ui| { ui.label(&progression); });
											row.col(|ui| { ui.label(&rewards); });
										});
									});
							}
							None => {
								ui.add_space(20.0);
								ui.vertical_centered(|ui| {
									ui.label("Select a quest with the details button to see its progression and rewards.");
								});
							}
						}
					 }

					 if self.state == AppState::Group {
						ui.add_space(10.0);
						ui.horizontal( |ui| {
							ui.add_space(20.0);
							ui.strong("Invitations")
						});
						let invitations: Vec<String> = data.invitations.iter().cloned().collect();
						for group in &invitations {
							ui.horizontal(|ui| {
								ui.add_space(30.0);
								ui.label(group);
								if ui.button("Accept").clicked() {
									data.invitations.remove(group);
									if let Some(tx) = &self.action_tx {
										let cmd = crate::messages::Command {
											kind: crate::messages::CommandKind::GroupJoin,
											payload: crate::messages::Payload::new(&[
												crate::messages::PayloadKind::String(group.clone()),
											]),
										};
										let _ = tx.send(UiAction::Command(cmd));
									}
								}
								if ui.button("Dismiss").clicked() {
									data.invitations.remove(group);
								}
							});
						}
					}

					if self.state == AppState::Chat {
						if self.chat_scope.is_empty() {
							self.chat_scope = crate::messages::EventScope::Global.to_string();
						}
						ui.centered_and_justified(|ui| {
							ui.horizontal(|ui| {
								ui.add_space(20.0);
								egui::ComboBox::from_id_salt("chat_scope")
									.selected_text(self.chat_scope.clone())
									.show_ui(ui, |ui| {
										for scope in [
											crate::messages::EventScope::Global,
											crate::messages::EventScope::Group,
											crate::messages::EventScope::Room,
										] {
											let label = scope.to_string();
											ui.selectable_value(&mut self.chat_scope, label.clone(), label);
										}
									});
								ui.add(
									egui::TextEdit::singleline(&mut self.message)
										.hint_text("Your message")
										.desired_width(1500.0),
								);
								if ui.button("Send").clicked() && !self.message.trim().is_empty() {
									if let Some(tx) = &self.action_tx {
										let cmd = crate::messages::Command {
											kind: crate::messages::CommandKind::Chat,
											payload: crate::messages::Payload::new(&[
												crate::messages::PayloadKind::String(self.chat_scope.clone()),
												crate::messages::PayloadKind::String(self.message.clone()),
											]),
										};
										let _ = tx.send(UiAction::Command(cmd));
									}
									self.message.clear();
								}
							});
						});
					}
				});
		}

		if self.state != AppState::AskUsername && self.state != AppState::Connect {
			egui::Panel::left("nav_panel")
				.resizable(false)
				.default_size(100.0)
				.show(ui, |ui| {
					ui.add_space(30.0);
					if ui.button("Rooms").clicked() {
						self.state = AppState::Rooms
					}
					ui.add_space(30.0);
					ui.separator();
					ui.add_space(30.0);
					if ui.button("Stats").clicked() {
						self.state = AppState::Stats
					}
					ui.add_space(30.0);
					ui.separator();
					ui.add_space(30.0);
					if ui.button("Quests").clicked() {
						self.state = AppState::Quests
					}
					ui.add_space(30.0);
					ui.separator();
					ui.add_space(30.0);
					if ui.button("Group").clicked() {
						self.state = AppState::Group
					}
					ui.add_space(30.0);
					ui.separator();
					ui.add_space(30.0);
					if ui.button("Chat").clicked() {
						self.state = AppState::Chat
					}
					ui.add_space(30.0);
					ui.separator();
					ui.add_space(30.0);
					if ui.button("Exit").clicked() {
						self.state = AppState::AskUsername;
						self.username_input.clear();
						if let Some(tx) = &self.action_tx {
							let cmd = crate::messages::Command::new(crate::messages::CommandKind::Quit);
							let _ = tx.send(UiAction::Command(cmd));
						};
					}
				});
			}

		if data.ui_popup_show {
			egui::Panel::right("popup")
				.resizable(false)
				.min_size(150.0)
				.max_size(300.0)
				.show(ui, |ui| {
					ui.add_space(20.0);
					if let Some((title, content)) = data.ui_popup.clone() {
						ui.horizontal(|ui| {
							ui.strong(title);
							if ui.button("Close").clicked() {
								data.ui_popup_show = false;
							}
						});
						ui.separator();
						ui.label(content);
					}
				});
		}


		drop(data);

		if open_popup.is_some() {
			self.popup = open_popup;
		}

		egui::CentralPanel::default().show(ui, |ui| {
			match self.state {
				AppState::AskUsername => self.show_username_screen(ui),
				AppState::Connect => self.connect(ui),
				AppState::Rooms => self.rooms_ui(ui),
				AppState::Stats => self.stats_ui(ui),
				AppState::Quests => self.quests_ui(ui),
				AppState::Group => self.group_ui(ui),
				AppState::Chat => self.chat_ui(ui),
			}
		});

		let ctx = self.egui_ctx.clone();
		self.show_popup_window(&ctx);
	}
}

impl MyApp {
	pub fn start(mut self, client: crate::network::Client) -> Option<crate::messages::Error> {
		let options = eframe::NativeOptions::default();

		let result = eframe::run_native(
			"My First GUI",
			options,
			Box::new(move |cc| {
				cc.egui_ctx.all_styles_mut(|style| {
					style.text_styles = [
						(egui::TextStyle::Heading, egui::FontId::new(50.0, egui::FontFamily::Proportional)),
						(egui::TextStyle::Body, egui::FontId::new(20.0, egui::FontFamily::Proportional)),
						(egui::TextStyle::Button, egui::FontId::new(30.0, egui::FontFamily::Proportional)),
						(egui::TextStyle::Monospace, egui::FontId::new(18.0, egui::FontFamily::Monospace)),
					]
					.into();
				});

				self.egui_ctx = cc.egui_ctx.clone();
				let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
				self.action_tx = Some(tx);
				tokio::spawn(run_network(
					client,
					self.knowledge.clone(),
					rx,
					self.egui_ctx.clone(),
				));

				Ok(Box::new(self))
			}),
		);

		result.err().map(|_| crate::messages::Error::UnknownError)
	}

	fn show_username_screen(&mut self, ui: &mut egui::Ui) {
		ui.vertical_centered(|ui| {
			ui.add_space(50.0);
			ui.heading("Enter username");
			ui.add_space(20.0);

			let response = ui.add(
				egui::TextEdit::singleline(&mut self.username_input)
					.hint_text("Username")
					.desired_width(200.0),
			);

			response.request_focus();

			let enter_pressed = response.lost_focus()
				&& ui.input(|i| i.key_pressed(egui::Key::Enter));

			ui.add_space(10.0);
			let button_clicked = ui.button("Validate").clicked();

			if (enter_pressed || button_clicked) && !self.username_input.trim().is_empty() {
				self.username = self.username_input.trim().to_string();
				self.state = AppState::Connect;
			}
		});
	}

	fn connect(&mut self, ui: &mut egui::Ui) {
		ui.vertical_centered(|ui| {
			ui.add_space(50.0);
			ui.heading(format!("Welcome {} !", self.username));
			ui.add_space(200.0);
			ui.heading(egui::RichText::new("42").size(80.0).strong());
			ui.heading(egui::RichText::new("TAP").size(80.0).strong());
			ui.add_space(10.0);
			if ui.button("Connect").clicked() {
				if let Some(tx) = &self.action_tx {
					let cmd = crate::messages::Command {
						kind: crate::messages::CommandKind::Connect,
						payload: crate::messages::Payload::new(&[
							crate::messages::PayloadKind::String(self.username.clone()),
						]),
					};
					let _ = tx.send(UiAction::Command(cmd));
				}
				self.state = AppState::Rooms;
			}
		});
	}

	fn rooms_ui(&mut self, ui: &mut egui::Ui) {

		let data = self.knowledge.lock().unwrap();
		let mut open_popup: Option<GuiPopup> = None;
		let in_combat = data.room.combat.index(&data.player.username).is_some();

		ui.add_space(20.0);
		ui.vertical_centered( |ui| {
			if ui.button("Refresh").clicked() && let Some(tx) = &self.action_tx {
                let _ = tx.send(UiAction::Command(crate::messages::Command::new(crate::messages::CommandKind::Look)));
			}
		});
		ui.add_space(20.0);
		ui.vertical_centered( |ui| {
			ui.strong(&data.room.room.name);
			ui.label(&data.room.room.description);
		});
		ui.add_space(20.0);

		if in_combat {
			ui.vertical_centered( |ui| {
				ui.heading("Combat");
			});
			ui.add_space(10.0);
			ui.columns(2, |columns| {
				columns[0].strong("Fighters");
				for fighter in &data.room.combat.players {
					columns[0].label(format!("{} ({}/{})", fighter.username, fighter.hp, fighter.max_hp));
				}
				columns[1].strong("Enemies");
				for enemy in &data.room.combat.enemies {
					let name = data.npcs.get(&enemy.id).map(|npc| npc.name.clone()).unwrap_or_else(|| format!("{{{}}}", enemy.id));
					if columns[1].button(format!("{} ({}/{})", name, enemy.hp, enemy.max_hp)).clicked() {
						open_popup = Some(GuiPopup::Enemy { id: enemy.id.clone() });
					}
				}
			});
		} else {
			ui.add_space(20.0);

			let mut to_move: Option<crate::game::Direction> = None;
			if let Some(current) = data.positions.get(&data.room.room.id).cloned() {
				let min_x = data.rpositions.keys().map(|p| p.0).min().unwrap_or(0);
				let max_x = data.rpositions.keys().map(|p| p.0).max().unwrap_or(0);
				let min_y = data.rpositions.keys().map(|p| p.1).min().unwrap_or(0);
				let max_y = data.rpositions.keys().map(|p| p.1).max().unwrap_or(0);

				let avail = ui.available_rect_before_wrap();
				let cols = (max_x - min_x + 1) as f32;
				let rows = (max_y - min_y + 1) as f32;
				let gap_ratio = 0.4;
				let cell_w = (avail.width() / (cols + gap_ratio * (cols - 1.0))).max(110.0);
				let cell_h = (avail.height() / (rows + gap_ratio * (rows - 1.0))).max(45.0);
				let gap_w = cell_w * gap_ratio;
				let gap_h = cell_h * gap_ratio;
				let grid_w = cols * cell_w + (cols - 1.0) * gap_w;
				let grid_h = rows * cell_h + (rows - 1.0) * gap_h;

				egui::ScrollArea::both().show(ui, |ui| {
					let (canvas, _) = ui.allocate_exact_size(
						egui::vec2(grid_w.max(avail.width()), grid_h.max(avail.height())),
						egui::Sense::hover(),
					);
					let origin = egui::pos2(
						canvas.center().x - grid_w / 2.0,
						canvas.center().y - grid_h / 2.0,
					);
					let cell_rect = |x: i32, y: i32| {
						egui::Rect::from_min_size(
							egui::pos2(
								origin.x + (x - min_x) as f32 * (cell_w + gap_w),
								origin.y + (y - min_y) as f32 * (cell_h + gap_h),
							),
							egui::vec2(cell_w, cell_h),
						)
					};

					let stroke = egui::Stroke::new(2.0, ui.visuals().weak_text_color());
					for y in min_y..=max_y {
						for x in min_x..=max_x {
							if data.connections.contains(&((x, y), (x + 1, y))) {
								let a = cell_rect(x, y);
								let b = cell_rect(x + 1, y);
								ui.painter().line_segment(
									[egui::pos2(a.right(), a.center().y), egui::pos2(b.left(), b.center().y)],
									stroke,
								);
							}
							if data.connections.contains(&((x, y), (x, y + 1))) {
								let a = cell_rect(x, y);
								let b = cell_rect(x, y + 1);
								ui.painter().line_segment(
									[egui::pos2(a.center().x, a.bottom()), egui::pos2(b.center().x, b.top())],
									stroke,
								);
							}
						}
					}

					for (pos, id) in data.rpositions.iter() {
						let (x, y) = *pos;
						let explored = data.positions.contains_key(id);
						let name = data.rooms.get(id).map(|r| r.name.clone()).unwrap_or_else(|| format!("{{{}}}", id));
						let direction = if (x, y) == (current.0 + 1, current.1) && data.connections.contains(&(current, (x, y))) {
							Some(crate::game::Direction::East)
						} else if (x, y) == (current.0 - 1, current.1) && data.connections.contains(&((x, y), current)) {
							Some(crate::game::Direction::West)
						} else if (x, y) == (current.0, current.1 - 1) && data.connections.contains(&((x, y), current)) {
							Some(crate::game::Direction::North)
						} else if (x, y) == (current.0, current.1 + 1) && data.connections.contains(&(current, (x, y))) {
							Some(crate::game::Direction::South)
						} else {
							None
						};

						let text = if (x, y) == current {
							egui::RichText::new(name).strong()
						} else if direction.is_some() {
							egui::RichText::new(name)
						} else if explored {
							egui::RichText::new(name).weak()
						} else {
							egui::RichText::new(name).weak().italics()
						};
						let mut button = egui::Button::new(text);
						if (x, y) == current {
							button = button.fill(egui::Color32::DARK_GREEN);
						}
						if let Some(direction) = direction {
							button = button.stroke(egui::Stroke::new(1.5, egui::Color32::LIGHT_GREEN));
							if ui.put(cell_rect(x, y), button).clicked() {
								to_move = Some(direction);
							}
						} else {
							let _ = ui.put(cell_rect(x, y), button);
						}
					}
				});

				if let Some(direction) = to_move && let Some(tx) = &self.action_tx {
                    let cmd = crate::messages::Command {
                        kind: crate::messages::CommandKind::Move,
                        payload: crate::messages::Payload::new(&[
                            crate::messages::PayloadKind::String(direction.to_string()),
                        ]),
                    };
                    let _ = tx.send(UiAction::Command(cmd));
				}
			}
		}

		drop(data);
		if open_popup.is_some() {
			self.popup = open_popup;
		}
	}

	fn stats_ui(&mut self, ui: &mut egui::Ui) {
		let data = self.knowledge.lock().unwrap();

		ui.add_space(15.0);
		ui.strong("Inventory");
		ui.add_space(15.0);
		ui.separator();
		ui.add_space(15.0);

		let item_ids: Vec<String> = data.player.items.clone();
		let mut to_send: Option<(crate::messages::CommandKind, String)> = None;
		let open_popup: Option<GuiPopup> = None;

		TableBuilder::new(ui)
			.column(Column::exact(400.0))
			.column(Column::remainder())
			.column(Column::exact(100.0))
			.column(Column::exact(100.0))
			.column(Column::exact(100.0))
			.header(30.0, |mut header| {
				header.col(|ui| { ui.strong("Item Name"); });
				header.col(|ui| { ui.strong("Description"); });
				header.col(|ui| { ui.strong("Consume"); });
				header.col(|ui| { ui.strong("Equip"); });
				header.col(|ui| { ui.strong("Drop"); });
			})
			.body(|mut body| {
				for id in &item_ids {
					let (name, description) = match data.items.get(id) {
						Some(item) => (item.name.clone(), item.description.clone()),
						None => (format!("{{{}}}", id), String::new()),
					};
 
					body.row(30.0, |mut row| {
						row.col(|ui| {
							ui.strong(&name);
						});
						row.col(|ui| { ui.label(&description); });
						row.col(|ui| {
							if ui.button("consume").clicked() {
								to_send = Some((crate::messages::CommandKind::Consume, id.clone()));
							}
						});
						row.col(|ui| {
							if ui.button("equip").clicked() {
								to_send = Some((crate::messages::CommandKind::Equip, id.clone()));
							}
						});
						row.col(|ui| {
							if ui.button("drop").clicked() {
								to_send = Some((crate::messages::CommandKind::Drop, id.clone()));
							}
						});
					});
				}
			});
		drop(data);
		if open_popup.is_some() {
			self.popup = open_popup;
		}
		if let Some((kind, id)) = to_send && let Some(tx) = &self.action_tx {
            let cmd = crate::messages::Command {
                kind,
                payload: crate::messages::Payload::new(&[
                    crate::messages::PayloadKind::String(id),
                ]),
            };
            let _ = tx.send(UiAction::Command(cmd));
		}
	}

	fn quests_ui(&mut self, ui: &mut egui::Ui) {
		let data = self.knowledge.lock().unwrap();

		let (mut completed, mut actives, mut abandoned) = (0, 0, 0);
		for q in data.player.quests.values() {
			match q.status {
				crate::game::QuestStatus::Completed => completed += 1,
				crate::game::QuestStatus::Active => actives += 1,
				crate::game::QuestStatus::Abandoned => abandoned += 1,
			}
		}

		ui.add_space(30.0);
		ui.horizontal( |ui| {
			ui.add_space(300.0);
			ui.strong(format!("Completed {}", completed));
			ui.add_space(300.0);
			ui.strong(format!("Actives {}", actives));
			ui.add_space(300.0);
			ui.strong(format!("Abandonned {}", abandoned));
		});
		ui.add_space(30.0);
		ui.separator();
		ui.add_space(30.0);
		let quests: Vec<(String, crate::game::QuestStatus, String, String)> = data
			.player
			.quests
			.values()
			.map(|qp| {
				let (name, description) = match data.quests.get(&qp.quest) {
					Some(quest) => (quest.name.clone(), quest.description.clone()),
					None => (format!("{{{}}}", qp.quest), String::new()),
				};
				(qp.quest.clone(), qp.status.clone(), name, description)
			})
			.collect();
 
		let mut to_abandon: Option<String> = None;
		let mut to_select: Option<String> = None;
 
		TableBuilder::new(ui)
			.column(Column::exact(100.0))
			.column(Column::exact(300.0))
			.column(Column::remainder())
			.column(Column::exact(100.0))
			.column(Column::exact(100.0))
			.header(30.0, |mut header| {
				header.col(|ui| { ui.strong("Status"); });
				header.col(|ui| { ui.strong("Quest Name"); });
				header.col(|ui| { ui.strong("Description"); });
				header.col(|ui| { ui.strong("Details"); });
				header.col(|ui| { ui.strong("Abandon"); });
			})
			.body(|mut body| {
				for (id, status, name, description) in &quests {
					let status_str = match status {
						crate::game::QuestStatus::Active => "Active",
						crate::game::QuestStatus::Completed => "Completed",
						crate::game::QuestStatus::Abandoned => "Abandoned",
					};
 
					body.row(30.0, |mut row| {
						row.col(|ui| { ui.label(status_str); });
						row.col(|ui| { ui.label(name); });
						row.col(|ui| { ui.label(description); });
						row.col(|ui| {
							if ui.button("details").clicked() {
								to_select = Some(id.clone());
							}
						});
						row.col(|ui| {
							if matches!(status, crate::game::QuestStatus::Active) && ui.button("abandon").clicked() {
								to_abandon = Some(id.clone());
							}
						});
					});
				}
			});
 
		drop(data);
		if to_select.is_some() {
			self.selected_quest = to_select;
		}
		if let Some(id) = to_abandon && let Some(tx) = &self.action_tx {
            let cmd = crate::messages::Command {
                kind: crate::messages::CommandKind::AbandonQuest,
                payload: crate::messages::Payload::new(&[
                    crate::messages::PayloadKind::String(id),
                ]),
            };
            let _ = tx.send(UiAction::Command(cmd));
		}
	}

	fn show_popup_window(&mut self, ctx: &egui::Context) {
		let Some(popup) = self.popup.take() else { return };
		let mut close = false;
		let mut commands: Vec<crate::messages::Command> = Vec::new();
		let data = self.knowledge.lock().unwrap();

		let window = |title: String| {
			egui::Window::new(title)
				.collapsible(false)
				.resizable(false)
				.anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
		};

		match &popup {
			GuiPopup::Item { id, in_room } => {
				let (name, description, details) = match data.items.get(id) {
					Some(item) => (
						item.name.clone(),
						item.description.clone(),
						match &item.data {
							crate::game::ItemKind::Valuable => "Valuable".to_string(),
							crate::game::ItemKind::Armor { armor } => format!("Armor: {}", armor),
							crate::game::ItemKind::Consumable { heal } => format!("Heal: {}", heal),
							crate::game::ItemKind::Weapon { damage } => format!("Damage: {}", damage),
						},
					),
					None => (format!("{{{}}}", id), String::new(), String::new()),
				};
				window(name).show(ctx, |ui| {
					ui.label(description);
					ui.label(details);
					ui.separator();
					ui.horizontal(|ui| {
						if *in_room {
							if ui.button("Take").clicked() {
								commands.push(crate::messages::Command {
									kind: crate::messages::CommandKind::Take,
									payload: crate::messages::Payload::new(&[
										crate::messages::PayloadKind::String(id.clone()),
									]),
								});
								close = true;
							}
						} else {
							for (label, kind) in [
								("Consume", crate::messages::CommandKind::Consume),
								("Equip", crate::messages::CommandKind::Equip),
								("Drop", crate::messages::CommandKind::Drop),
							] {
								if ui.button(label).clicked() {
									commands.push(crate::messages::Command {
										kind,
										payload: crate::messages::Payload::new(&[
											crate::messages::PayloadKind::String(id.clone()),
										]),
									});
									close = true;
								}
							}
						}
						if ui.button("Return").clicked() {
							close = true;
						}
					});
				});
			}
			GuiPopup::Npc { id } => {
				let (name, description, is_enemy) = match data.npcs.get(id) {
					Some(npc) => (npc.name.clone(), npc.description.clone(), npc.is_enemy()),
					None => (format!("{{{}}}", id), String::new(), false),
				};
				window(name).show(ctx, |ui| {
					ui.label(description);
					ui.separator();
					ui.horizontal(|ui| {
						if is_enemy {
							if ui.button("Attack").clicked() {
								commands.push(crate::messages::Command {
									kind: crate::messages::CommandKind::Attack,
									payload: crate::messages::Payload::new(&[
										crate::messages::PayloadKind::String(id.clone()),
									]),
								});
								close = true;
							}
						} else {
							if ui.button("Talk").clicked() {
								commands.push(crate::messages::Command {
									kind: crate::messages::CommandKind::Talk,
									payload: crate::messages::Payload::new(&[
										crate::messages::PayloadKind::String(id.clone()),
									]),
								});
								close = true;
							}
							if ui.button("Ask for a quest").clicked() {
								commands.push(crate::messages::Command {
									kind: crate::messages::CommandKind::Quest,
									payload: crate::messages::Payload::new(&[
										crate::messages::PayloadKind::String(id.clone()),
									]),
								});
								close = true;
							}
						}
						if ui.button("Return").clicked() {
							close = true;
						}
					});
				});
			}
			GuiPopup::Player { name } => {
				window(name.clone()).show(ctx, |ui| {
					ui.horizontal(|ui| {
						if ui.button("Invite").clicked() {
							commands.push(crate::messages::Command {
								kind: crate::messages::CommandKind::GroupInvite,
								payload: crate::messages::Payload::new(&[
									crate::messages::PayloadKind::String(name.clone()),
								]),
							});
							close = true;
						}
						if ui.button("Return").clicked() {
							close = true;
						}
					});
				});
			}
			GuiPopup::Enemy { id } => {
				let name = data.npcs.get(id).map(|npc| npc.name.clone()).unwrap_or_else(|| format!("{{{}}}", id));
				let status = data.room.combat.enemies.iter().find(|e| &e.id == id).cloned();
				window(name).show(ctx, |ui| {
					if let Some(status) = status {
						ui.label(format!("HP: {}/{}", status.hp, status.max_hp));
						ui.label(format!("Armor: {}", status.armor));
						ui.label(format!("Attack: {}", status.attack));
					}
					ui.separator();
					ui.horizontal(|ui| {
						if ui.button("Attack").clicked() {
							commands.push(crate::messages::Command {
								kind: crate::messages::CommandKind::Attack,
								payload: crate::messages::Payload::new(&[
									crate::messages::PayloadKind::String(id.clone()),
								]),
							});
							close = true;
						}
						if ui.button("Return").clicked() {
							close = true;
						}
					});
				});
			}
			GuiPopup::GroupLeave => {
				window("Leave Group".to_string()).show(ctx, |ui| {
					ui.label("Do you really want to leave the group ?");
					ui.separator();
					ui.horizontal(|ui| {
						if ui.button("Yes").clicked() {
							commands.push(crate::messages::Command::new(crate::messages::CommandKind::GroupLeave));
							close = true;
						}
						if ui.button("No").clicked() {
							close = true;
						}
					});
				});
			}
		}

		drop(data);
		if let Some(tx) = &self.action_tx {
			for cmd in commands {
				let _ = tx.send(UiAction::Command(cmd));
			}
		}
		if !close {
			self.popup = Some(popup);
		}
	}
}

impl MyApp {
	fn group_ui(&mut self, ui: &mut egui::Ui) {
		let data = self.knowledge.lock().unwrap();
		let mut open_popup: Option<GuiPopup> = None;
 
		ui.add_space(20.0);
		ui.horizontal(|ui| {
			let create_clicked = ui.button("Create Group").clicked();
			ui.add(
				egui::TextEdit::singleline(&mut self.group_name)
					.hint_text("Group name")
					.desired_width(500.0),
			);
			if create_clicked && !self.group_name.trim().is_empty() && let Some(tx) = &self.action_tx {
                let cmd = crate::messages::Command {
                    kind: crate::messages::CommandKind::GroupCreate,
                    payload: crate::messages::Payload::new(&[
                        crate::messages::PayloadKind::String(self.group_name.clone()),
                    ]),
                };
                let _ = tx.send(UiAction::Command(cmd));
			}
		});
		ui.add_space(20.0);
		ui.separator();
		ui.add_space(20.0);
 
		if data.player.group.is_empty() {
			ui.label("Not in a group.");
		} else {
			ui.horizontal(|ui| {
				ui.strong(format!("{} members", data.group.name));
				ui.add_space(50.0);
				if ui.button("Leave Group").clicked() {
					open_popup = Some(GuiPopup::GroupLeave);
				}
			});
			for member in data.group.players.iter() {
				ui.label(member);
			}
		}

		drop(data);
		if open_popup.is_some() {
			self.popup = open_popup;
		}
	}

		fn chat_ui(&mut self, ui: &mut egui::Ui) {
		let data = self.knowledge.lock().unwrap();
		egui::ScrollArea::vertical().show(ui, |ui| {
			for msg in data.chat.iter() {
				ui.horizontal(|ui| {
					ui.strong(format!("[{}] {}: ", msg.scope, msg.author));
					ui.label(&msg.content);
				});
			}
		});
	}
}

fn quest_progression(
	data: &crate::tui::Knowledge,
	quest: &crate::game::Quest,
	progress: &crate::game::QuestProgress,
) -> String {
	match &quest.task {
		crate::game::QuestKind::Bring { item, count } => {
			let item = data.items.get(item).map(|i| i.name.clone()).unwrap_or_else(|| item.clone());
			format!("{}/{} {}", progress.progress, count, item)
		}
		crate::game::QuestKind::Kill { enemy, count } => {
			let enemy = data.npcs.get(enemy).map(|n| n.name.clone()).unwrap_or_else(|| enemy.clone());
			format!("{}/{} {}", progress.progress, count, enemy)
		}
		crate::game::QuestKind::Goto { room } => {
			let room = data.rooms.get(room).map(|r| r.name.clone()).unwrap_or_else(|| room.clone());
			format!("Goto {}", room)
		}
		crate::game::QuestKind::Talk { npc } => {
			let npc = data.npcs.get(npc).map(|n| n.name.clone()).unwrap_or_else(|| npc.clone());
			format!("Talk to {}", npc)
		}
	}
}

fn quest_rewards(data: &crate::tui::Knowledge, quest: &crate::game::Quest) -> String {
	quest
		.reward
		.iter()
		.map(|id| data.items.get(id).map(|i| i.name.clone()).unwrap_or_else(|| id.clone()))
		.collect::<Vec<String>>()
		.join(", ")
}
fn queue_quest_describes(
	k: &mut crate::tui::Knowledge,
	commands: &mut Vec<crate::messages::Command>,
) {
	let progresses: Vec<(String, String)> = k.player.quests
		.values()
		.map(|qp| (qp.quest.clone(), qp.giver.clone()))
		.collect();
	for (quest, giver) in progresses {
		if !k.quests.contains_key(&quest) {
			k.describes.insert(quest);
		}
		k.npc_name(&giver);
	}

	let details: Vec<(crate::game::QuestKind, Vec<String>)> = k.player.quests
		.values()
		.filter_map(|qp| k.quests.get(&qp.quest))
		.map(|quest| (quest.task.clone(), quest.reward.clone()))
		.collect();
	for (task, rewards) in details {
		match task {
			crate::game::QuestKind::Bring { item, .. } => { k.item_name(&item); }
			crate::game::QuestKind::Kill { enemy, .. } => { k.npc_name(&enemy); }
			crate::game::QuestKind::Goto { room } => { k.room_name(&room); }
			crate::game::QuestKind::Talk { npc } => { k.npc_name(&npc); }
		}
		for reward in rewards {
			k.item_name(&reward);
		}
	}

	while let Some(id) = k.need() {
		commands.push(crate::messages::Command {
			kind: crate::messages::CommandKind::Describe,
			payload: crate::messages::Payload::new(&[
				crate::messages::PayloadKind::String(id),
			]),
		});
	}
}

async fn run_network(
	mut client: crate::network::Client,
	knowledge: Arc<Mutex<crate::tui::Knowledge>>,
	mut action_rx: mpsc::UnboundedReceiver<UiAction>,
	ctx: egui::Context,
) {
	let mut waiter = crate::utils::Waiter::default();
	let mut commands: Vec<crate::messages::Command> = Vec::new();
	waiter.begin(3);
	loop {
		if !client.is_open() {
			break;
		}

		if let Some(writer) = &client.writer && !waiter.is_waiting() && !commands.is_empty() {
            if writer.write_message(&crate::messages::Message::Command(commands[0].clone())).await.is_err() {
                break;
            }
            waiter.begin(3);
		}

		if let Some(e) = tokio::select! {
			_ = waiter.wait() => {
				Some(crate::messages::Error::ServerTimeOut)
			}
			action = action_rx.recv() => {
				match action {
					Some(UiAction::Command(cmd)) => commands.push(cmd),
					None => break, 
				}
				None
			}
			message = client.reader.read() => {
				match message {
					Ok(Some(raw)) => {
						if let Ok(msg) = crate::messages::Message::from_str(&raw) {
							match msg {
								crate::messages::Message::Error(e) => {
									waiter.end();
									knowledge.lock().unwrap().last_error = Some(e);
									if !commands.is_empty() { commands.remove(0); }
								}
								crate::messages::Message::Response(resp) => {
									waiter.end();
									let command = if commands.is_empty() { None } else { Some(commands.remove(0)) };
									process_response(resp, command, &knowledge, &mut client, &mut commands).await;
								}
								crate::messages::Message::Event(evt) => {
									process_event(evt, &knowledge, &mut commands).await;
								}
								crate::messages::Message::Command(_) => {
								}
							}
						}
						None
					}
					_ => Some(crate::messages::Error::ConnectionClosed),
				}
			}
		} && e.is_fatal() {
			break;
		}

		ctx.request_repaint();
	}
	ctx.send_viewport_cmd(egui::ViewportCommand::Close);
}

async fn process_response(
	response: crate::messages::Response,
	command: Option<crate::messages::Command>,
	knowledge: &Arc<Mutex<crate::tui::Knowledge>>,
	client: &mut crate::network::Client,
	commands: &mut Vec<crate::messages::Command>,
) -> Option<crate::messages::Error> {
	match command {
		Some(command) => match command.kind {
			crate::messages::CommandKind::Chat => {
				if response.payload.is_empty() { None } else { Some(crate::messages::Error::UnexpectedServerResponse) }
			}
			crate::messages::CommandKind::Connect => {
				if response.payload.extract(&mut [
					crate::messages::PayloadExtractor::String(&mut "connected".to_string()),
				]).is_ok() {
					client.state = crate::network::ClientState::Authenticated;
					let mut k = knowledge.lock().unwrap();
					k.player.username.clear();
					if command.payload.extract(&mut [
						crate::messages::PayloadExtractor::String(&mut k.player.username),
					]).is_err() {
						return Some(crate::messages::Error::UnexpectedServerResponse);
					}
					drop(k);
					commands.push(crate::messages::Command::new(crate::messages::CommandKind::Look));
					commands.push(crate::messages::Command::new(crate::messages::CommandKind::Status));
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
					let mut k = knowledge.lock().unwrap();
					k.change_room(room);

					let npc_ids: Vec<String> = k.room.npcs.clone();
					for id in &npc_ids { k.npc_name(id); }

					let item_ids: Vec<String> = k.room.items.clone();
					for id in &item_ids { k.item_name(id); }

					while let Some(id) = k.need() {
						commands.push(crate::messages::Command {
							kind: crate::messages::CommandKind::Describe,
							payload: crate::messages::Payload::new(&[
								crate::messages::PayloadKind::String(id),
							]),
						});
					}
					None
				} else {
					Some(crate::messages::Error::UnexpectedServerResponse)
				}
			}
			crate::messages::CommandKind::Describe => {
				let mut data = crate::game::WorldData::default();
				let mut k = knowledge.lock().unwrap();
				if response.payload.extract(&mut [
					crate::messages::PayloadExtractor::Json(&mut data),
				]).is_ok() && k.update(data).is_ok() {
					queue_quest_describes(&mut k, commands);
					None
				} else {
					Some(crate::messages::Error::UnexpectedServerResponse)
				}
			}
			crate::messages::CommandKind::Status => {
				let mut status = crate::game::PlayerStatus::default();
				if response.payload.extract(&mut [
					crate::messages::PayloadExtractor::Json(&mut status),
				]).is_ok() {
					knowledge.lock().unwrap().player.status = status;
					None
				} else {
					Some(crate::messages::Error::UnexpectedServerResponse)
				}
			}
			crate::messages::CommandKind::Inventory => {
				let mut k = knowledge.lock().unwrap();
				if response.payload.extract(&mut [
					crate::messages::PayloadExtractor::Json(&mut k.player.items),
				]).is_ok() { None } else { Some(crate::messages::Error::UnexpectedServerResponse) }
			}
			crate::messages::CommandKind::Quests => {
				let mut quests: Vec<crate::game::QuestProgress> = Vec::new();
				if response.payload.extract(&mut [
					crate::messages::PayloadExtractor::Json(&mut quests),
				]).is_ok() {
					let mut k = knowledge.lock().unwrap();
					k.player.quests.clear();
					for q in quests { k.player.quests.insert(q.quest.clone(), q); }
					queue_quest_describes(&mut k, commands);
					None
				} else {
					Some(crate::messages::Error::UnexpectedServerResponse)
				}
			}
			crate::messages::CommandKind::GroupDescribe => {
				let mut group = crate::game::Group::default();
				if response.payload.extract(&mut [
					crate::messages::PayloadExtractor::Json(&mut group),
				]).is_ok() {
					knowledge.lock().unwrap().change_group(Some(group));
					None
				} else {
					Some(crate::messages::Error::UnexpectedServerResponse)
				}
			}
			crate::messages::CommandKind::GroupCreate | crate::messages::CommandKind::GroupJoin => {
				let mut group = String::new();
				if response.payload.extract(&mut [
					crate::messages::PayloadExtractor::KeyValue { key: &mut "group".to_string(), value: &mut group },
				]).is_ok() {
					commands.push(crate::messages::Command::new(crate::messages::CommandKind::GroupDescribe));
					None
				} else {
					Some(crate::messages::Error::UnexpectedServerResponse)
				}
			}
			crate::messages::CommandKind::GroupLeave => {
				if response.payload.is_empty() {
					knowledge.lock().unwrap().change_group(None);
					None
				} else {
					Some(crate::messages::Error::UnexpectedServerResponse)
				}
			}
			crate::messages::CommandKind::Move => {
				let mut room = String::new();
				if response.payload.extract(&mut [
					crate::messages::PayloadExtractor::KeyValue { key: &mut "room".to_string(), value: &mut room },
				]).is_ok() {
					commands.push(crate::messages::Command::new(crate::messages::CommandKind::Look));
					None
				} else {
					Some(crate::messages::Error::UnexpectedServerResponse)
				}
			}
			crate::messages::CommandKind::Attack => {
				let mut combat = crate::game::Combat::default();
				if response.payload.extract(&mut [
					crate::messages::PayloadExtractor::Json(&mut combat),
				]).is_ok() {
					knowledge.lock().unwrap().room.combat = combat;
					commands.push(crate::messages::Command::new(crate::messages::CommandKind::Status));
					None
				} else {
					Some(crate::messages::Error::UnexpectedServerResponse)
				}
			}
			crate::messages::CommandKind::AbandonQuest => {
				if response.payload.is_empty() {
					if let crate::messages::PayloadKind::String(quest_id) = &command.payload.args[0] {
						let mut k = knowledge.lock().unwrap();
						if let Some(q) = k.player.quests.get_mut(quest_id) {
							q.status = crate::game::QuestStatus::Abandoned;
						}
					}
					None
				} else {
					Some(crate::messages::Error::UnexpectedServerResponse)
				}
			}
			crate::messages::CommandKind::Consume => {
				if let crate::messages::PayloadKind::String(mut s) = command.payload.args[0].clone() {
					if response.payload.extract(&mut [
						crate::messages::PayloadExtractor::KeyValue {
							key: &mut "consumed".to_string(),
							value: &mut s,
						},
					]).is_ok() {
						commands.push(crate::messages::Command::new(crate::messages::CommandKind::Inventory));
						commands.push(crate::messages::Command::new(crate::messages::CommandKind::Status));
						commands.push(crate::messages::Command::new(crate::messages::CommandKind::Quests));
						None
					} else {
						Some(crate::messages::Error::UnexpectedServerResponse)
					}
				} else {
					None
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
						commands.push(crate::messages::Command::new(crate::messages::CommandKind::Inventory));
						commands.push(crate::messages::Command::new(crate::messages::CommandKind::Quests));
						commands.push(crate::messages::Command::new(crate::messages::CommandKind::Look));
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
						commands.push(crate::messages::Command::new(crate::messages::CommandKind::Inventory));
						commands.push(crate::messages::Command::new(crate::messages::CommandKind::Status));
						commands.push(crate::messages::Command::new(crate::messages::CommandKind::Quests));
						None
					} else {
						Some(crate::messages::Error::UnexpectedServerResponse)
					}
				} else {
					None
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
						commands.push(crate::messages::Command::new(crate::messages::CommandKind::Inventory));
						commands.push(crate::messages::Command::new(crate::messages::CommandKind::Status));
						commands.push(crate::messages::Command::new(crate::messages::CommandKind::Quests));
						None
					} else {
						Some(crate::messages::Error::UnexpectedServerResponse)
					}
				} else {
					None
				}
			}
			crate::messages::CommandKind::Take => {
				if response.payload.extract(&mut [
					crate::messages::PayloadExtractor::KeyValue {
						key: &mut "taken".to_string(),
						value: &mut "".to_string(),
					},
				]).is_ok() {
					commands.push(crate::messages::Command::new(crate::messages::CommandKind::Look));
					commands.push(crate::messages::Command::new(crate::messages::CommandKind::Quests));
					commands.push(crate::messages::Command::new(crate::messages::CommandKind::Inventory));
					None
				} else {
					Some(crate::messages::Error::UnexpectedServerResponse)
				}
			}
						crate::messages::CommandKind::Talk => {
				let mut dialogue = String::new();
				let mut k = knowledge.lock().unwrap();
				if response.payload.extract(&mut [
					crate::messages::PayloadExtractor::String(&mut dialogue),
				]).is_ok() {
					if let crate::messages::PayloadKind::String(id) = &command.payload.args[0] {
						let npc_name = k.npc_name(id);
						k.show_popup(npc_name, dialogue);
					}
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
					let mut k = knowledge.lock().unwrap();
					k.show_popup("New quest", format!("New quest obtained: {}", quest.name));
					drop(k);
					commands.push(crate::messages::Command::new(crate::messages::CommandKind::Quests));
					None
				} else {
					Some(crate::messages::Error::UnexpectedServerResponse)
				}
			}

			crate::messages::CommandKind::Who => {
				let mut players = String::new();
				let mut k = knowledge.lock().unwrap();
				if response.payload.extract(&mut [
					crate::messages::PayloadExtractor::KeyValue {
						key: &mut "players".to_string(),
						value: &mut players,
					},
				]).is_ok() && let Ok(v) = players.parse::<usize>() {
					k.players = v;
					None
				} else {
					Some(crate::messages::Error::UnexpectedServerResponse)
				}
			}
			_ => None,
		},
		None => {
			client.proto.clear();
			if response.payload.extract(&mut [
				crate::messages::PayloadExtractor::String(&mut "hello".to_string()),
				crate::messages::PayloadExtractor::KeyValue { key: &mut "proto".to_string(), value: &mut client.proto },
			]).is_err() {
				Some(crate::messages::Error::UnexpectedServerResponse)
			} else {
				let mut k = knowledge.lock().unwrap();
				k.addr = client.addr.clone();
				k.proto = client.proto.clone();
				None
			}
		}
	}
}

async fn process_event(
	event: crate::messages::Event,
	knowledge: &Arc<Mutex<crate::tui::Knowledge>>,
	commands: &mut Vec<crate::messages::Command>,
) -> Option<crate::messages::Error> {
	match (&event.scope, &event.kind) {
		(crate::messages::EventScope::Group, crate::messages::EventKind::Invite) => {
			let mut group = String::new();
			if event.payload.extract(&mut [crate::messages::PayloadExtractor::String(&mut group)]).is_err() {
				return Some(crate::messages::Error::UnexpectedServerResponse);
			}
			knowledge.lock().unwrap().invitations.insert(group);
			None
		}
		(crate::messages::EventScope::Group, crate::messages::EventKind::Join) => {
			let mut player = String::new();
			if event.payload.extract(&mut [crate::messages::PayloadExtractor::String(&mut player)]).is_err() {
				return Some(crate::messages::Error::UnexpectedServerResponse);
			}
			let mut k = knowledge.lock().unwrap();
			if k.player.group.is_empty() || !k.group.players.insert(player) {
				return Some(crate::messages::Error::UnexpectedServerResponse);
			}
			None
		}
		(crate::messages::EventScope::Group, crate::messages::EventKind::Leave) => {
			let mut player = String::new();
			if event.payload.extract(&mut [crate::messages::PayloadExtractor::String(&mut player)]).is_err() {
				return Some(crate::messages::Error::UnexpectedServerResponse);
			}
			let mut k = knowledge.lock().unwrap();
			if k.player.group.is_empty() || !k.group.players.remove(&player) {
				return Some(crate::messages::Error::UnexpectedServerResponse);
			}
			None
		}
		(crate::messages::EventScope::Stats, crate::messages::EventKind::Players) => {
			let mut n = String::new();
			if event.payload.extract(&mut [
				crate::messages::PayloadExtractor::KeyValue { key: &mut "players".to_string(), value: &mut n },
			]).is_err() {
				return Some(crate::messages::Error::UnexpectedServerResponse);
			}
			match n.parse::<usize>() {
				Ok(v) => { knowledge.lock().unwrap().players = v; None }
				Err(_) => Some(crate::messages::Error::UnexpectedServerResponse),
			}
		}
		(crate::messages::EventScope::Room, crate::messages::EventKind::PresenceEnter) => {
			let mut player = String::new();
			if event.payload.extract(&mut [crate::messages::PayloadExtractor::String(&mut player)]).is_err()
				|| !knowledge.lock().unwrap().room.players.insert(player) {
				return Some(crate::messages::Error::UnexpectedServerResponse);
			}
			None
		}
		(crate::messages::EventScope::Room, crate::messages::EventKind::PresenceLeave) => {
			let mut player = String::new();
			if event.payload.extract(&mut [crate::messages::PayloadExtractor::String(&mut player)]).is_err()
				|| !knowledge.lock().unwrap().room.players.remove(&player) {
				return Some(crate::messages::Error::UnexpectedServerResponse);
			}
			None
		}
		(crate::messages::EventScope::Player, crate::messages::EventKind::QuestComplete) => {
			let mut quest = String::new();
			let mut k = knowledge.lock().unwrap();
			if event.payload.extract(&mut [
				crate::messages::PayloadExtractor::String(&mut quest),
			]).is_ok() && let Some(q) = k.player.quests.get_mut(&quest) {
				q.status = crate::game::QuestStatus::Completed;
				commands.push(crate::messages::Command::new(crate::messages::CommandKind::Inventory));
			} else {
				return Some(crate::messages::Error::UnexpectedServerResponse);
			}
			None
		}

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
			knowledge.lock().unwrap().chat.push(message);
			None
		}
				(crate::messages::EventScope::Room, crate::messages::EventKind::CombatEnd) => {
			let mut k = knowledge.lock().unwrap();
			if !event.payload.is_empty() {
				return Some(crate::messages::Error::UnexpectedServerResponse);
			}
			k.room.combat = crate::game::Combat::default();
			drop(k);
			commands.push(crate::messages::Command::new(crate::messages::CommandKind::Look));
			None
		}
		(crate::messages::EventScope::Room, crate::messages::EventKind::CombatStats) => {
			let mut combat = crate::game::Combat::default();
			if event.payload.extract(&mut [
				crate::messages::PayloadExtractor::Json(&mut combat),
			]).is_err() {
				return Some(crate::messages::Error::UnexpectedServerResponse);
			}
			knowledge.lock().unwrap().room.combat = combat;
			commands.push(crate::messages::Command::new(crate::messages::CommandKind::Status));
			None
		}
		(crate::messages::EventScope::Player, crate::messages::EventKind::Die) => {
			let mut k = knowledge.lock().unwrap();
			if !event.payload.is_empty() {
				return Some(crate::messages::Error::UnexpectedServerResponse);
			}
			k.room.combat = crate::game::Combat::default();
			drop(k);
			commands.push(crate::messages::Command::new(crate::messages::CommandKind::Status));
			commands.push(crate::messages::Command::new(crate::messages::CommandKind::Inventory));
			commands.push(crate::messages::Command::new(crate::messages::CommandKind::Quests));
			commands.push(crate::messages::Command::new(crate::messages::CommandKind::Look));
			None
		}
		(_, _) => None,
	}
}


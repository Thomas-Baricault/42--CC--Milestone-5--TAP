use ratatui::widgets::Widget;

use crate::tui::Color;

impl crate::tui::ToListItem for crate::game::Npc {
    fn update_item(knowledge: &crate::tui::Knowledge, s: &str) -> Option<Self> {
        knowledge.npcs.get(s).cloned()
    }

    fn to_item(&self, _knowledge: &crate::tui::Knowledge) -> ratatui::widgets::ListItem<'static> {
        ratatui::widgets::ListItem::new(
            ratatui::text::Span::styled(self.name.clone(), ratatui::style::Style::default().fg(
                if self.is_enemy() {
                    ratatui::style::Color::Red
                } else {
                    ratatui::style::Color::White
                }
            )),
        )
    }
}

impl crate::tui::Widget for crate::game::Npc {
    fn width(&self) -> u16 {
        self.description.len() as u16
    }

    fn height(&self) -> u16 {
        1
    }

    fn render_with_data(&mut self, _knowledge: &mut crate::tui::Knowledge, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        ratatui::widgets::Paragraph::new(self.description.as_str())
            .render(area, buf);
    }
}

impl crate::tui::PopupDescribeInfos for crate::game::Npc {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn title(&self) -> &str {
        &self.name
    }
}

impl crate::tui::ToListItem for crate::game::Item {
    fn update_item(knowledge: &crate::tui::Knowledge, s: &str) -> Option<Self> {
        knowledge.items.get(s).cloned()
    }

    fn to_item(&self, _knowledge: &crate::tui::Knowledge) -> ratatui::widgets::ListItem<'static> {
        ratatui::widgets::ListItem::new(self.name.clone())
    }
}

impl crate::tui::Widget for crate::game::Item {
    fn width(&self) -> u16 {
        self.description.len() as u16
    }

    fn height(&self) -> u16 {
        1
    }

    fn render_with_data(&mut self, _knowledge: &mut crate::tui::Knowledge, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        ratatui::widgets::Paragraph::new(self.description.as_str())
            .render(area, buf);
    }
}

impl crate::tui::PopupDescribeInfos for crate::game::Item {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn title(&self) -> &str {
        &self.name
    }
}

impl crate::tui::Color for crate::game::QuestStatus {
    fn color(&self) -> ratatui::style::Color {
        match self {
            crate::game::QuestStatus::Abandoned => ratatui::style::Color::Red,
            crate::game::QuestStatus::Active => ratatui::style::Color::Yellow,
            crate::game::QuestStatus::Completed => ratatui::style::Color::LightGreen,
        }
    }
}

impl crate::tui::ToListItem for crate::game::Quest {
    fn update_item(knowledge: &crate::tui::Knowledge, s: &str) -> Option<Self> {
        knowledge.quests.get(s).cloned()
    }

    fn to_item(&self, knowledge: &crate::tui::Knowledge) -> ratatui::widgets::ListItem<'static> {
        ratatui::widgets::ListItem::new(ratatui::text::Line::from(vec![
            ratatui::text::Span::styled("(", ratatui::style::Style::default().fg(ratatui::style::Color::White)),
            ratatui::text::Span::styled(
                knowledge.player.quests[&self.id].status.to_string(),
                ratatui::style::Style::default().fg(knowledge.player.quests[&self.id].status.color())
            ),
            ratatui::text::Span::styled(") ", ratatui::style::Style::default().fg(ratatui::style::Color::White)),
            ratatui::text::Span::styled(self.name.clone(), ratatui::style::Style::default().fg(ratatui::style::Color::White)),
        ]))
    }
}

impl crate::tui::Widget for crate::game::Quest {
    fn width(&self) -> u16 {
        std::cmp::max(
            self.description.len() as u16,
            50,
        )
    }

    fn height(&self) -> u16 {
        9 + self.reward.len() as u16
    }

    fn render_with_data(&mut self, knowledge: &mut crate::tui::Knowledge, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        if !knowledge.player.quests.contains_key(&self.id) {
            return;
        }
        let progress = knowledge.player.quests[&self.id].clone();
        let mut lines = vec![
            ratatui::text::Line::from(vec![
                ratatui::text::Span::styled(self.name.as_str(), ratatui::style::Style::default().fg(ratatui::style::Color::White)),
                ratatui::text::Span::styled(" (from ", ratatui::style::Style::default().fg(ratatui::style::Color::White)),
                ratatui::text::Span::styled(knowledge.npc_name(&progress.giver).clone(), ratatui::style::Style::default().fg(ratatui::style::Color::LightMagenta)),
                ratatui::text::Span::styled("): ", ratatui::style::Style::default().fg(ratatui::style::Color::White)),
                ratatui::text::Span::styled(progress.status.to_string(), ratatui::style::Style::default().fg(progress.status.color())),
            ]),
            ratatui::text::Line::default(),
            ratatui::text::Line::from(self.description.as_str()),
            ratatui::text::Line::default(),
            ratatui::text::Line::from(match &self.task {
                crate::game::QuestKind::Bring { item, count } => vec![
                    ratatui::text::Span::styled("Bring ", ratatui::style::Style::default().fg(ratatui::style::Color::White)),
                    ratatui::text::Span::styled(count.to_string(), ratatui::style::Style::default().fg(ratatui::style::Color::LightGreen)),
                    ratatui::text::Span::styled(" ", ratatui::style::Style::default().fg(ratatui::style::Color::White)),
                    ratatui::text::Span::styled(knowledge.item_name(item), ratatui::style::Style::default().fg(ratatui::style::Color::LightMagenta)),
                ],
                crate::game::QuestKind::Goto { room } => vec![
                    ratatui::text::Span::styled("Go to ", ratatui::style::Style::default().fg(ratatui::style::Color::White)),
                    ratatui::text::Span::styled(knowledge.room_name(room), ratatui::style::Style::default().fg(ratatui::style::Color::LightMagenta)),
                ],
                crate::game::QuestKind::Kill { enemy, count } => vec![
                    ratatui::text::Span::styled("Kill ", ratatui::style::Style::default().fg(ratatui::style::Color::White)),
                    ratatui::text::Span::styled(count.to_string(), ratatui::style::Style::default().fg(ratatui::style::Color::LightGreen)),
                    ratatui::text::Span::styled(" ", ratatui::style::Style::default().fg(ratatui::style::Color::White)),
                    ratatui::text::Span::styled(knowledge.npc_name(enemy), ratatui::style::Style::default().fg(ratatui::style::Color::LightMagenta)),
                ],
                crate::game::QuestKind::Talk { npc } => vec![
                    ratatui::text::Span::styled("Talk to ", ratatui::style::Style::default().fg(ratatui::style::Color::White)),
                    ratatui::text::Span::styled(knowledge.npc_name(npc), ratatui::style::Style::default().fg(ratatui::style::Color::LightMagenta)),
                ],
            }),
            ratatui::text::Line::from(match &progress.status {
                crate::game::QuestStatus::Abandoned => "You have abandoned this quest!".to_string(),
                crate::game::QuestStatus::Active => if progress.is_complete(&knowledge.quests) {
                    format!(
                        "Requirements completed, go talk to {}",
                        knowledge.npc_name(&progress.giver),
                    )
                } else {
                    format!(
                        "{}/{}",
                        progress.progress,
                        match &self.task {
                            crate::game::QuestKind::Bring { item: _, count } => count,
                            crate::game::QuestKind::Kill { enemy: _, count } => count,
                            _ => &1,
                        },
                    )
                },
                crate::game::QuestStatus::Completed => "Quest completed!".to_string(),
            }),
            ratatui::text::Line::default(),
            ratatui::text::Line::from("Reward:"),
        ];
        for item in &self.reward {
            lines.push(ratatui::text::Line::from(format!("- {}", knowledge.item_name(item))));
        }
        ratatui::widgets::Paragraph::new(lines)
            .render(area, buf);
    }
}

impl crate::tui::PopupDescribeInfos for crate::game::Quest {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn title(&self) -> &str {
        &self.name
    }
}

impl crate::tui::PopupDescribeInfos for crate::game::PlayerStatus {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn title(&self) -> &str {
        &self.username
    }
}

impl crate::tui::Widget for crate::game::PlayerStatus {
    fn width(&self) -> u16 {
        20
    }

    fn height(&self) -> u16 {
        3
    }

    fn render_with_data(&mut self, knowledge: &mut crate::tui::Knowledge, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let [top, middle, bottom] = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
        ])
            .areas(area);
        ratatui::widgets::Paragraph::new(format!(
            "HP: {}/{}",
            self.hp,
            self.max_hp,
        ))
            .render(top, buf);
        ratatui::widgets::Paragraph::new(format!(
            "Armor: {}",
            if self.armor.is_empty() {
                "None".to_string()
            } else {
                format!(
                    "{} ({})",
                    knowledge.item_name(&self.armor.clone()),
                    if let Some(item) = knowledge.items.get(&self.armor) && let crate::game::ItemKind::Armor { armor } = &item.data {
                        armor.to_string()
                    } else {
                        "?".to_string()
                    }
                )
            },
        ))
            .render(middle, buf);
        ratatui::widgets::Paragraph::new(format!(
            "Weapon: {}",
            if self.weapon.is_empty() {
                "None".to_string()
            } else {
                format!(
                    "{} ({})",
                    knowledge.item_name(&self.weapon.clone()),
                    if let Some(item) = knowledge.items.get(&self.weapon) && let crate::game::ItemKind::Weapon { damage } = &item.data {
                        damage.to_string()
                    } else {
                        "?".to_string()
                    }
                )
            },
        ))
            .render(bottom, buf);
    }
}

impl crate::tui::PopupDescribeInfos for crate::game::EnemyStatus {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn title(&self) -> &str {
        ""
    }
}

impl crate::tui::Widget for crate::game::EnemyStatus {
    fn width(&self) -> u16 {
        20
    }

    fn height(&self) -> u16 {
        3
    }

    fn render_with_data(&mut self, _knowledge: &mut crate::tui::Knowledge, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let [top, middle, bottom] = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
        ])
            .areas(area);
        ratatui::widgets::Paragraph::new(format!(
            "HP: {}/{}",
            self.hp,
            self.max_hp,
        ))
            .render(top, buf);
        ratatui::widgets::Paragraph::new(format!(
            "Armor: {}",
            self.armor,
        ))
            .render(middle, buf);
        ratatui::widgets::Paragraph::new(format!(
            "Attack: {}",
            self.attack,
        ))
            .render(bottom, buf);
    }
}

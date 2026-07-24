use ratatui::prelude::Widget;

pub struct QuestsPage {
    pub quests: crate::tui::List<crate::game::Quest>,
}

impl QuestsPage {
    pub fn update(&mut self, knowledge: &crate::tui::Knowledge) {
        self.quests.items = knowledge.player.quests
            .values()
            .map(|i| crate::tui::ListItem::Unknown(i.quest.clone()))
            .collect();
    }
}

impl Default for QuestsPage {
    fn default() -> Self {
        Self {
            quests: crate::tui::List::new("Quests"),
        }
    }
}

impl crate::tui::NotebookPage for QuestsPage {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn title(&self) -> &str {
        "QUESTS"
    }
}

impl crate::tui::Widget for QuestsPage {
    fn render_with_data(&mut self, knowledge: &mut crate::tui::Knowledge, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let [_, top, _, bottom] = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Fill(1),
        ])
            .areas(area);
        let [left, middle, right] = ratatui::layout::Layout::horizontal([
            ratatui::layout::Constraint::Fill(1),
            ratatui::layout::Constraint::Fill(1),
            ratatui::layout::Constraint::Fill(1),
        ])
            .areas(top);
        let mut completed = 0;
        let mut actives = 0;
        let mut abandoned = 0;
        for quest in knowledge.player.quests.values() {
            match quest.status {
                crate::game::QuestStatus::Abandoned => abandoned += 1,
                crate::game::QuestStatus::Active => actives += 1,
                crate::game::QuestStatus::Completed => completed += 1,
            }
        }
        ratatui::widgets::Paragraph::new(ratatui::text::Line::from(vec![
            ratatui::text::Span::styled("Completed: ", ratatui::style::Style::default().fg(ratatui::style::Color::White)),
            ratatui::text::Span::styled(completed.to_string(), ratatui::style::Style::default().fg(ratatui::style::Color::LightMagenta)),
        ]))
            .centered()
            .render(left, buf);
        ratatui::widgets::Paragraph::new(ratatui::text::Line::from(vec![
            ratatui::text::Span::styled("Actives: ", ratatui::style::Style::default().fg(ratatui::style::Color::White)),
            ratatui::text::Span::styled(actives.to_string(), ratatui::style::Style::default().fg(ratatui::style::Color::LightMagenta)),
        ]))
            .centered()
            .render(middle, buf);
        ratatui::widgets::Paragraph::new(ratatui::text::Line::from(vec![
            ratatui::text::Span::styled("Abandoned: ", ratatui::style::Style::default().fg(ratatui::style::Color::White)),
            ratatui::text::Span::styled(abandoned.to_string(), ratatui::style::Style::default().fg(ratatui::style::Color::LightMagenta)),
        ]))
            .centered()
            .render(right, buf);
        self.quests.render_with_data(knowledge, bottom, buf);
    }
}

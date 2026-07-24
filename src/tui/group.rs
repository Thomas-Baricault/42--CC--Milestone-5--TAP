use ratatui::widgets::Widget;

use crate::tui::ToListItem;

pub struct GroupPage {
    pub create: crate::tui::Button,
    pub leave: crate::tui::Button,
    pub players: crate::tui::List<String>,
    pub invitations: crate::tui::List<String>,
}

impl GroupPage {
    pub fn update(&mut self, knowledge: &crate::tui::Knowledge) {
        self.players.items = String::items_from_iter(&knowledge.group.players);
        self.invitations.items = String::items_from_iter(&knowledge.invitations);
    }
}

impl Default for GroupPage {
    fn default() -> Self {
        Self {
            create: crate::tui::Button::new("Create"),
            leave: crate::tui::Button::new("Leave"),
            players: crate::tui::List::new("Players"),
            invitations: crate::tui::List::new("Invitations"),
        }
    }
}

impl crate::tui::NotebookPage for GroupPage {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn title(&self) -> &str {
        "GROUP"
    }
}

impl crate::tui::Widget for GroupPage {
    fn render_with_data(&mut self, knowledge: &mut crate::tui::Knowledge, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let block = ratatui::widgets::Block::default()
            .padding(ratatui::widgets::Padding::uniform(1));
        let [top, _, middle, _, middle2, _, bottom] = ratatui::layout::Layout::vertical([
                ratatui::layout::Constraint::Length(1),
                ratatui::layout::Constraint::Length(1),
                ratatui::layout::Constraint::Fill(1),
                ratatui::layout::Constraint::Length(1),
                ratatui::layout::Constraint::Length(1),
                ratatui::layout::Constraint::Length(1),
                ratatui::layout::Constraint::Fill(1),
            ])
                .areas(block.inner(area));
        if knowledge.player.group.is_empty() {
            ratatui::widgets::Paragraph::new("You are not in a group")
                .centered()
                .render(top, buf);
            self.create.centered = true;
            self.create.render(middle, buf);
        } else {
            ratatui::widgets::Paragraph::new(ratatui::text::Line::from(vec![
                ratatui::text::Span::styled("In group: ", ratatui::style::Style::default().fg(ratatui::style::Color::White)),
                ratatui::text::Span::styled(knowledge.group.name.as_str(), ratatui::style::Style::default().fg(ratatui::style::Color::LightMagenta)),
            ]))
                .centered()
                .render(top, buf);
            self.players.render_with_data(knowledge, middle, buf);
            self.leave.centered = true;
            self.leave.render(middle2, buf);
        }
        self.invitations.render_with_data(knowledge, bottom, buf);
    }
}

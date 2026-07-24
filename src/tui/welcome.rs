use ratatui::widgets::Widget;

pub struct WelcomePage {
    pub exit: crate::tui::Button,
}

impl Default for WelcomePage {
    fn default() -> Self {
        Self {
            exit: crate::tui::Button::new("Exit"),
        }
    }
}

impl crate::tui::NotebookPage for WelcomePage {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn title(&self) -> &str {
        "TAP"
    }
}

impl crate::tui::Widget for WelcomePage {
    fn render_with_data(&mut self, _knowledge: &mut crate::tui::Knowledge, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let [top, _, bottom, _] = ratatui::layout::Layout::vertical([
                ratatui::layout::Constraint::Fill(1),
                ratatui::layout::Constraint::Length(1),
                ratatui::layout::Constraint::Length(1),
                ratatui::layout::Constraint::Length(1),
            ])
                .areas(area);
        let mut lines = Vec::new();
        for _ in 0..top.height.saturating_sub(13) / 2 {
            lines.push(ratatui::text::Line::default());
        }
        lines.append(&mut vec![
            ratatui::text::Line::from("        __    ____      "),
            ratatui::text::Line::from("       / /   | /| |     "),
            ratatui::text::Line::from("      / /    |/_| |     "),
            ratatui::text::Line::from("     / /___   / _/      "),
            ratatui::text::Line::from("    |____  | | | /|     "),
            ratatui::text::Line::from("         | | |_|/_|     "),
            ratatui::text::Line::from("         |_|            "),
            ratatui::text::Line::from(" _______   ___    ____  "),
            ratatui::text::Line::from("|__   __| / _ \\  |  _ \\ "),
            ratatui::text::Line::from("   | |   | /_\\ | | |_) |"),
            ratatui::text::Line::from("   | |   |  _  | |  __/ "),
            ratatui::text::Line::from("   | |   | | | | | |    "),
            ratatui::text::Line::from("   |_|   |_| |_| |_|    "),
        ]);
        ratatui::widgets::Paragraph::new(lines)
            .centered()
            .render(top, buf);
        self.exit.centered = true;
        self.exit.render(bottom, buf);
    }
}

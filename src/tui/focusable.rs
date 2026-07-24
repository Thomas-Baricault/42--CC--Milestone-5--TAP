pub trait Focusable: crate::cli::HandleEvent {
    fn focused(&self) -> bool;

    fn span(&self, s: String) -> ratatui::text::Span<'static> {
        ratatui::text::Span::styled(
            s,
            ratatui::style::Style::default()
                .fg(
                    if self.focused() {
                        ratatui::style::Color::White
                    } else {
                        ratatui::style::Color::DarkGray
                    }
                )
        )
    }
}

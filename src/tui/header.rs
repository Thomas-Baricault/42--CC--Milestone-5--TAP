use ratatui::widgets::Widget;

pub struct Header;

impl crate::tui::Widget for Header {
    fn render_with_data(&mut self, knowledge: &mut crate::tui::Knowledge, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let block = ratatui::widgets::Block::bordered()
            .padding(ratatui::widgets::Padding::horizontal(1));
        let [left, right] = ratatui::layout::Layout::horizontal([
            ratatui::layout::Constraint::Fill(1),
            ratatui::layout::Constraint::Fill(1),
        ])
            .areas(block.inner(area));
        ratatui::widgets::Paragraph::new(ratatui::text::Line::from(vec![
            ratatui::text::Span::styled("Connected at ", ratatui::style::Style::default().fg(ratatui::style::Color::White)),
            ratatui::text::Span::styled(&knowledge.addr, ratatui::style::Style::default().fg(ratatui::style::Color::LightBlue)),
            ratatui::text::Span::styled(" (proto=", ratatui::style::Style::default().fg(ratatui::style::Color::White)),
            ratatui::text::Span::styled(&knowledge.proto, ratatui::style::Style::default().fg(ratatui::style::Color::LightGreen)),
            ratatui::text::Span::styled(") as ", ratatui::style::Style::default().fg(ratatui::style::Color::White)),
            ratatui::text::Span::styled(
                if knowledge.player.username.is_empty() {
                    "?"
                } else {
                    &knowledge.player.username
                },
                ratatui::style::Style::default().fg(ratatui::style::Color::LightMagenta),
            ),
        ]))
            .render(left, buf);
        ratatui::widgets::Paragraph::new(ratatui::text::Line::from(vec![
            ratatui::text::Span::styled(knowledge.players.to_string(), ratatui::style::Style::default().fg(ratatui::style::Color::LightMagenta)),
            ratatui::text::Span::styled(" players connected", ratatui::style::Style::default().fg(ratatui::style::Color::White)),
        ]))
            .right_aligned()
            .render(right, buf);
        block.render(area, buf);
    }
}

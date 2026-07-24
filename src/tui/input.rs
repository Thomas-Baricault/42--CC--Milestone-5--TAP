use ratatui::widgets::Widget;

#[derive(Default)]
pub struct Input {
    pub min_width: u16,
    pub max_width: u16,
    pub input: crate::cli::Input,
}

impl Input {
    pub fn new(min_width: u16, max_width: u16) -> Self {
        Self {
            min_width,
            max_width,
            input: crate::cli::Input::default(),
        }
    }
}

impl crate::cli::HandleEvent for Input {
    async fn handle_event(&mut self) -> Option<crate::cli::Event> {
        self.input.handle_event().await
    }
}

impl crate::tui::Widget for Input {
    fn width(&self) -> u16 {
        4 + std::cmp::min(
            std::cmp::max(
                self.input.buffer.len() as u16,
                self.min_width,
            ),
            if self.max_width == 0 {
                u16::MAX
            } else {
                self.max_width
            }
        )
    }

    fn height(&self) -> u16 {
        3
    }

    fn render(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        ratatui::widgets::Paragraph::new(
            ratatui::text::Span::styled(
                &self.input.buffer, 
                ratatui::style::Style::default()
                    .fg(ratatui::style::Color::White),
            ),
        )
            .block(ratatui::widgets::Block::bordered()
                .padding(ratatui::widgets::Padding::horizontal(1)))
            .scroll((0, (self.input.buffer.len() as u16).saturating_sub(area.width - 4)))
            .render(area, buf);
    }
}

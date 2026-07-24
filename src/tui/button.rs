use ratatui::prelude::Widget;

pub struct Button {
    pub text: String,
    pub focus: bool,
    pub centered: bool,
    handler: crate::cli::Handler,
}

impl Button {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            focus: false,
            centered: false,
            handler: crate::cli::Handler::default(),
        }
    }
}

impl crate::tui::Widget for Button {
    fn width(&self) -> u16 {
        self.text.len() as u16
    }

    fn height(&self) -> u16 {
        1
    }

    fn render(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let mut p = ratatui::widgets::Paragraph::new(ratatui::text::Span::styled(
            self.text.as_str(),
            if self.focus {
                ratatui::style::Style::default()
                    .fg(ratatui::style::Color::Black)
                    .bg(ratatui::style::Color::White)
            } else {
                ratatui::style::Style::default()
                    .fg(ratatui::style::Color::White)
            }
        ));
        if self.centered {
            p = p.centered();
        }
        p.render(area, buf);
    }
}

impl crate::cli::HandleEvent for Button {
    async fn handle_event(&mut self) -> Option<crate::cli::Event> {
        self.handler.handle_event().await
    }
}

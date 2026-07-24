use ratatui::widgets::Widget;

use crate::tui::Color;
use crate::tui::Focusable;

impl crate::tui::Color for crate::messages::EventScope {
    fn color(&self) -> ratatui::style::Color {
        match self {
            crate::messages::EventScope::Group => ratatui::style::Color::LightYellow,
            crate::messages::EventScope::Room => ratatui::style::Color::LightCyan,
            _ => ratatui::style::Color::White,
        }
    }
}

pub struct ChatMessage {
    pub scope: crate::messages::EventScope,
    pub author: String,
    pub content: String,
}

pub struct Chat {
    pub scope: crate::messages::EventScope,
    pub input: crate::tui::Input,
    pub messages: Vec<ChatMessage>,
    pub focus: bool,
    scrollbar: crate::tui::Scrollbar,
}

impl Chat {
    pub fn push(&mut self, message: ChatMessage) {
        self.messages.push(message);
        if self.messages.len() > 100 {
            self.messages.remove(0);
        }
    }
}

impl Default for Chat {
    fn default() -> Self {
        Self {
            scope: crate::messages::EventScope::Global,
            input: crate::tui::Input::new(30, 30),
            messages: Vec::new(),
            focus: false,
            scrollbar: crate::tui::Scrollbar::default(),
        }
    }
}

impl crate::cli::HandleEvent for Chat {
    async fn handle_event(&mut self) -> Option<crate::cli::Event> {
        if let Some(event) = self.input.handle_event().await {
            match event {
                crate::cli::Event::Key {
                    code,
                    modifiers,
                } => match (code, modifiers) {
                    (crate::cli::KeyCode::Char(' '), crate::cli::KeyModifiers::CONTROL) => self.scope = match &self.scope {
                        crate::messages::EventScope::Global => crate::messages::EventScope::Room,
                        crate::messages::EventScope::Room => crate::messages::EventScope::Group,
                        _ => crate::messages::EventScope::Global,
                    },
                    (crate::cli::KeyCode::Up, crate::cli::KeyModifiers::NONE) => self.scrollbar.previous(),
                    (crate::cli::KeyCode::Down, crate::cli::KeyModifiers::NONE) => self.scrollbar.next(),
                    _ => return Some(event),
                }
                _ => return Some(event),
            }
        }
        None
    }
}

impl crate::tui::Focusable for Chat {
    fn focused(&self) -> bool {
        self.focus
    }
}

impl crate::tui::Widget for Chat {
    fn width(&self) -> u16 {
        5 + self.scope.to_string().len() as u16 + self.input.width()
    }

    fn height(&self) -> u16 {
        2 + self.input.height()
    }

    fn render(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let block = ratatui::widgets::Block::bordered()
            .title(self.span(" CHAT ".to_string()));
        let [top, bottom] = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Fill(1),
            ratatui::layout::Constraint::Length(3),
        ])
            .areas(block.inner(area));
        let mut lines = Vec::new();
        for message in &self.messages {
            let scope = message.scope.to_string();
            let mut content = message.content.clone();
            let too_long = (scope.len() + message.author.len() + 5) + content.len() > (top.width - 2) as usize;
            lines.push(ratatui::text::Line::from(vec![
                ratatui::text::Span::styled("[", ratatui::style::Style::default().fg(ratatui::style::Color::White)),
                ratatui::text::Span::styled(scope, ratatui::style::Style::default().fg(message.scope.color())),
                ratatui::text::Span::styled("] ", ratatui::style::Style::default().fg(ratatui::style::Color::White)),
                ratatui::text::Span::styled(&message.author, ratatui::style::Style::default().fg(ratatui::style::Color::LightMagenta)),
                ratatui::text::Span::styled(": ", ratatui::style::Style::default().fg(ratatui::style::Color::White)),
                ratatui::text::Span::styled((if too_long { "" } else { &content }).to_string(), ratatui::style::Style::default().fg(message.scope.color())),
            ]));
            if too_long {
                while !content.is_empty() {
                    lines.push(ratatui::text::Line::from(vec![
                        ratatui::text::Span::styled(content.drain(..std::cmp::min(content.len(), (top.width - 2) as usize)).collect::<String>(), ratatui::style::Style::default().fg(message.scope.color())),
                    ]));
                }
            }
        }
        let at_end = self.scrollbar.is_at_end();
        self.scrollbar.content = lines.len();
        self.scrollbar.viewport = top.height as usize;
        if self.scrollbar.remaining() > 0 {
            let mut padding = Vec::new();
            for _ in 0..self.scrollbar.remaining() {
                padding.push(ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled("", ratatui::style::Style::default().fg(ratatui::style::Color::White)),
                ]));
            }
            padding.append(&mut lines);
            lines = padding;
            self.scrollbar.content = self.scrollbar.viewport;
        }
        if at_end {
            self.scrollbar.to_end();
        }
        ratatui::widgets::Paragraph::new(lines)
            .block(ratatui::widgets::Block::default()
                .padding(ratatui::widgets::Padding::horizontal(1)))
            .scroll((self.scrollbar.position as u16, 0))
            .render(top, buf);
        let prefix = format!("{}:", self.scope);
        let [left, right] = ratatui::layout::Layout::horizontal([
            ratatui::layout::Constraint::Length(prefix.len() as u16 + 2),
            ratatui::layout::Constraint::Fill(1),
        ])
            .areas(bottom);
        ratatui::widgets::Paragraph::new(ratatui::text::Line::from(vec![
            ratatui::text::Span::styled(prefix, ratatui::style::Style::default().fg(self.scope.color())),
        ]))
            .block(ratatui::widgets::Block::default()
                .padding(ratatui::widgets::Padding::uniform(1)))
            .render(left, buf);
        self.input.render(right, buf);
        block.render(area, buf);
        self.scrollbar.render(top, buf);
    }
}

#[derive(Default)]
pub struct ChatPage {
    pub chat: Chat,
}

impl crate::tui::NotebookPage for ChatPage {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn title(&self) -> &str {
        "CHAT"
    }
}

impl crate::tui::Widget for ChatPage {
    fn render_with_data(&mut self, _knowledge: &mut crate::tui::Knowledge, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        self.chat.render(area, buf);
    }
}


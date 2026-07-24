#[derive(Default)]
pub struct Input {
    pub buffer: String,
    handler: crate::cli::Handler,
}

impl Input {
    pub fn consume(&mut self) -> String {
        let s = self.buffer.clone();
        self.buffer.clear();
        s
    }
}

impl crate::cli::HandleEvent for Input {
    async fn handle_event(&mut self) -> Option<crate::cli::Event> {
        match self.handler.handle_event().await {
            Some(event) => match event {
                crate::cli::Event::Key { code, modifiers } => match (code, modifiers) {
                    (crate::cli::KeyCode::Char(c), _) if modifiers != crate::cli::KeyModifiers::CONTROL => {
                        self.buffer.push(c);
                        None
                    }
                    (crate::cli::KeyCode::Backspace, crate::cli::KeyModifiers::NONE) => {
                        self.buffer.pop();
                        None
                    }
                    _ => Some(event),
                }
                crate::cli::Event::Validate if self.buffer.is_empty() => None,
                _ => Some(event),
            }
            _ => None,
        }
    }
}

impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.buffer)
    }
}

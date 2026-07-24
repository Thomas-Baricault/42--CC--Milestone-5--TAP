pub enum ButtonsKind {
    Horizontal,
    Vertical,
}

pub struct Buttons {
    kind: ButtonsKind,
    buttons: Vec<crate::tui::Button>,
    handler: crate::cli::Handler,
}

impl Buttons {
    pub fn new(buttons: Vec<String>, kind: ButtonsKind) -> Self {
        Self {
            kind,
            buttons: buttons
                .into_iter()
                .map(|i| crate::tui::Button::new(&i))
                .collect(),
            handler: crate::cli::Handler::default(),
        }
    }

    pub fn selected(&self) -> Option<&String> {
        if let Some(i) = self.index() {
            Some(&self.buttons[i].text)
        } else {
            None
        }
    }

    pub fn index(&self) -> Option<usize> {
        for (i, button) in self.buttons.iter().enumerate() {
            if button.focus {
                return Some(i);
            }
        }
        None
    }

    pub fn focus(&mut self, i: usize) {
        self.buttons[i].focus = true;
    }

    pub fn unfocus(&mut self) {
        if let Some(i) = self.index() {
            self.buttons[i].focus = false;
        }
    }

    pub fn previous(&mut self) {
        let length = self.buttons.len();
        if length > 0 {
            if let Some(i) = self.index() {
                self.buttons[i].focus = false;
                if i > 0 {
                    self.buttons[i - 1].focus = true;
                } else {
                    self.buttons[length - 1].focus = true;
                }
            } else {
                self.buttons[length - 1].focus = true;
            }
        }
    }

    pub fn next(&mut self) {
        let length = self.buttons.len();
        if length > 0 {
            if let Some(i) = self.index() {
                self.buttons[i].focus = false;
                if i < length - 1 {
                    self.buttons[i + 1].focus = true;
                } else {
                    self.buttons[0].focus = true;
                }
            } else {
                self.buttons[0].focus = true;
            }
        }
    }
}

impl crate::tui::Widget for Buttons {
    fn width(&self) -> u16 {
        if self.buttons.is_empty() {
            0
        } else {
            match self.kind {
                ButtonsKind::Horizontal => {
                    let mut width = 0;
                    for button in &self.buttons {
                        width += 1 + button.width();
                    }
                    width - 1
                }
                ButtonsKind::Vertical => {
                    let mut width = 0;
                    for button in &self.buttons {
                        width = std::cmp::max(width, button.width());
                    }
                    width
                }
            }
        }
    }

    fn height(&self) -> u16 {
        match self.kind {
            ButtonsKind::Horizontal => {
                1
            }
            ButtonsKind::Vertical => {
                self.buttons.len() as u16
            }
        }
    }

    fn render(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        if self.buttons.is_empty() {
            return;
        }
        let mut areas = Vec::new();
        match self.kind {
            ButtonsKind::Horizontal => {
                for button in &self.buttons {
                    areas.push(ratatui::layout::Constraint::Length(button.width()));
                    areas.push(ratatui::layout::Constraint::Fill(1));
                }
                areas.pop();
            }
            ButtonsKind::Vertical => {
                for _ in &self.buttons {
                    areas.push(ratatui::layout::Constraint::Length(1));
                }
            }
        }
        let areas = match self.kind {
            ButtonsKind::Horizontal => ratatui::layout::Layout::horizontal(areas)
                .split(area),
            ButtonsKind::Vertical => ratatui::layout::Layout::vertical(areas)
                .split(area),
        };
        let mut i = 0;
        for button in self.buttons.iter_mut() {
            button.render(areas[i], buf);
            i += 1;
            if matches!(self.kind, ButtonsKind::Horizontal) {
                i += 1;
            }
        }
    }
}

impl crate::cli::HandleEvent for Buttons {
    async fn handle_event(&mut self) -> Option<crate::cli::Event> {
        match self.handler.handle_event().await {
            Some(event) => match event {
                crate::cli::Event::Key {
                    code: crate::cli::KeyCode::Left,
                    modifiers: crate::cli::KeyModifiers::NONE,
                } if matches!(self.kind, ButtonsKind::Horizontal) => {
                    self.previous();
                    None
                }
                crate::cli::Event::Key {
                    code: crate::cli::KeyCode::Right,
                    modifiers: crate::cli::KeyModifiers::NONE,
                } if matches!(self.kind, ButtonsKind::Horizontal) => {
                    self.next();
                    None
                }
                crate::cli::Event::Key {
                    code: crate::cli::KeyCode::Up,
                    modifiers: crate::cli::KeyModifiers::NONE,
                } if matches!(self.kind, ButtonsKind::Vertical) => {
                    self.previous();
                    None
                }
                crate::cli::Event::Key {
                    code: crate::cli::KeyCode::Down,
                    modifiers: crate::cli::KeyModifiers::NONE,
                } if matches!(self.kind, ButtonsKind::Vertical) => {
                    self.next();
                    None
                }
                crate::cli::Event::Validate if self.index().is_none() => None,
                _ => Some(event),
            }
            None => None,
        }
    }
}

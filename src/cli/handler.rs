use futures::StreamExt;

pub type KeyCode = crossterm::event::KeyCode;
pub type KeyModifiers = crossterm::event::KeyModifiers;

pub enum Event {
    Interrupted,
    Key {
        code: KeyCode,
        modifiers: KeyModifiers,
    },
    Validate,
}

pub trait HandleEvent {
    fn handle_event(&mut self) -> impl std::future::Future<Output = Option<Event>>;
}

pub struct Handler {
    events: crossterm::event::EventStream,
}

impl Handler {
    pub fn init() {
        let _ = crossterm::terminal::enable_raw_mode();
    }

    pub fn cleanup() {
        let _ = crossterm::terminal::disable_raw_mode();
    }
}

impl HandleEvent for Handler {
    async fn handle_event(&mut self) -> Option<Event> {
        match self.events.next().await {
            Some(Ok(event)) => match event {
                crossterm::event::Event::Key(key) if key.kind == crossterm::event::KeyEventKind::Press => match (key.code, key.modifiers) {
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(Event::Interrupted),
                    (KeyCode::Enter, crate::cli::KeyModifiers::NONE) => Some(Event::Validate),
                    (_, _) => Some(Event::Key {
                        code: key.code,
                        modifiers: key.modifiers,
                    }),
                }
                _ => None,
            }
            _ => None,
        }
    }
}

impl Default for Handler {
    fn default() -> Self {
        Self {
            events: crossterm::event::EventStream::new(),
        }
    }
}

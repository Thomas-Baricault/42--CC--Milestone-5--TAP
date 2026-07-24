use ratatui::prelude::StatefulWidget;

use crate::tui::Focusable;

pub trait ToListItem: Sized {
    fn items_from_iter<T: IntoIterator<Item: AsRef<str>>>(v: T) -> Vec<ListItem<Self>> {
        v
            .into_iter()
            .map(|i| crate::tui::ListItem::Unknown(i.as_ref().to_string()))
            .collect()
    }

    fn update_item(_knowledge: &crate::tui::Knowledge, _s: &str) -> Option<Self> {
        None
    }

    fn to_item(&self, knowledge: &crate::tui::Knowledge) -> ratatui::widgets::ListItem<'static>;
}

impl ToListItem for String {
    fn update_item(_knowledge: &crate::tui::Knowledge, s: &str) -> Option<Self> {
        Some(s.to_string())
    }

    fn to_item(&self, _knowledge: &crate::tui::Knowledge) -> ratatui::widgets::ListItem<'static> {
        ratatui::widgets::ListItem::new(self.clone())
    }
}

pub enum ListItem<T: ToListItem> {
    Known(T),
    Unknown(String),
}

pub struct List<T: ToListItem> {
    pub title: String,
    pub items: Vec<ListItem<T>>,
    pub focus: bool,
    handler: crate::cli::Handler,
    state: ratatui::widgets::ListState,
    scrollbar: crate::tui::Scrollbar,
}

impl<T: ToListItem> List<T> {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            items: Vec::new(),
            focus: false,
            handler: crate::cli::Handler::default(),
            state: ratatui::widgets::ListState::default(),
            scrollbar: crate::tui::Scrollbar::default(),
        }
    }

    pub fn index(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn selected(&self) -> Option<&T> {
        match self.state.selected() {
            Some(i) => match &self.items[i] {
                ListItem::Known(v) => Some(v),
                ListItem::Unknown(_) => None,
            }
            None => None,
        }
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn previous(&mut self) {
        let len = self.items.len();
        if len == 0 {
            return;
        }
        if let Some(mut i) = self.state.selected() {
            if i == 0 {
                i = len;
            } else {
                i -= 1;
            }
            self.state.select(Some(i));
            self.scrollbar.center(i);
        } else {
            self.state.select(Some(len - 1));
            self.scrollbar.center(len - 1);
        }
    }

    pub fn next(&mut self) {
        let len = self.items.len();
        if len == 0 {
            return;
        }
        if let Some(mut i) = self.state.selected() {
            if i == len - 1 {
                i = 0;
            } else {
                i += 1
            }
            self.state.select(Some(i));
            self.scrollbar.center(i);
        } else {
            self.state.select(Some(0));
            self.scrollbar.center(0);
        }
    }
}

impl<T: ToListItem> crate::cli::HandleEvent for List<T> {
    async fn handle_event(&mut self) -> Option<crate::cli::Event> {
        if let Some(event) = self.handler.handle_event().await {
            match event {
                crate::cli::Event::Key {
                    code: crate::cli::KeyCode::Up,
                    modifiers: crate::cli::KeyModifiers::NONE,
                } => self.previous(),
                crate::cli::Event::Key {
                    code: crate::cli::KeyCode::Down,
                    modifiers: crate::cli::KeyModifiers::NONE,
                } => self.next(),
                crate::cli::Event::Validate if self.state.selected().is_none() => (),
                _ => return Some(event),
            }
        }
        None
    }
}

impl<T: ToListItem> crate::tui::Focusable for List<T> {
    fn focused(&self) -> bool {
        self.focus
    }
}

impl<T: ToListItem> crate::tui::Widget for List<T> {
    fn render_with_data(&mut self, knowledge: &mut crate::tui::Knowledge, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        ratatui::widgets::List::new(self.items
            .iter_mut()
            .map(|i| {
                if let ListItem::Unknown(s) = i {
                    if let Some(v) = T::update_item(knowledge, s) {
                        *i = ListItem::Known(v);
                    } else {
                        knowledge.describes.insert(s.clone());
                    }
                }
                match i {
                    ListItem::Known(v) => v.to_item(knowledge),
                    ListItem::Unknown(s) => ratatui::widgets::ListItem::new(format!("{{{s}}}")),
                }
            })
            .collect::<Vec<ratatui::widgets::ListItem>>()
        )
            .block(ratatui::widgets::Block::bordered()
                .title(self.span(format!(" {} ", self.title))))
            .highlight_symbol("> ")
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
            .render(area, buf, &mut self.state);
        self.scrollbar.viewport = area.height.saturating_sub(2) as usize;
        self.scrollbar.content = self.items.len();
        self.scrollbar.render(area, buf);
    }
}

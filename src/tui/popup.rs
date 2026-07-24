use ratatui::prelude::Widget;

trait PopupWidget: crate::tui::Widget {
    fn title(&self) -> &str;

    fn content_width(&self) -> u16;

    fn content_height(&self) -> u16;

    fn render_layout(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) -> ratatui::layout::Rect {
        let [_, area, _] = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Fill(1),
            ratatui::layout::Constraint::Length(4 + std::cmp::min(self.content_height(), area.height.saturating_sub(4))),
            ratatui::layout::Constraint::Fill(1),
        ])
            .areas(area);
        let [_, area, _] = ratatui::layout::Layout::horizontal([
            ratatui::layout::Constraint::Fill(1),
            ratatui::layout::Constraint::Length(6 + std::cmp::min(self.content_width(), area.width.saturating_sub(6))),
            ratatui::layout::Constraint::Fill(1),
        ])
            .areas(area);
        ratatui::widgets::Clear
            .render(area, buf);
        let block = ratatui::widgets::Block::bordered()
            .title(if self.title().is_empty() { String::new() } else { format!(" {} ", self.title()) })
            .padding(ratatui::widgets::Padding::symmetric(2, 1));
        let inner = block.inner(area);
        block.render(area, buf);
        inner
    }

    fn render_content(&mut self, _area: ratatui::layout::Rect, _buf: &mut ratatui::buffer::Buffer) {}

    fn render_content_with_data(&mut self, _knowledge: &mut crate::tui::Knowledge, _area: ratatui::layout::Rect, _buf: &mut ratatui::buffer::Buffer) {}
}

impl<T: PopupWidget> crate::tui::Widget for T {
    fn render(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let area = self.render_layout(area, buf);
        self.render_content(area, buf);
    }

    fn render_with_data(&mut self, knowledge: &mut crate::tui::Knowledge, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let area = self.render_layout(area, buf);
        self.render_content_with_data(knowledge, area, buf);
    }
}

pub struct PopupAction {
    pub id: &'static str,
    pub title: String,
    pub buttons: crate::tui::Buttons,
}

impl PopupAction {
    pub fn new(id: &'static str, title: &str, actions: Vec<String>) -> Self {
        Self {
            id,
            title: title.to_string(),
            buttons: crate::tui::Buttons::new(
                actions,
                crate::tui::ButtonsKind::Vertical,
            ),
        }
    }
}

impl PopupWidget for PopupAction {
    fn title(&self) -> &str {
        &self.title
    }

    fn content_width(&self) -> u16 {
        use crate::tui::Widget;
        std::cmp::max(
            self.buttons.width(),
            2,
        )
    }

    fn content_height(&self) -> u16 {
        use crate::tui::Widget;
        self.buttons.height()
    }

    fn render_content(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        use crate::tui::Widget;
        self.buttons.render(area, buf);
    }
}

impl crate::cli::HandleEvent for PopupAction {
    async fn handle_event(&mut self) -> Option<crate::cli::Event> {
        self.buttons.handle_event().await
    }
}

pub struct PopupAsk {
    pub id: &'static str,
    pub title: String,
    pub text: String,
    pub yes: crate::tui::Button,
    pub no: crate::tui::Button,
}

impl PopupAsk {
    pub fn new(id: &'static str, title: &str, text: &str) -> Self {
        Self {
            id,
            title: title.to_string(),
            text: text.to_string(),
            yes: crate::tui::Button::new("Yes"),
            no: crate::tui::Button::new("No"),
        }
    }

    pub fn selected(&self) -> Option<bool> {
        if self.yes.focus {
            Some(true)
        } else if self.no.focus {
            Some(false)
        } else {
            None
        }
    }
}

impl PopupWidget for PopupAsk {
    fn title(&self) -> &str {
        &self.title
    }

    fn content_width(&self) -> u16 {
        use crate::tui::Widget;
        std::cmp::max(
            self.text.len() as u16,
            self.yes.width() + 1 + self.no.width(),
        )
    }

    fn content_height(&self) -> u16 {
        3
    }

    fn render_content(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        use crate::tui::Widget;
        let [top, _, bottom] = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
        ])
            .areas(area);
        let [left, _, right] = ratatui::layout::Layout::horizontal([
            ratatui::layout::Constraint::Length(self.yes.width()),
            ratatui::layout::Constraint::Fill(1),
            ratatui::layout::Constraint::Length(self.no.width()),
        ])
            .areas(bottom);
        ratatui::widgets::Paragraph::new(self.text.as_str())
            .centered()
            .render(top, buf);
        self.yes.render(left, buf);
        self.no.render(right, buf);
    }
}

impl crate::cli::HandleEvent for PopupAsk {
    async fn handle_event(&mut self) -> Option<crate::cli::Event> {
        let event = if self.yes.focus {
            self.yes.handle_event().await
        } else {
            self.no.handle_event().await
        };
        match event {
            Some(event) => match event {
                crate::cli::Event::Key {
                    code: crate::cli::KeyCode::Left,
                    modifiers: crate::cli::KeyModifiers::NONE,
                } => {
                    if !self.yes.focus && !self.no.focus {
                        self.no.focus = true;
                    } else {
                        self.yes.focus = !self.yes.focus;
                        self.no.focus = !self.no.focus;
                    }
                    None
                }
                crate::cli::Event::Key {
                    code: crate::cli::KeyCode::Right,
                    modifiers: crate::cli::KeyModifiers::NONE,
                } => {
                    if !self.yes.focus && !self.no.focus {
                        self.yes.focus = true;
                    } else {
                        self.yes.focus = !self.yes.focus;
                        self.no.focus = !self.no.focus;
                    }
                    None
                }
                crate::cli::Event::Validate if !self.yes.focus && !self.no.focus => None,
                _ => Some(event),
            }
            None => None,
        }
    }
}

pub trait PopupDescribeInfos: crate::tui::Widget {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    fn title(&self) -> &str;
}

pub struct PopupDescribe {
    pub id: &'static str,
    pub buttons: crate::tui::Buttons,
    b_return: crate::tui::Button,
    data: Box<dyn PopupDescribeInfos>,
    handler: crate::cli::Handler,
}

impl PopupDescribe {
    pub fn new(id: &'static str, data: Box<dyn PopupDescribeInfos>, actions: Vec<String>) -> Self {
        Self {
            id,
            buttons: crate::tui::Buttons::new(
                actions,
                crate::tui::ButtonsKind::Horizontal,
            ),
            b_return: crate::tui::Button::new("Return"),
            data,
            handler: crate::cli::Handler::default(),
        }
    }

    pub fn data<T: 'static>(&mut self) -> &mut T {
        self.data
            .as_any_mut()
            .downcast_mut::<T>()
            .unwrap()
    }
}

impl PopupWidget for PopupDescribe {
    fn title(&self) -> &str {
        self.data.title()
    }

    fn content_width(&self) -> u16 {
        use crate::tui::Widget;
        std::cmp::max(
            std::cmp::max(
                self.data.width(),
                self.buttons.width(),
            ),
            8,
        )
    }

    fn content_height(&self) -> u16 {
        self.data.height() + 4
    }

    fn render_content_with_data(&mut self, knowledge: &mut crate::tui::Knowledge, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let [top, _, middle, _, bottom] = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Fill(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
        ])
            .areas(area);
        use crate::tui::Widget;
        self.b_return.render(top, buf);
        self.data.render_with_data(knowledge, middle, buf);
        self.buttons.render(bottom, buf);
    }
}

impl crate::cli::HandleEvent for PopupDescribe {
    async fn handle_event(&mut self) -> Option<crate::cli::Event> {
        let event = if self.b_return.focus {
            self.handler.handle_event().await
        } else {
            self.buttons.handle_event().await
        };
        if let Some(crate::cli::Event::Key {
            code: crate::cli::KeyCode::Up | crate::cli::KeyCode::Down,
            modifiers: crate::cli::KeyModifiers::NONE
        }) = event {
            self.b_return.focus = !self.b_return.focus;
            if self.b_return.focus {
                self.buttons.unfocus();
            } else {
                self.buttons.focus(0);
            }
            None
        } else {
            event
        }
    }
}

pub struct PopupError {
    title: String,
    error: String,
    button: crate::tui::Button,
    handler: crate::cli::Handler,
}

impl PopupError {
    pub fn new(error: crate::messages::Error) -> Self {
        Self {
            title: format!("ERR {}", error.code()),
            error: error.message().to_string(),
            button: crate::tui::Button::new("Ok"),
            handler: crate::cli::Handler::default(),
        }
    }
}

impl PopupWidget for PopupError {
    fn title(&self) -> &str {
        &self.title
    }

    fn content_width(&self) -> u16 {
        std::cmp::max(
            std::cmp::max(
                self.title.len() as u16,
                self.error.len() as u16,
            ),
            2,
        )
    }

    fn content_height(&self) -> u16 {
        3
    }

    fn render_content(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let [top, _, bottom] = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
        ])
            .areas(area);
        ratatui::widgets::Paragraph::new(self.error.as_str())
            .centered()
            .render(top, buf);
        use crate::tui::Widget;
        self.button.focus = true;
        self.button.render(bottom, buf);
    }
}

impl crate::cli::HandleEvent for PopupError {
    async fn handle_event(&mut self) -> Option<crate::cli::Event> {
        let event = self.handler.handle_event().await;
        if let Some(crate::cli::Event::Validate) = event {
            event
        } else {
            None
        }
    }
}

pub struct PopupInfo {
    title: String,
    info: String,
    handler: crate::cli::Handler,
}

impl PopupInfo {
    pub fn new(title: &str, info: &str) -> Self {
        Self {
            title: title.to_string(),
            info: info.to_string(),
            handler: crate::cli::Handler::default(),
        }
    }
}

impl PopupWidget for PopupInfo {
    fn title(&self) -> &str {
        &self.title
    }

    fn content_width(&self) -> u16 {
        std::cmp::max(
            std::cmp::max(
                self.title.len() as u16,
                self.info.len() as u16,
            ),
            2,
        )
    }

    fn content_height(&self) -> u16 {
        1
    }

    fn render_content(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        ratatui::widgets::Paragraph::new(self.info.as_str())
            .centered()
            .render(area, buf);
    }
}

impl crate::cli::HandleEvent for PopupInfo {
    async fn handle_event(&mut self) -> Option<crate::cli::Event> {
        let event = self.handler.handle_event().await;
        if let Some(crate::cli::Event::Validate) = event {
            event
        } else {
            None
        }
    }
}

pub struct PopupInput {
    pub id: &'static str,
    pub input: crate::tui::Input,
    title: String,
}

impl PopupInput {
    pub fn new(id: &'static str, title: &str) -> Self {
        Self {
            id,
            title: title.to_string(),
            input: crate::tui::Input::new(30, 30),
        }
    }
}

impl PopupWidget for PopupInput {
    fn title(&self) -> &str {
        &self.title
    }

    fn content_width(&self) -> u16 {
        use crate::tui::Widget;
        std::cmp::max(
            self.title.len() as u16,
            self.input.width(),
        )
    }

    fn content_height(&self) -> u16 {
        use crate::tui::Widget;
        self.input.height()
    }

    fn render_content(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        use crate::tui::widget::Widget;
        self.input.render(area, buf);
    }
}

impl crate::cli::HandleEvent for PopupInput {
    async fn handle_event(&mut self) -> Option<crate::cli::Event> {
        self.input.handle_event().await
    }
}

pub enum Popup {
    Action(PopupAction),
    Ask(PopupAsk),
    Describe(PopupDescribe),
    Error(PopupError),
    Info(PopupInfo),
    Input(PopupInput),
}

impl Popup {
    pub fn action(id: &'static str, title: &str, actions: Vec<String>) -> Self {
        Self::Action(
            PopupAction::new(id, title, actions),
        )
    }
    pub fn ask(id: &'static str, title: &str, text: &str) -> Self {
        Self::Ask(
            PopupAsk::new(id, title, text),
        )
    }

    pub fn describe(id: &'static str, data: Box<dyn PopupDescribeInfos>, actions: Vec<String>) -> Self {
        Self::Describe(
            PopupDescribe::new(id, data, actions),
        )
    }

    pub fn error(error: crate::messages::Error) -> Self {
        Self::Error(
            PopupError::new(error),
        )
    }

    pub fn info(title: &str, info: &str) -> Self {
        Self::Info(
            PopupInfo::new(title, info),
        )
    }

    pub fn input(id: &'static str, title: &str) -> Self {
        Self::Input(
            PopupInput::new(id, title),
        )
    }
}

impl crate::cli::HandleEvent for Popup {
    async fn handle_event(&mut self) -> Option<crate::cli::Event> {
        match self {
            Popup::Action(popup) => popup.handle_event().await,
            Popup::Ask(popup) => popup.handle_event().await,
            Popup::Describe(popup) => popup.handle_event().await,
            Popup::Error(popup) => popup.handle_event().await,
            Popup::Info(popup) => popup.handle_event().await,
            Popup::Input(popup) => popup.handle_event().await,
        }
    }
}

impl crate::tui::Widget for Popup {
    fn render_with_data(&mut self, knowledge: &mut crate::tui::Knowledge, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        match self {
            Popup::Action(popup) => popup.render(area, buf),
            Popup::Ask(popup) => popup.render(area, buf),
            Popup::Describe(popup) => popup.render_with_data(knowledge, area, buf),
            Popup::Error(popup) => popup.render(area, buf),
            Popup::Info(popup) => popup.render(area, buf),
            Popup::Input(popup) => popup.render(area, buf),
        }
    }
}

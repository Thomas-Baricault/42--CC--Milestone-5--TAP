use ratatui::prelude::Widget;

pub trait NotebookPage: std::any::Any + crate::tui::Widget {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    fn title(&self) -> &str;
}

#[derive(Default)]
pub struct Notebook {
    pub current: usize,
    pages: Vec<Box<dyn NotebookPage>>,
}

impl Notebook {
    pub fn new(pages: Vec<Box<dyn NotebookPage>>) -> Self {
        Self {
            current: 0,
            pages,
        }
    }

    pub fn page<T: 'static>(&mut self, i: usize) -> &mut T {
        self.pages[i]
            .as_any_mut()
            .downcast_mut::<T>()
            .unwrap()
    }

    fn render_layout(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) -> ratatui::layout::Rect {
        let [top, bottom] = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Length(3),
            ratatui::layout::Constraint::Fill(1),
        ])
            .areas(area);
        if self.pages.is_empty() {
            ratatui::widgets::Block::bordered()
                .borders(ratatui::widgets::Borders::LEFT | ratatui::widgets::Borders::RIGHT | ratatui::widgets::Borders::TOP)
                .render(top, buf);
        } else {
            let areas = ratatui::layout::Layout::horizontal(vec![
                ratatui::layout::Constraint::Fill(1); self.pages.len()
            ])
                .split(top);
            for i in 0..self.pages.len() {
                let mut flag = ratatui::widgets::Borders::TOP | ratatui::widgets::Borders::RIGHT;
                if i == 0 {
                    flag |= ratatui::widgets::Borders::LEFT;
                }
                if i != self.current {
                    flag |= ratatui::widgets::Borders::BOTTOM;
                }
                ratatui::widgets::Paragraph::new(self.pages[i].title())
                    .block(ratatui::widgets::Block::bordered()
                        .padding(ratatui::widgets::Padding::horizontal(1))
                        .borders(flag))
                    .centered()
                    .render(areas[i], buf);
            }
        }
        let block = ratatui::widgets::Block::bordered()
            .borders(ratatui::widgets::Borders::LEFT | ratatui::widgets::Borders::RIGHT | ratatui::widgets::Borders::BOTTOM);
        let inner = block.inner(bottom);
        block.render(bottom, buf);
        inner
    }
}

impl crate::tui::Widget for Notebook {
    fn render(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let area = self.render_layout(area, buf);
        if !self.pages.is_empty() {
            self.pages[self.current].render(area, buf);
        }
    }

    fn render_with_data(&mut self, knowledge: &mut crate::tui::Knowledge, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let area = self.render_layout(area, buf);
        if !self.pages.is_empty() {
            self.pages[self.current].render_with_data(knowledge, area, buf);
        }
    }
}

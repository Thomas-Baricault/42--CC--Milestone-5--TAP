use ratatui::prelude::StatefulWidget;

#[derive(Default)]
pub struct Scrollbar {
    pub content: usize,
    pub viewport: usize,
    pub position: usize,
}

impl Scrollbar {
    pub fn is_at_begin(&self) -> bool {
        self.position == 0
    }

    pub fn is_at_end(&self) -> bool {
        if self.content > self.viewport {
            self.position == self.content - self.viewport
        } else {
            true
        }
    }

    pub fn remaining(&self) -> usize {
        self.viewport.saturating_sub(self.content)
    }

    pub fn overflow(&self) -> usize {
        self.content.saturating_sub(self.viewport)
    }

    pub fn to_begin(&mut self) {
        self.position = 0;
    }

    pub fn to_end(&mut self) {
        if self.content > self.viewport {
            self.position = self.content - self.viewport;
        } else {
            self.position = 0;
        }
    }

    pub fn previous(&mut self) {
        if self.position > 0 {
            self.position -= 1;
        }
    }

    pub fn next(&mut self) {
        if self.content > self.viewport {
            self.position = std::cmp::min(self.position + 1, self.content - self.viewport);
        }
    }

    pub fn center(&mut self, i: usize) {
        if self.content > self.viewport {
            if i <= self.viewport / 2 {
                self.position = 0;
            } else {
                self.position = std::cmp::min(i - self.viewport / 2, self.content - self.viewport);
            }
        }
    }
}

impl crate::tui::Widget for Scrollbar {
    fn render(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        if self.content <= self.viewport {
            self.position = 0;
        } else {
            self.position = std::cmp::min(self.position, self.content - self.viewport);
        }
        let mut state = ratatui::widgets::ScrollbarState::default()
            .content_length(self.content.saturating_sub(self.viewport))
            .viewport_content_length(self.viewport)
            .position(self.position);
        ratatui::widgets::Scrollbar::default()
            .orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight)
            .render(area, buf, &mut state);
    }
}

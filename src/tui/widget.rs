pub trait Widget {
    fn width(&self) -> u16 {
        0
    }

    fn height(&self) -> u16 {
        0
    }

    fn render(&mut self, _area: ratatui::layout::Rect, _buf: &mut ratatui::buffer::Buffer) {}

    fn render_with_data(&mut self, _knowledge: &mut crate::tui::Knowledge, _area: ratatui::layout::Rect, _buf: &mut ratatui::buffer::Buffer) {}
}

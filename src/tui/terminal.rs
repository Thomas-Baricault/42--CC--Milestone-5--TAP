pub struct Terminal {
    tui: ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
}

impl Terminal {
    pub fn new() -> Result<Self, std::io::Error> {
        let mut stdout = std::io::stdout();
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
        Ok(Self {
            tui: ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(stdout))?,
        })
    }

    pub fn close(&mut self) -> Result<(), std::io::Error> {
        crossterm::execute!(self.tui.backend_mut(), crossterm::terminal::LeaveAlternateScreen)?;
        self.tui.show_cursor()?;
        crossterm::terminal::disable_raw_mode()?;
        Ok(())
    }

    pub fn update<T, F: Fn(T, &mut ratatui::Frame)>(&mut self, obj: T, ui: F) {
        let _ = self.tui.draw(|f| ui(obj, f));
    }
}

pub struct FloodDetector {
    history: std::collections::VecDeque<std::time::Instant>,
    limit: usize,
    window: std::time::Duration,
}

impl FloodDetector {
    pub fn new(limit: usize, secs: u64) -> Self {
        Self {
            history: std::collections::VecDeque::new(),
            limit,
            window: std::time::Duration::from_secs(secs),
        }
    }

    pub fn record(&mut self) -> bool {
        let now = std::time::Instant::now();
        while let Some(&t) = self.history.front() {
            if now.duration_since(t) > self.window {
                self.history.pop_front();
            } else {
                break;
            }
        }
        self.history.push_back(now);
        let flood = self.history.len() > self.limit;
        while self.history.len() > self.limit {
            self.history.pop_front();
        }
        flood
    }
}

#[derive(Default)]
pub struct Waiter {
    instant: Option<tokio::time::Instant>,
    seconds: u64,
}

impl Waiter {
    pub async fn block() {
        std::future::pending::<()>().await
    }

    pub fn is_waiting(&self) -> bool {
        self.instant.is_some()
    }

    pub fn begin(&mut self, seconds: u64) {
        self.instant = Some(tokio::time::Instant::now());
        self.seconds = seconds;
    }

    pub fn end(&mut self) {
        self.instant = None;
    }

    pub async fn wait(&mut self) {
        match self.instant {
            Some(start) => tokio::time::sleep_until(start + tokio::time::Duration::from_secs(self.seconds)).await,
            None => Self::block().await,
        };
        self.instant = None;
    }
}

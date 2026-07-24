pub struct Shared<T>(std::sync::Arc<tokio::sync::Mutex<T>>);

impl<T> Shared<T> {
    pub fn new(data: T) -> Self {
        Self(
            std::sync::Arc::new(
                tokio::sync::Mutex::new(
                    data
                )
            )
        )
    }

    pub async fn lock(&self) -> tokio::sync::MutexGuard<'_, T> {
        self.0.lock().await
    }
}

impl<T> From<T> for Shared<T> {
    fn from(data: T) -> Self {
        Self::new(data)
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self(
            std::sync::Arc::clone(
                &self.0
            )
        )
    }
}

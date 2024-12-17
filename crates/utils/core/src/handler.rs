use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Initializable event handler error: {0}")]
    EventHandler(String),
}

#[async_trait]
pub trait InitializableEventHandler {
    async fn init_event_handler(&self)
        -> Option<tokio::sync::oneshot::Receiver<Result<(), Error>>>;
}

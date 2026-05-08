#[derive(Debug, thiserror::Error)]
pub enum MessagingError {
    #[error("Connection error: {0}")]
    UncnownProvider(String),
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Publish error: {0}")]
    Publish(String),
    #[error("Subscribe error: {0}")]
    Subscribe(String),
}

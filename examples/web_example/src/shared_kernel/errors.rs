use event_hex::shared_kernel::errors::EventHexError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Event hexagonal error: {0}")]
    EventHex(#[from] EventHexError),

    #[error("MongoDB error: {0}")]
    MongoError(#[from] mongodb::error::Error),
}
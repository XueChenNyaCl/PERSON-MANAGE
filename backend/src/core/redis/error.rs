use thiserror::Error;

#[derive(Error, Debug)]
pub enum RedisError {
    #[error("Redis connection error: {0}")]
    ConnectionError(String),

    #[error("Redis operation timeout")]
    TimeoutError,

    #[error("Redis operation error: {0}")]
    OperationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Redis not available")]
    NotAvailable,

    #[error("Buffer full")]
    BufferFull,

    #[allow(dead_code)]
    #[error("Rate limited")]
    RateLimited,
}

impl From<RedisError> for crate::core::error::AppError {
    fn from(err: RedisError) -> Self {
        match err {
            RedisError::NotAvailable => crate::core::error::AppError::InternalWithMessage(
                "Cache service temporarily unavailable".to_string(),
            ),
            RedisError::RateLimited => crate::core::error::AppError::InternalWithMessage(
                "Too many requests, please try again later".to_string(),
            ),
            RedisError::BufferFull => crate::core::error::AppError::InternalWithMessage(
                "Write buffer is full, please try again later".to_string(),
            ),
            _ => crate::core::error::AppError::InternalWithMessage(err.to_string()),
        }
    }
}

impl From<redis::RedisError> for RedisError {
    fn from(err: redis::RedisError) -> Self {
        if err.is_timeout() {
            RedisError::TimeoutError
        } else if err.is_io_error() {
            RedisError::ConnectionError(err.to_string())
        } else {
            RedisError::OperationError(err.to_string())
        }
    }
}

pub type Result<T> = std::result::Result<T, RedisError>;

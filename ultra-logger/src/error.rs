//! Error types for the ultra-low latency logging system

use thiserror::Error;

pub type Result<T> = std::result::Result<T, LoggingError>;

#[derive(Error, Debug)]
pub enum LoggingError {
    #[error("Logger already initialized")]
    AlreadyInitialized,
    
    #[error("Invalid log level: {0}")]
    InvalidLogLevel(String),
    
    #[error("Buffer error: {0}")]
    BufferError(String),
    
    #[error("Transport error: {0}")]
    TransportError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Compression error: {0}")]
    CompressionError(String),
    
    #[error("Channel send error")]
    ChannelSendError,
    
    #[error("Channel receive error")]
    ChannelReceiveError,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Metrics error: {0}")]
    MetricsError(String),
    
    #[error("Tracing error: {0}")]
    TracingError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Timeout error: operation timed out after {0}ms")]
    TimeoutError(u64),
    
    #[error("Memory allocation error")]
    MemoryError,
    
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl LoggingError {
    pub fn is_recoverable(&self) -> bool {
        matches!(self, 
            Self::TransportError(_) |
            Self::NetworkError(_) |
            Self::TimeoutError(_) |
            Self::IoError(_)
        )
    }
    
    pub fn should_retry(&self) -> bool {
        matches!(self,
            Self::TransportError(_) |
            Self::NetworkError(_) |
            Self::TimeoutError(_) |
            Self::ChannelSendError
        )
    }
}

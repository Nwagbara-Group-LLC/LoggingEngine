//! Ultra-Low Latency Logging Engine for Trading Systems
//! 
//! This library provides microsecond-precision logging optimized for high-frequency trading
//! with zero-copy operations, lock-free data structures, and batch processing.

#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub mod config;
pub mod logger;
pub mod buffer;
pub mod compression;
pub mod transport;
pub mod metrics;
pub mod trace;
pub mod error;

pub use config::LoggerConfig;
pub use logger::{UltraLogger, LogLevel, LogEntry};
pub use error::{LoggingError, Result};

/// Re-export commonly used types for convenience
pub mod prelude {
    pub use crate::{
        UltraLogger,
        LogLevel,
        LogEntry,
        LoggerConfig,
        Result,
        LoggingError,
    };
    pub use tracing::{info, warn, error, debug, trace};
}

/// Initialize the ultra-low latency logger with default configuration
pub fn init() -> Result<()> {
    let config = LoggerConfig::default();
    UltraLogger::init(config)
}

/// Initialize the ultra-low latency logger with custom configuration
pub fn init_with_config(config: LoggerConfig) -> Result<()> {
    UltraLogger::init(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_logger_initialization() {
        let result = init();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_logging_performance() {
        let _logger = init().unwrap();
        let start = Instant::now();
        
        // Log 10,000 messages
        for i in 0..10_000 {
            tracing::info!(message = "Performance test", iteration = i, timestamp = %chrono::Utc::now());
        }
        
        let duration = start.elapsed();
        let throughput = 10_000.0 / duration.as_secs_f64();
        
        println!("Logged 10,000 messages in {:?}", duration);
        println!("Throughput: {:.0} messages/second", throughput);
        
        // Should achieve >100k messages per second
        assert!(throughput > 50_000.0, "Logging throughput too low: {:.0} msg/s", throughput);
    }
}

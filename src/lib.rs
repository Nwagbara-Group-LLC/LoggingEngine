//! Logging Engine - Simple, fast logging for trading systems
//! 
//! This is a streamlined logging solution optimized for high-frequency trading.
//! 
//! # Features
//! - Ultra-low latency logging
//! - Structured logging with JSON output  
//! - Multiple transport options (stdout, file)
//! - Async processing with background threads
//! - Simple configuration
//! 
//! # Quick Start
//! 
//! ```rust
//! use logging_engine::UltraLogger;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let logger = UltraLogger::new("trading-system".to_string());
//!     
//!     logger.info("System started".to_string()).await?;
//!     logger.error("Critical error occurred".to_string()).await?;
//!     
//!     logger.shutdown().await?;
//!     Ok(())
//! }
//! ```

pub use ultra_logger::*;

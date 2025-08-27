//! Log Aggregator Binary
//! 
//! Standalone service for high-throughput log aggregation
//! Collects, batches, and forwards log entries from multiple sources

use anyhow::Result;
use log_aggregator::{AggregatorConfig, LogAggregator, Transport};
use std::time::Duration;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    let logger = ultra_logger::UltraLogger::new("log-aggregator".to_string());

    let _ = logger.info("Starting Log Aggregator Service...".to_string()).await;

    // Create configuration
    let config = AggregatorConfig {
        batch_size: 1000,
        batch_timeout: Duration::from_millis(100),
        max_memory_usage: 100 * 1024 * 1024, // 100MB
        output_transport: Transport::Memory,
        filters: Vec::new(),
    };

    // Create and start the log aggregator
    let aggregator = LogAggregator::new(config)?;
    aggregator.start().await?;

    let _ = logger.info("Log Aggregator Service started successfully!".to_string()).await;
    println!("Press Ctrl+C to shutdown...");

    // Wait for shutdown signal
    signal::ctrl_c().await?;
    
    let _ = logger.info("Shutdown signal received, stopping log aggregator...".to_string()).await;
    aggregator.stop().await?;
    let _ = logger.info("Log Aggregator Service stopped successfully!".to_string()).await;

    Ok(())
}
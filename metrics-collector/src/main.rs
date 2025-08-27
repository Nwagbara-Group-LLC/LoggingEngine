//! Metrics Collector Binary
//! 
//! Standalone service for high-performance metrics collection
//! Optimized for trading applications with ultra-low latency requirements

use anyhow::Result;
use metrics_collector::{MetricsCollector, MetricsConfig};
use std::time::Duration;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    let logger = ultra_logger::UltraLogger::new("metrics-collector".to_string());

    let _ = logger.info("Starting Metrics Collector Service...".to_string()).await;

    // Create configuration optimized for trading workloads
    let config = MetricsConfig {
        buffer_size: 10_000,
        flush_interval: Duration::from_millis(50), // High frequency for trading
        retention_time: Duration::from_secs(300), // 5 minutes
        high_precision: true,
        max_concurrent: 200, // High concurrency for trading systems
    };

    // Create and start the metrics collector
    let collector = MetricsCollector::with_config(config).await?;
    collector.start().await?;

    let _ = logger.info("Metrics Collector Service started successfully!".to_string()).await;
    let _ = logger.info("Configuration: High-precision timestamps enabled, 50ms flush interval".to_string()).await;
    println!("Press Ctrl+C to shutdown...");

    // Wait for shutdown signal
    signal::ctrl_c().await?;
    
    let _ = logger.info("Shutdown signal received, stopping metrics collector...".to_string()).await;
    collector.stop().await?;
    let _ = logger.info("Metrics Collector Service stopped successfully!".to_string()).await;

    Ok(())
}
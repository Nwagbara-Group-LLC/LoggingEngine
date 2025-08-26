mod test_ultra_logger;
mod test_log_aggregator;
mod test_metrics_collector;

use ultra_logger::LogLevel;
use log_aggregator::{LogAggregator, AggregatorConfig};
use metrics_collector::{MetricsCollector, MetricsConfig};
use std::time::Duration;

#[tokio::test]
async fn test_basic_integration() {
    println!("Running basic integration test");
    
    // Test basic component creation
    let aggregator_config = AggregatorConfig::default();
    let aggregator = LogAggregator::new(aggregator_config)
        .expect("Failed to create aggregator");

    let metrics_config = MetricsConfig::default();
    let metrics_collector = MetricsCollector::new(metrics_config)
        .expect("Failed to create metrics collector");

    // Start components
    aggregator.start().await.expect("Failed to start aggregator");
    metrics_collector.start().await.expect("Failed to start metrics collector");

    // Wait for initialization
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Simulate some basic operations
    aggregator.process_log_entry("INFO", "integration_test", "Test message").await;
    metrics_collector.record_counter("integration.test", 1.0, &[("status", "success")]).await;

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Stop components
    aggregator.stop().await.expect("Failed to stop aggregator");
    metrics_collector.stop().await.expect("Failed to stop metrics collector");
    
    println!("Basic integration test completed successfully");
}

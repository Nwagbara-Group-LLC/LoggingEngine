use log_aggregator::LogAggregator;
use metrics_collector::MetricsCollector;

#[tokio::test]
async fn test_log_aggregator_integration() {
    let aggregator = LogAggregator::new().await;
    assert!(aggregator.is_ok(), "LogAggregator should be created successfully");
    
    let aggregator = aggregator.unwrap();
    
    // Test basic functionality
    aggregator.process_log_entry("info", "test", "Test message").await;
    // If we get here without panic, the test passes
}

#[tokio::test]
async fn test_metrics_collector_integration() {
    let collector = MetricsCollector::new();
    
    // Test basic metrics operations
    collector.increment_counter("test_counter");
    collector.set_gauge("test_gauge", 42.0);
    collector.record_histogram("test_histogram", 1.5);
    
    // If we get here without panic, the test passes
}

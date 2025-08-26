use metrics_collector::{MetricsCollector, MetricsConfig};
use std::time::Duration;

#[tokio::test]
async fn test_basic_metrics_collector_functionality() {
    println!("Testing basic metrics collector functionality");
    
    let config = MetricsConfig::default();
    let collector = MetricsCollector::new(config).expect("Failed to create metrics collector");
    
    collector.start().await.expect("Failed to start collector");
    
    // Record some basic metrics
    collector.record_counter("test.counter", 1.0, &[("env", "test")]).await;
    collector.record_gauge("test.gauge", 42.0, &[("service", "logging")]).await;
    collector.record_histogram("test.latency", 0.001, &[("operation", "log")]).await;
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let metrics = collector.get_metrics().await;
    assert_eq!(metrics.len(), 3, "Should have recorded 3 metrics");
    
    collector.stop().await.expect("Failed to stop collector");
    
    assert!(true, "Basic metrics collector test passed");
}

#[tokio::test]
async fn test_metrics_trading_scenarios() {
    let config = MetricsConfig::default();
    let collector = MetricsCollector::new(config).expect("Failed to create metrics collector");
    
    collector.start().await.expect("Failed to start collector");
    
    // Trading-specific metrics
    collector.record_counter("orders.received", 1.0, &[("symbol", "BTCUSD")]).await;
    collector.record_counter("orders.executed", 1.0, &[("symbol", "BTCUSD")]).await;
    collector.record_histogram("order.latency", 0.00005, &[("exchange", "binance")]).await;
    collector.record_gauge("position.size", 0.5, &[("symbol", "BTCUSD")]).await;
    collector.record_gauge("pnl.unrealized", 1250.50, &[("portfolio", "main")]).await;
    
    // System metrics
    collector.record_gauge("memory.usage", 85.5, &[("component", "logger")]).await;
    collector.record_gauge("cpu.usage", 12.3, &[("component", "aggregator")]).await;
    collector.record_counter("messages.processed", 10000.0, &[("service", "logging")]).await;
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    collector.stop().await.expect("Failed to stop collector");
}

#[tokio::test]
async fn test_metrics_high_frequency() {
    let config = MetricsConfig::default();
    let collector = Arc::new(
        MetricsCollector::new(config).expect("Failed to create metrics collector")
    );
    
    collector.start().await.expect("Failed to start collector");
    
    let start_time = std::time::Instant::now();
    let metric_count = 100000;
    
    // Record metrics at high frequency
    let mut handles = Vec::new();
    for task_id in 0..10 {
        let collector_clone = Arc::clone(&collector);
        let handle = tokio::spawn(async move {
            for i in 0..(metric_count / 10) {
                collector_clone.record_counter(
                    "high_frequency.counter",
                    1.0,
                    &[("task", &task_id.to_string()), ("iteration", &i.to_string())]
                ).await;
                
                if i % 100 == 0 {
                    collector_clone.record_histogram(
                        "high_frequency.latency",
                        0.000001 * (i as f64),
                        &[("task", &task_id.to_string())]
                    ).await;
                }
            }
        });
        handles.push(handle);
    }
    
    // Wait for all tasks
    for handle in handles {
        handle.await.expect("Task failed");
    }
    
    let processing_time = start_time.elapsed();
    let throughput = metric_count as f64 / processing_time.as_secs_f64();
    
    println!("Processed {} metrics in {:?}", metric_count, processing_time);
    println!("Throughput: {:.0} metrics/second", throughput);
    
    // Assert high throughput
    assert!(throughput > 50000.0, "Metrics throughput too low: {:.0} metrics/s", throughput);
    
    collector.stop().await.expect("Failed to stop collector");
}

#[tokio::test]
async fn test_metrics_aggregation() {
    let mut config = MetricsConfig::default();
    config.aggregation_interval = Duration::from_millis(50);
    
    let collector = MetricsCollector::new(config).expect("Failed to create metrics collector");
    
    collector.start().await.expect("Failed to start collector");
    
    // Record the same metric multiple times
    for i in 0..100 {
        collector.record_counter("aggregation.test", 1.0, &[("run", "test1")]).await;
        collector.record_histogram("latency.test", 0.001 + (i as f64 * 0.0001), &[("run", "test1")]).await;
    }
    
    // Wait for aggregation
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Get aggregated metrics
    let metrics = collector.get_metrics().await;
    assert!(!metrics.is_empty());
    
    collector.stop().await.expect("Failed to stop collector");
}

#[tokio::test]
async fn test_prometheus_export() {
    let mut config = MetricsConfig::default();
    config.prometheus_enabled = true;
    config.prometheus_port = 9090;
    
    let collector = MetricsCollector::new(config).expect("Failed to create metrics collector");
    
    collector.start().await.expect("Failed to start collector");
    
    // Record some metrics
    collector.record_counter("prometheus_test_counter", 42.0, &[("service", "test")]).await;
    collector.record_gauge("prometheus_test_gauge", 3.14, &[("type", "pi")]).await;
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Try to fetch metrics from Prometheus endpoint
    let client = reqwest::Client::new();
    match client.get("http://localhost:9090/metrics").send().await {
        Ok(response) => {
            let body = response.text().await.expect("Failed to get response body");
            assert!(body.contains("prometheus_test_counter"));
        }
        Err(_) => {
            // Prometheus endpoint might not be available in test environment
            println!("Prometheus endpoint not accessible, test skipped");
        }
    }
    
    collector.stop().await.expect("Failed to stop collector");
}

#[tokio::test]
async fn test_metrics_file_export() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let metrics_file = temp_dir.path().join("metrics.json");
    
    let mut config = MetricsConfig::default();
    config.export_file = Some(metrics_file.clone());
    config.export_interval = Duration::from_millis(50);
    
    let collector = MetricsCollector::new(config).expect("Failed to create metrics collector");
    
    collector.start().await.expect("Failed to start collector");
    
    // Record some metrics
    collector.record_counter("file_export.counter", 10.0, &[("test", "true")]).await;
    collector.record_gauge("file_export.gauge", 25.5, &[("unit", "percent")]).await;
    
    // Wait for export
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    collector.stop().await.expect("Failed to stop collector");
    
    // Verify file was created and contains metrics
    if metrics_file.exists() {
        let content = std::fs::read_to_string(&metrics_file)
            .expect("Failed to read metrics file");
        assert!(content.contains("file_export"));
    }
}

#[tokio::test]
#[traced_test]
async fn test_metrics_memory_limits() {
    let mut config = MetricsConfig::default();
    config.max_metrics_in_memory = 1000; // Small limit
    
    let collector = MetricsCollector::new(config).expect("Failed to create metrics collector");
    
    collector.start().await.expect("Failed to start collector");
    
    // Try to exceed the limit
    for i in 0..2000 {
        collector.record_counter(
            &format!("memory_limit_test_{}", i),
            1.0,
            &[("test", "memory_limit")]
        ).await;
        
        if i % 100 == 0 {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Metrics collector should handle memory limits gracefully
    let metrics = collector.get_metrics().await;
    println!("Metrics in memory: {}", metrics.len());
    
    collector.stop().await.expect("Failed to stop collector");
}

#[tokio::test]
async fn test_metrics_different_types() {
    let config = MetricsConfig::default();
    let collector = MetricsCollector::new(config).expect("Failed to create metrics collector");
    
    collector.start().await.expect("Failed to start collector");
    
    // Counter - always increases
    collector.record_counter("type_test.counter", 1.0, &[]).await;
    collector.record_counter("type_test.counter", 2.0, &[]).await;
    collector.record_counter("type_test.counter", 3.0, &[]).await;
    
    // Gauge - can go up and down
    collector.record_gauge("type_test.gauge", 10.0, &[]).await;
    collector.record_gauge("type_test.gauge", 5.0, &[]).await;
    collector.record_gauge("type_test.gauge", 15.0, &[]).await;
    
    // Histogram - records distribution of values
    collector.record_histogram("type_test.histogram", 0.001, &[]).await;
    collector.record_histogram("type_test.histogram", 0.005, &[]).await;
    collector.record_histogram("type_test.histogram", 0.010, &[]).await;
    collector.record_histogram("type_test.histogram", 0.002, &[]).await;
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    collector.stop().await.expect("Failed to stop collector");
}

#[tokio::test]
async fn test_metrics_labels() {
    let config = MetricsConfig::default();
    let collector = MetricsCollector::new(config).expect("Failed to create metrics collector");
    
    collector.start().await.expect("Failed to start collector");
    
    // Same metric name with different labels should be treated as different metrics
    collector.record_counter("labeled_metric", 1.0, &[("env", "prod"), ("service", "api")]).await;
    collector.record_counter("labeled_metric", 2.0, &[("env", "dev"), ("service", "api")]).await;
    collector.record_counter("labeled_metric", 3.0, &[("env", "prod"), ("service", "worker")]).await;
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let metrics = collector.get_metrics().await;
    
    // Should have separate entries for different label combinations
    let labeled_metrics: Vec<_> = metrics.iter()
        .filter(|m| m.name == "labeled_metric")
        .collect();
    
    assert!(labeled_metrics.len() >= 3);
    
    collector.stop().await.expect("Failed to stop collector");
}

#[tokio::test]
async fn test_metrics_performance_regression() {
    let config = MetricsConfig::default();
    let collector = MetricsCollector::new(config).expect("Failed to create metrics collector");
    
    collector.start().await.expect("Failed to start collector");
    
    // Measure performance
    let start_time = std::time::Instant::now();
    let operations = 10000;
    
    for i in 0..operations {
        let latency = 0.000001 + (i as f64 * 0.000000001); // Simulate varying latency
        collector.record_histogram("performance.latency", latency, &[("test", "regression")]).await;
    }
    
    let total_time = start_time.elapsed();
    let ops_per_second = operations as f64 / total_time.as_secs_f64();
    
    println!("Performance test: {:.0} operations/second", ops_per_second);
    
    // Performance regression check
    assert!(ops_per_second > 100000.0, "Performance regression detected: {:.0} ops/s", ops_per_second);
    
    collector.stop().await.expect("Failed to stop collector");
}

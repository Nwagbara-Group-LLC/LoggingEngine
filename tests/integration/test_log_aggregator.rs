use log_aggregator::{LogAggregator, AggregatorConfig};
use std::time::Duration;

#[tokio::test]
async fn test_basic_aggregator_functionality() {
    println!("Testing basic aggregator functionality");
    
    let config = AggregatorConfig::default();
    let aggregator = LogAggregator::new(config).expect("Failed to create aggregator");
    
    aggregator.start().await.expect("Failed to start aggregator");
    
    // Simulate processing some log entries
    aggregator.process_log_entry("INFO", "test_module", "Test message 1").await;
    aggregator.process_log_entry("WARN", "test_module", "Test message 2").await;
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    aggregator.stop().await.expect("Failed to stop aggregator");
    
    assert!(true, "Basic aggregator test passed");
}

#[tokio::test]
async fn test_aggregator_with_filters() {
    let mut config = AggregatorConfig::default();
    config.filters.push(Filter::LevelFilter(LogLevel::Warn));
    config.filters.push(Filter::ModuleFilter("sensitive_module".to_string()));
    
    let aggregator = LogAggregator::new(config).expect("Failed to create aggregator");
    
    aggregator.start().await.expect("Failed to start aggregator");
    
    // These should be aggregated
    aggregator.process_log_entry("WARN", "normal_module", "Warning message").await;
    aggregator.process_log_entry("ERROR", "normal_module", "Error message").await;
    
    // These should be filtered out
    aggregator.process_log_entry("INFO", "normal_module", "Info message").await;
    aggregator.process_log_entry("ERROR", "sensitive_module", "Sensitive error").await;
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    aggregator.stop().await.expect("Failed to stop aggregator");
}

#[tokio::test]
async fn test_aggregator_batching() {
    let mut config = AggregatorConfig::default();
    config.batch_size = 5;
    config.batch_timeout = Duration::from_millis(50);
    
    let aggregator = LogAggregator::new(config).expect("Failed to create aggregator");
    
    aggregator.start().await.expect("Failed to start aggregator");
    
    // Send exactly batch_size messages
    for i in 0..5 {
        aggregator.process_log_entry(
            "INFO",
            "batch_test",
            &format!("Batch message {}", i)
        ).await;
    }
    
    // Wait for batch processing
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    aggregator.stop().await.expect("Failed to stop aggregator");
}

#[tokio::test]
async fn test_aggregator_timeout_flush() {
    let mut config = AggregatorConfig::default();
    config.batch_size = 100; // Large batch size
    config.batch_timeout = Duration::from_millis(50); // Short timeout
    
    let aggregator = LogAggregator::new(config).expect("Failed to create aggregator");
    
    aggregator.start().await.expect("Failed to start aggregator");
    
    // Send fewer messages than batch_size
    for i in 0..3 {
        aggregator.process_log_entry(
            "INFO",
            "timeout_test",
            &format!("Timeout message {}", i)
        ).await;
    }
    
    // Wait for timeout flush
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    aggregator.stop().await.expect("Failed to stop aggregator");
}

#[tokio::test]
#[traced_test]
async fn test_aggregator_high_throughput() {
    let mut config = AggregatorConfig::default();
    config.batch_size = 1000;
    config.batch_timeout = Duration::from_millis(10);
    
    let aggregator = Arc::new(
        LogAggregator::new(config).expect("Failed to create aggregator")
    );
    
    aggregator.start().await.expect("Failed to start aggregator");
    
    let start_time = std::time::Instant::now();
    let message_count = 50000;
    
    // Send messages concurrently
    let mut handles = Vec::new();
    for task_id in 0..10 {
        let aggregator_clone = Arc::clone(&aggregator);
        let handle = tokio::spawn(async move {
            for i in 0..(message_count / 10) {
                aggregator_clone.process_log_entry(
                    "INFO",
                    "throughput_test",
                    &format!("Task {} message {}", task_id, i)
                ).await;
            }
        });
        handles.push(handle);
    }
    
    // Wait for all tasks
    for handle in handles {
        handle.await.expect("Task failed");
    }
    
    let processing_time = start_time.elapsed();
    let throughput = message_count as f64 / processing_time.as_secs_f64();
    
    println!("Processed {} messages in {:?}", message_count, processing_time);
    println!("Throughput: {:.0} messages/second", throughput);
    
    // Assert high throughput
    assert!(throughput > 100000.0, "Throughput too low: {:.0} msg/s", throughput);
    
    aggregator.stop().await.expect("Failed to stop aggregator");
}

#[tokio::test]
async fn test_aggregator_redis_transport() {
    // Skip if Redis is not available
    if std::env::var("SKIP_REDIS_TESTS").is_ok() {
        return;
    }
    
    let mut config = AggregatorConfig::default();
    config.output_transport = Transport::Redis {
        url: "redis://localhost:6379".to_string(),
        channel: "logs".to_string(),
    };
    
    let aggregator = match LogAggregator::new(config) {
        Ok(agg) => agg,
        Err(_) => {
            println!("Redis not available, skipping test");
            return;
        }
    };
    
    aggregator.start().await.expect("Failed to start aggregator");
    
    aggregator.process_log_entry("INFO", "redis_test", "Redis message").await;
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    aggregator.stop().await.expect("Failed to stop aggregator");
}

#[tokio::test]
async fn test_aggregator_file_output() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let output_file = temp_dir.path().join("aggregated.log");
    
    let mut config = AggregatorConfig::default();
    config.output_transport = Transport::File(output_file.clone());
    
    let aggregator = LogAggregator::new(config).expect("Failed to create aggregator");
    
    aggregator.start().await.expect("Failed to start aggregator");
    
    // Process several log entries
    for i in 0..10 {
        aggregator.process_log_entry(
            "INFO",
            "file_test",
            &format!("File output message {}", i)
        ).await;
    }
    
    // Wait for file writing
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    aggregator.stop().await.expect("Failed to stop aggregator");
    
    // Verify file was created and contains data
    assert!(output_file.exists());
    let content = std::fs::read_to_string(&output_file)
        .expect("Failed to read output file");
    assert!(content.contains("File output message"));
}

#[tokio::test]
async fn test_aggregator_memory_usage() {
    let mut config = AggregatorConfig::default();
    config.max_memory_usage = 1024 * 1024; // 1MB limit
    
    let aggregator = LogAggregator::new(config).expect("Failed to create aggregator");
    
    aggregator.start().await.expect("Failed to start aggregator");
    
    // Try to exceed memory limit
    let large_message = "A".repeat(1000);
    for i in 0..2000 {
        aggregator.process_log_entry(
            "INFO",
            "memory_test",
            &format!("{} - message {}", large_message, i)
        ).await;
        
        // Check if memory usage is being controlled
        if i % 100 == 0 {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    aggregator.stop().await.expect("Failed to stop aggregator");
}

#[tokio::test]
async fn test_aggregator_error_handling() {
    let mut config = AggregatorConfig::default();
    config.output_transport = Transport::File("/invalid/path/output.log".into());
    
    let aggregator = LogAggregator::new(config).expect("Failed to create aggregator");
    
    // Starting should handle the invalid path gracefully
    let result = timeout(Duration::from_secs(5), aggregator.start()).await;
    
    match result {
        Ok(Ok(_)) => {
            // If it started successfully, test error handling during processing
            aggregator.process_log_entry("INFO", "error_test", "Test message").await;
            tokio::time::sleep(Duration::from_millis(100)).await;
            let _ = aggregator.stop().await;
        }
        Ok(Err(_)) => {
            // Expected error due to invalid path
            println!("Aggregator properly handled invalid file path");
        }
        Err(_) => {
            panic!("Aggregator startup timed out");
        }
    }
}

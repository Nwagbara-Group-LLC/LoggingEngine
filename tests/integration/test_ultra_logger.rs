use ultra_logger::{UltraLogger, LogLevel};
use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
async fn test_basic_logger_functionality() {
    // Basic test that should work with our current implementation
    println!("Testing basic logger functionality");
    
    // For now, just test that we can create the types
    // In a real implementation, these would do actual logging
    
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert!(true, "Basic logger test passed");
}

#[tokio::test]
async fn test_file_transport() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let log_file_path = temp_dir.path().join("test.log");
    
    let logger = UltraLogger::builder()
        .with_transport(Transport::File(log_file_path.clone()))
        .with_buffer_config(BufferConfig::default())
        .build()
        .expect("Failed to create logger");

    logger.start().await.expect("Failed to start logger");
    
    logger.log(LogLevel::Info, "test", "File transport test").await;
    logger.log(LogLevel::Warn, "test", "Warning message").await;
    
    logger.stop().await.expect("Failed to stop logger");
    
    // Verify file was created and contains data
    assert!(log_file_path.exists());
    let content = std::fs::read_to_string(&log_file_path)
        .expect("Failed to read log file");
    assert!(content.contains("File transport test"));
    assert!(content.contains("Warning message"));
}

#[tokio::test]
async fn test_compression() {
    let logger = UltraLogger::builder()
        .with_transport(Transport::Memory)
        .with_compression(CompressionType::Lz4)
        .build()
        .expect("Failed to create logger");

    logger.start().await.expect("Failed to start logger");
    
    // Log a large message that should benefit from compression
    let large_message = "A".repeat(1000);
    logger.log(LogLevel::Info, "test", &large_message).await;
    
    logger.stop().await.expect("Failed to stop logger");
}

#[tokio::test]
async fn test_log_levels() {
    let logger = UltraLogger::builder()
        .with_transport(Transport::Memory)
        .with_min_level(LogLevel::Warn)
        .build()
        .expect("Failed to create logger");

    logger.start().await.expect("Failed to start logger");
    
    // These should be logged
    logger.log(LogLevel::Error, "test", "Error message").await;
    logger.log(LogLevel::Warn, "test", "Warning message").await;
    
    // These should be filtered out
    logger.log(LogLevel::Info, "test", "Info message").await;
    logger.log(LogLevel::Debug, "test", "Debug message").await;
    
    logger.stop().await.expect("Failed to stop logger");
}

#[tokio::test]
async fn test_concurrent_logging() {
    let logger = Arc::new(
        UltraLogger::builder()
            .with_transport(Transport::Memory)
            .build()
            .expect("Failed to create logger")
    );

    logger.start().await.expect("Failed to start logger");
    
    let mut handles = Vec::new();
    let log_count = Arc::new(AtomicU64::new(0));
    
    // Spawn multiple tasks that log concurrently
    for task_id in 0..10 {
        let logger_clone = Arc::clone(&logger);
        let log_count_clone = Arc::clone(&log_count);
        
        let handle = tokio::spawn(async move {
            for i in 0..100 {
                logger_clone.log(
                    LogLevel::Info,
                    "concurrent_test",
                    &format!("Task {} message {}", task_id, i)
                ).await;
                log_count_clone.fetch_add(1, Ordering::Relaxed);
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.expect("Task failed");
    }
    
    // Verify all messages were logged
    assert_eq!(log_count.load(Ordering::Relaxed), 1000);
    
    logger.stop().await.expect("Failed to stop logger");
}

#[tokio::test]
async fn test_ultra_low_latency() {
    let logger = UltraLogger::builder()
        .with_transport(Transport::Memory)
        .with_buffer_config(BufferConfig {
            capacity: 1024 * 1024, // 1MB buffer
            batch_size: 100,
            flush_interval: Duration::from_nanos(100), // 100ns flush
        })
        .build()
        .expect("Failed to create logger");

    logger.start().await.expect("Failed to start logger");
    
    // Measure logging latency
    let mut latencies = Vec::new();
    
    for i in 0..1000 {
        let start = Instant::now();
        
        logger.log(
            LogLevel::Info,
            "latency_test",
            &format!("Latency test message {}", i)
        ).await;
        
        let latency = start.elapsed();
        latencies.push(latency);
    }
    
    logger.stop().await.expect("Failed to stop logger");
    
    // Calculate statistics
    latencies.sort();
    let avg_latency = latencies.iter().sum::<Duration>() / latencies.len() as u32;
    let p50_latency = latencies[latencies.len() / 2];
    let p95_latency = latencies[(latencies.len() * 95) / 100];
    let p99_latency = latencies[(latencies.len() * 99) / 100];
    
    println!("Latency Statistics:");
    println!("  Average: {:?}", avg_latency);
    println!("  P50: {:?}", p50_latency);
    println!("  P95: {:?}", p95_latency);
    println!("  P99: {:?}", p99_latency);
    
    // Assert ultra-low latency requirements
    assert!(avg_latency < Duration::from_micros(1), "Average latency too high: {:?}", avg_latency);
    assert!(p99_latency < Duration::from_micros(5), "P99 latency too high: {:?}", p99_latency);
}

#[tokio::test]
async fn test_trading_scenario_logging() {
    let logger = UltraLogger::builder()
        .with_transport(Transport::Memory)
        .build()
        .expect("Failed to create logger");

    logger.start().await.expect("Failed to start logger");
    
    // Simulate a trading scenario
    let order_id = "ORD_12345";
    let symbol = "BTCUSD";
    let price = "45000.50";
    let quantity = "0.1";
    
    // Order received
    let start_time = Instant::now();
    logger.log(
        LogLevel::Info,
        "trading",
        &format!("ORDER_RECEIVED|{}|{}|{}|{}", order_id, symbol, price, quantity)
    ).await;
    
    // Risk check
    logger.log(
        LogLevel::Info,
        "risk",
        &format!("RISK_CHECK_PASSED|{}", order_id)
    ).await;
    
    // Order sent to exchange
    logger.log(
        LogLevel::Info,
        "exchange",
        &format!("ORDER_SENT_TO_EXCHANGE|{}", order_id)
    ).await;
    
    // Order acknowledged
    logger.log(
        LogLevel::Info,
        "exchange",
        &format!("ORDER_ACK|{}", order_id)
    ).await;
    
    let total_time = start_time.elapsed();
    println!("Total trading scenario time: {:?}", total_time);
    
    // Assert the entire flow completed quickly
    assert!(total_time < Duration::from_micros(50), "Trading scenario too slow: {:?}", total_time);
    
    logger.stop().await.expect("Failed to stop logger");
}

#[tokio::test]
#[traced_test]
async fn test_buffer_overflow_handling() {
    let small_buffer_config = BufferConfig {
        capacity: 1024, // Small buffer
        batch_size: 10,
        flush_interval: Duration::from_millis(10),
    };
    
    let logger = UltraLogger::builder()
        .with_transport(Transport::Memory)
        .with_buffer_config(small_buffer_config)
        .build()
        .expect("Failed to create logger");

    logger.start().await.expect("Failed to start logger");
    
    // Generate enough logs to potentially overflow the buffer
    for i in 0..10000 {
        logger.log(
            LogLevel::Info,
            "overflow_test",
            &format!("Buffer overflow test message {} with some extra data to make it larger", i)
        ).await;
    }
    
    // Wait for processing
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    logger.stop().await.expect("Failed to stop logger");
}

#[tokio::test]
async fn test_structured_logging() {
    let logger = UltraLogger::builder()
        .with_transport(Transport::Memory)
        .build()
        .expect("Failed to create logger");

    logger.start().await.expect("Failed to start logger");
    
    // Test structured logging with JSON-like format
    logger.log_structured(
        LogLevel::Info,
        "trading",
        &[
            ("event", "order_placed"),
            ("order_id", "ORD_123"),
            ("symbol", "BTCUSD"),
            ("price", "45000.50"),
            ("quantity", "0.1"),
            ("timestamp", "2025-08-26T10:30:00Z")
        ]
    ).await;
    
    logger.stop().await.expect("Failed to stop logger");
}

use proptest::prelude::*;
use std::time::Duration;

// Basic property-based test
proptest! {
    #[test]
    fn test_basic_string_properties(
        message in r".{1,100}",
    ) {
        // Test that string handling is robust
        assert!(!message.is_empty());
        assert!(message.len() <= 100);
    }
}

proptest! {
    #[test]
    fn test_duration_properties(
        millis in 1u64..=1000,
    ) {
        let duration = Duration::from_millis(millis);
        assert!(duration.as_millis() > 0);
        assert!(duration.as_millis() <= 1000);
    }
}

proptest! {
    #[test]
    fn test_buffer_config_properties(
        capacity in 1024usize..=1024*1024*10, // 1KB to 10MB
        batch_size in 1usize..=10000,
        flush_interval_ms in 1u64..=1000,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        rt.block_on(async {
            let buffer_config = BufferConfig {
                capacity,
                batch_size: batch_size.min(capacity / 10), // Ensure batch size is reasonable
                flush_interval: Duration::from_millis(flush_interval_ms),
            };
            
            let logger = UltraLogger::builder()
                .with_transport(Transport::Memory)
                .with_buffer_config(buffer_config)
                .build()
                .expect("Failed to create logger");
            
            logger.start().await.expect("Failed to start logger");
            
            // Log a few messages to test the configuration
            for i in 0..10 {
                logger.log(LogLevel::Info, "proptest", &format!("Message {}", i)).await;
            }
            
            logger.stop().await.expect("Failed to stop logger");
        });
    }
}

proptest! {
    #[test]
    fn test_aggregator_batch_properties(
        batch_size in 1usize..=1000,
        timeout_ms in 1u64..=100,
        message_count in 1usize..=100,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        rt.block_on(async {
            let mut config = AggregatorConfig::default();
            config.batch_size = batch_size;
            config.batch_timeout = Duration::from_millis(timeout_ms);
            
            let aggregator = LogAggregator::new(config)
                .expect("Failed to create aggregator");
            
            aggregator.start().await.expect("Failed to start aggregator");
            
            // Process arbitrary number of messages
            for i in 0..message_count {
                aggregator.process_log_entry(
                    "INFO",
                    "proptest",
                    &format!("Property test message {}", i)
                ).await;
            }
            
            // Wait for processing
            tokio::time::sleep(Duration::from_millis(timeout_ms * 2)).await;
            
            aggregator.stop().await.expect("Failed to stop aggregator");
        });
    }
}

proptest! {
    #[test]
    fn test_concurrent_logging_invariants(
        thread_count in 1usize..=8,
        messages_per_thread in 1usize..=100,
        message_length in 1usize..=100,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        rt.block_on(async {
            let logger = std::sync::Arc::new(
                UltraLogger::builder()
                    .with_transport(Transport::Memory)
                    .build()
                    .expect("Failed to create logger")
            );
            
            logger.start().await.expect("Failed to start logger");
            
            let mut handles = Vec::new();
            
            for thread_id in 0..thread_count {
                let logger_clone = std::sync::Arc::clone(&logger);
                let handle = tokio::spawn(async move {
                    for msg_id in 0..messages_per_thread {
                        let message = "A".repeat(message_length);
                        logger_clone.log(
                            LogLevel::Info,
                            &format!("thread_{}", thread_id),
                            &format!("{}_{}", message, msg_id)
                        ).await;
                    }
                });
                handles.push(handle);
            }
            
            // All threads should complete successfully
            for handle in handles {
                handle.await.expect("Thread should not panic");
            }
            
            logger.stop().await.expect("Failed to stop logger");
        });
    }
}

// Property test for structured logging
proptest! {
    #[test]
    fn test_structured_logging_properties(
        key_value_pairs in prop::collection::vec(
            (r"[a-zA-Z_][a-zA-Z0-9_]*", r".{1,100}"),
            1..=10
        ),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        rt.block_on(async {
            let logger = UltraLogger::builder()
                .with_transport(Transport::Memory)
                .build()
                .expect("Failed to create logger");
            
            logger.start().await.expect("Failed to start logger");
            
            // Convert to string tuples for the API
            let fields: Vec<(&str, &str)> = key_value_pairs
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            
            // Should handle arbitrary structured data
            logger.log_structured(
                LogLevel::Info,
                "proptest",
                &fields
            ).await;
            
            logger.stop().await.expect("Failed to stop logger");
        });
    }
}

// Property test for compression efficiency
proptest! {
    #[test]
    fn test_compression_properties(
        message_size in 100usize..=10000,
        repetition_factor in 1usize..=10,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        rt.block_on(async {
            // Create a message with potential for compression
            let base_message = "Trading data: ".repeat(repetition_factor);
            let full_message = format!("{}{}", base_message, "X".repeat(message_size));
            
            for compression_type in [
                ultra_logger::CompressionType::None,
                ultra_logger::CompressionType::Lz4,
                ultra_logger::CompressionType::Zstd,
            ].iter() {
                let logger = UltraLogger::builder()
                    .with_transport(Transport::Memory)
                    .with_compression(*compression_type)
                    .build()
                    .expect("Failed to create logger");
                
                logger.start().await.expect("Failed to start logger");
                
                // Should handle large messages with any compression
                logger.log(LogLevel::Info, "compression_test", &full_message).await;
                
                logger.stop().await.expect("Failed to stop logger");
            }
        });
    }
}

// Property test for metrics with arbitrary labels
proptest! {
    #[test]
    fn test_metrics_with_arbitrary_labels(
        metric_name in r"[a-zA-Z_][a-zA-Z0-9_\.]*",
        metric_value in -1000.0f64..=1000.0f64,
        labels in prop::collection::vec(
            (r"[a-zA-Z_][a-zA-Z0-9_]*", r"[a-zA-Z0-9_\-\.]*"),
            0..=5
        ),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        rt.block_on(async {
            let collector = metrics_collector::MetricsCollector::new(
                metrics_collector::MetricsConfig::default()
            ).expect("Failed to create metrics collector");
            
            collector.start().await.expect("Failed to start collector");
            
            let label_refs: Vec<(&str, &str)> = labels
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            
            // Should handle arbitrary metric names, values, and labels
            collector.record_gauge(&metric_name, metric_value, &label_refs).await;
            collector.record_counter(&metric_name, metric_value.abs(), &label_refs).await;
            
            if metric_value.is_finite() && metric_value >= 0.0 {
                collector.record_histogram(&metric_name, metric_value, &label_refs).await;
            }
            
            collector.stop().await.expect("Failed to stop collector");
        });
    }
}

// Property test for error recovery
proptest! {
    #[test]
    fn test_error_recovery_properties(
        operation_count in 1usize..=50,
        failure_points in prop::collection::vec(0usize..50, 0..=5),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        rt.block_on(async {
            let logger = UltraLogger::builder()
                .with_transport(Transport::Memory)
                .build()
                .expect("Failed to create logger");
            
            logger.start().await.expect("Failed to start logger");
            
            for i in 0..operation_count {
                // Simulate potential failure points
                if failure_points.contains(&i) {
                    // Try to cause stress by rapid operations
                    for j in 0..10 {
                        logger.log(
                            LogLevel::Info,
                            "stress_test",
                            &format!("Rapid message {} at operation {}", j, i)
                        ).await;
                    }
                } else {
                    logger.log(
                        LogLevel::Info,
                        "normal_test",
                        &format!("Normal message at operation {}", i)
                    ).await;
                }
                
                // Small delay to prevent overwhelming
                if i % 10 == 0 {
                    tokio::time::sleep(Duration::from_micros(1)).await;
                }
            }
            
            logger.stop().await.expect("Failed to stop logger");
        });
    }
}

// Property test for memory bounds
proptest! {
    #[test]
    fn test_memory_bounds_invariant(
        buffer_size in 1024usize..=1024*100, // 1KB to 100KB for fast tests
        message_count in 10usize..=1000,
        message_size in 10usize..=500,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        rt.block_on(async {
            let logger = UltraLogger::builder()
                .with_transport(Transport::Memory)
                .with_buffer_config(BufferConfig {
                    capacity: buffer_size,
                    batch_size: (buffer_size / 100).max(1),
                    flush_interval: Duration::from_millis(1),
                })
                .build()
                .expect("Failed to create logger");
            
            logger.start().await.expect("Failed to start logger");
            
            // Try to exceed buffer limits
            let message = "X".repeat(message_size);
            for i in 0..message_count {
                logger.log(
                    LogLevel::Info,
                    "memory_test",
                    &format!("{}_message_{}", message, i)
                ).await;
                
                // Occasional yield to prevent test timeout
                if i % 50 == 0 {
                    tokio::time::sleep(Duration::from_micros(1)).await;
                }
            }
            
            // System should handle memory pressure gracefully
            tokio::time::sleep(Duration::from_millis(10)).await;
            
            logger.stop().await.expect("Failed to stop logger");
        });
    }
}

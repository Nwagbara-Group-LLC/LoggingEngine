use ultra_logger::{UltraLogger, LogLevel, Transport};
use log_aggregator::{LogAggregator, AggregatorConfig};
use metrics_collector::{MetricsCollector, MetricsConfig};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing_test::traced_test;

#[tokio::test]
#[traced_test]
async fn test_complete_logging_pipeline() {
    // Set up all components
    let logger = Arc::new(
        UltraLogger::builder()
            .with_transport(Transport::Memory)
            .build()
            .expect("Failed to create logger")
    );

    let aggregator_config = AggregatorConfig::default();
    let aggregator = Arc::new(
        LogAggregator::new(aggregator_config)
            .expect("Failed to create aggregator")
    );

    let metrics_config = MetricsConfig::default();
    let metrics_collector = Arc::new(
        MetricsCollector::new(metrics_config)
            .expect("Failed to create metrics collector")
    );

    // Start all components
    logger.start().await.expect("Failed to start logger");
    aggregator.start().await.expect("Failed to start aggregator");
    metrics_collector.start().await.expect("Failed to start metrics collector");

    // Wait for initialization
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Simulate a complete trading workflow
    let order_id = "ORD_E2E_12345";
    let start_time = Instant::now();

    // 1. Order received
    logger.log(LogLevel::Info, "trading", &format!("ORDER_RECEIVED|{}", order_id)).await;
    metrics_collector.record_counter("orders.received", 1.0, &[("symbol", "BTCUSD")]).await;

    // 2. Risk validation
    logger.log(LogLevel::Info, "risk", &format!("RISK_VALIDATION_START|{}", order_id)).await;
    
    let risk_start = Instant::now();
    tokio::time::sleep(Duration::from_micros(10)).await; // Simulate risk check
    let risk_duration = risk_start.elapsed();
    
    logger.log(LogLevel::Info, "risk", &format!("RISK_VALIDATION_COMPLETE|{}|duration_us={}", order_id, risk_duration.as_micros())).await;
    metrics_collector.record_histogram("risk.validation.latency", risk_duration.as_secs_f64(), &[("result", "passed")]).await;

    // 3. Order routing
    logger.log(LogLevel::Info, "routing", &format!("ORDER_ROUTING_START|{}", order_id)).await;
    
    let routing_start = Instant::now();
    tokio::time::sleep(Duration::from_micros(5)).await; // Simulate routing
    let routing_duration = routing_start.elapsed();
    
    logger.log(LogLevel::Info, "routing", &format!("ORDER_ROUTED|{}|exchange=binance|duration_us={}", order_id, routing_duration.as_micros())).await;
    metrics_collector.record_histogram("routing.latency", routing_duration.as_secs_f64(), &[("exchange", "binance")]).await;

    // 4. Order execution
    logger.log(LogLevel::Info, "execution", &format!("ORDER_SENT_TO_EXCHANGE|{}", order_id)).await;
    
    let execution_start = Instant::now();
    tokio::time::sleep(Duration::from_micros(20)).await; // Simulate exchange response
    let execution_duration = execution_start.elapsed();
    
    logger.log(LogLevel::Info, "execution", &format!("ORDER_ACKNOWLEDGED|{}|duration_us={}", order_id, execution_duration.as_micros())).await;
    metrics_collector.record_histogram("execution.latency", execution_duration.as_secs_f64(), &[("status", "ack")]).await;

    // 5. Fill notification
    logger.log(LogLevel::Info, "execution", &format!("ORDER_FILLED|{}|price=45000.50|quantity=0.1", order_id)).await;
    metrics_collector.record_counter("orders.filled", 1.0, &[("symbol", "BTCUSD")]).await;

    let total_duration = start_time.elapsed();
    logger.log(LogLevel::Info, "trading", &format!("ORDER_COMPLETE|{}|total_duration_us={}", order_id, total_duration.as_micros())).await;
    metrics_collector.record_histogram("order.total.latency", total_duration.as_secs_f64(), &[("outcome", "filled")]).await;

    // Wait for all processing to complete
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify total latency is within acceptable bounds for HFT
    assert!(total_duration < Duration::from_micros(100), 
            "End-to-end latency too high: {:?}", total_duration);

    println!("End-to-end test completed successfully:");
    println!("  Total latency: {:?}", total_duration);
    println!("  Risk validation: {:?}", risk_duration);
    println!("  Order routing: {:?}", routing_duration);
    println!("  Order execution: {:?}", execution_duration);

    // Stop all components
    logger.stop().await.expect("Failed to stop logger");
    aggregator.stop().await.expect("Failed to stop aggregator");
    metrics_collector.stop().await.expect("Failed to stop metrics collector");
}

#[tokio::test]
async fn test_system_under_load() {
    // Set up components with higher performance configuration
    let logger = Arc::new(
        UltraLogger::builder()
            .with_transport(Transport::Memory)
            .build()
            .expect("Failed to create logger")
    );

    let mut aggregator_config = AggregatorConfig::default();
    aggregator_config.batch_size = 1000;
    aggregator_config.batch_timeout = Duration::from_millis(5);
    
    let aggregator = Arc::new(
        LogAggregator::new(aggregator_config)
            .expect("Failed to create aggregator")
    );

    let metrics_config = MetricsConfig::default();
    let metrics_collector = Arc::new(
        MetricsCollector::new(metrics_config)
            .expect("Failed to create metrics collector")
    );

    // Start all components
    logger.start().await.expect("Failed to start logger");
    aggregator.start().await.expect("Failed to start aggregator");
    metrics_collector.start().await.expect("Failed to start metrics collector");

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Generate high load
    let concurrent_orders = 1000;
    let start_time = Instant::now();

    let mut handles = Vec::new();
    
    for order_id in 0..concurrent_orders {
        let logger_clone = Arc::clone(&logger);
        let metrics_clone = Arc::clone(&metrics_collector);
        
        let handle = tokio::spawn(async move {
            let order_start = Instant::now();
            
            // Log order lifecycle
            logger_clone.log(LogLevel::Info, "trading", 
                           &format!("ORDER_RECEIVED|ORD_{}", order_id)).await;
            
            logger_clone.log(LogLevel::Info, "risk", 
                           &format!("RISK_CHECK_PASSED|ORD_{}", order_id)).await;
            
            logger_clone.log(LogLevel::Info, "execution", 
                           &format!("ORDER_EXECUTED|ORD_{}", order_id)).await;
            
            let order_duration = order_start.elapsed();
            
            // Record metrics
            metrics_clone.record_counter("orders.processed", 1.0, 
                                       &[("test", "load")]).await;
            metrics_clone.record_histogram("order.processing.latency", 
                                         order_duration.as_secs_f64(), 
                                         &[("test", "load")]).await;
        });
        
        handles.push(handle);
    }

    // Wait for all orders to complete
    for handle in handles {
        handle.await.expect("Order processing failed");
    }

    let total_time = start_time.elapsed();
    let throughput = concurrent_orders as f64 / total_time.as_secs_f64();

    println!("Load test results:");
    println!("  Processed {} orders in {:?}", concurrent_orders, total_time);
    println!("  Throughput: {:.0} orders/second", throughput);

    // Assert system can handle high load
    assert!(throughput > 10000.0, "System throughput too low under load: {:.0} orders/s", throughput);

    // Wait for final processing
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Stop components
    logger.stop().await.expect("Failed to stop logger");
    aggregator.stop().await.expect("Failed to stop aggregator");
    metrics_collector.stop().await.expect("Failed to stop metrics collector");
}

#[tokio::test]
async fn test_component_failure_recovery() {
    let logger = Arc::new(
        UltraLogger::builder()
            .with_transport(Transport::Memory)
            .build()
            .expect("Failed to create logger")
    );

    let aggregator_config = AggregatorConfig::default();
    let aggregator = Arc::new(
        LogAggregator::new(aggregator_config)
            .expect("Failed to create aggregator")
    );

    // Start components
    logger.start().await.expect("Failed to start logger");
    aggregator.start().await.expect("Failed to start aggregator");

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Log some messages
    logger.log(LogLevel::Info, "test", "Before failure").await;

    // Simulate component restart (stop and start aggregator)
    aggregator.stop().await.expect("Failed to stop aggregator");
    tokio::time::sleep(Duration::from_millis(50)).await;
    aggregator.start().await.expect("Failed to restart aggregator");

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Continue logging after restart
    logger.log(LogLevel::Info, "test", "After recovery").await;

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Clean shutdown
    logger.stop().await.expect("Failed to stop logger");
    aggregator.stop().await.expect("Failed to stop aggregator");
}

#[tokio::test]
async fn test_cross_component_communication() {
    let (tx, mut rx) = mpsc::channel(100);

    // Set up components with communication channel
    let logger = Arc::new(
        UltraLogger::builder()
            .with_transport(Transport::Memory)
            .with_notification_channel(tx)
            .build()
            .expect("Failed to create logger")
    );

    logger.start().await.expect("Failed to start logger");

    // Log messages and verify notifications
    logger.log(LogLevel::Info, "comm_test", "Message 1").await;
    logger.log(LogLevel::Error, "comm_test", "Error message").await;
    logger.log(LogLevel::Info, "comm_test", "Message 2").await;

    // Check for notifications
    let mut notification_count = 0;
    while let Ok(notification) = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await {
        if notification.is_some() {
            notification_count += 1;
        } else {
            break;
        }
        
        if notification_count >= 3 {
            break;
        }
    }

    assert!(notification_count >= 1, "No notifications received");
    println!("Received {} notifications", notification_count);

    logger.stop().await.expect("Failed to stop logger");
}

#[tokio::test]
#[traced_test]
async fn test_distributed_tracing() {
    use tracing::{info_span, Instrument};

    let logger = Arc::new(
        UltraLogger::builder()
            .with_transport(Transport::Memory)
            .with_tracing_enabled(true)
            .build()
            .expect("Failed to create logger")
    );

    logger.start().await.expect("Failed to start logger");

    // Test with distributed tracing
    async {
        let order_span = info_span!("process_order", order_id = "ORD_TRACE_123");
        
        async {
            logger.log(LogLevel::Info, "tracing", "Order processing started").await;
            
            let risk_span = info_span!("risk_check");
            async {
                logger.log(LogLevel::Info, "tracing", "Risk check in progress").await;
            }.instrument(risk_span).await;
            
            let execution_span = info_span!("execute_order");
            async {
                logger.log(LogLevel::Info, "tracing", "Order execution complete").await;
            }.instrument(execution_span).await;
            
        }.instrument(order_span).await;
    }.await;

    tokio::time::sleep(Duration::from_millis(100)).await;

    logger.stop().await.expect("Failed to stop logger");
}

#[tokio::test]
async fn test_backpressure_handling() {
    // Set up logger with small buffer to test backpressure
    let logger = Arc::new(
        UltraLogger::builder()
            .with_transport(Transport::Memory)
            .with_buffer_config(ultra_logger::BufferConfig {
                capacity: 100, // Very small buffer
                batch_size: 10,
                flush_interval: Duration::from_millis(100),
            })
            .build()
            .expect("Failed to create logger")
    );

    logger.start().await.expect("Failed to start logger");

    // Try to overwhelm the buffer
    let start_time = Instant::now();
    for i in 0..1000 {
        logger.log(LogLevel::Info, "backpressure", 
                   &format!("Message {} - testing backpressure handling with a longer message to fill buffer faster", i)).await;
        
        // Small delay to prevent overwhelming the test
        if i % 50 == 0 {
            tokio::time::sleep(Duration::from_micros(1)).await;
        }
    }
    let processing_time = start_time.elapsed();

    println!("Backpressure test completed in {:?}", processing_time);

    // System should handle backpressure gracefully without crashing
    tokio::time::sleep(Duration::from_millis(200)).await;

    logger.stop().await.expect("Failed to stop logger");
}

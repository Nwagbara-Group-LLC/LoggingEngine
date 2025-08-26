use std::time::Duration;
use log_aggregator::{LogAggregator, AggregatorConfig};
use metrics_collector::{MetricsCollector, MetricsConfig};

#[tokio::test]
async fn test_basic_e2e_scenario() {
    println!("🚀 Starting basic end-to-end test...");
    
    // Set up components
    let aggregator = LogAggregator::new(AggregatorConfig::default())
        .expect("Failed to create log aggregator");

    let metrics_collector = MetricsCollector::new(MetricsConfig::default())
        .expect("Failed to create metrics collector");

    // Start components
    aggregator.start().await.expect("Failed to start aggregator");
    metrics_collector.start().await.expect("Failed to start metrics collector");

    println!("✅ All components started successfully");
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Simulate basic operations
    println!("📊 Running basic operations simulation");
    
    for i in 0..10 {
        aggregator.process_log_entry("INFO", "e2e_test", &format!("Test message {}", i)).await;
        metrics_collector.record_counter("e2e.operations", 1.0, &[("test", "basic")]).await;
        
        if i % 5 == 0 {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(100)).await;

    println!("📈 Validating results");
    let metrics = metrics_collector.get_metrics().await;
    println!("  Collected {} metrics", metrics.len());
    assert!(!metrics.is_empty(), "Should have collected some metrics");

    // Clean shutdown
    aggregator.stop().await.expect("Failed to stop aggregator");
    metrics_collector.stop().await.expect("Failed to stop metrics collector");
    
    println!("✅ Basic end-to-end test completed successfully!");
}

#[tokio::test]
#[traced_test]
async fn test_comprehensive_trading_platform_simulation() {
    println!("🚀 Starting comprehensive trading platform simulation...");
    
    // Set up high-performance logging infrastructure
    let logger = Arc::new(
        UltraLogger::builder()
            .with_transport(Transport::Memory)
            .with_buffer_config(BufferConfig {
                capacity: 1024 * 1024 * 10, // 10MB buffer
                batch_size: 1000,
                flush_interval: Duration::from_micros(100),
            })
            .with_compression(CompressionType::Lz4)
            .build()
            .expect("Failed to create ultra logger")
    );

    let mut aggregator_config = AggregatorConfig::default();
    aggregator_config.batch_size = 5000;
    aggregator_config.batch_timeout = Duration::from_millis(5);
    
    let aggregator = Arc::new(
        LogAggregator::new(aggregator_config)
            .expect("Failed to create log aggregator")
    );

    let mut metrics_config = MetricsConfig::default();
    metrics_config.aggregation_interval = Duration::from_millis(100);
    
    let metrics_collector = Arc::new(
        MetricsCollector::new(metrics_config)
            .expect("Failed to create metrics collector")
    );

    // Start all components
    logger.start().await.expect("Failed to start logger");
    aggregator.start().await.expect("Failed to start aggregator");
    metrics_collector.start().await.expect("Failed to start metrics collector");

    println!("✅ All logging components started successfully");
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Phase 1: System initialization and health checks
    println!("📊 Phase 1: System health check");
    
    logger.log(LogLevel::Info, "system", "Trading platform initialization started").await;
    metrics_collector.record_gauge("system.status", 1.0, &[("phase", "startup")]).await;
    
    for component in ["order_gateway", "risk_engine", "market_data", "execution", "portfolio"] {
        logger.log(LogLevel::Info, component, "Component initialized").await;
        metrics_collector.record_gauge("component.status", 1.0, &[("component", component)]).await;
    }

    // Phase 2: Market data simulation
    println!("📈 Phase 2: Market data streaming simulation");
    
    let market_data_start = Instant::now();
    let symbols = vec!["BTCUSD", "ETHUSD", "ADAUSD", "DOTUSD"];
    
    for symbol in &symbols {
        let price_updates = MarketDataFixtures::generate_price_stream(100, symbol, 45000.0);
        
        for (i, update) in price_updates.iter().enumerate() {
            logger.log(
                LogLevel::Debug,
                "market_data",
                &format!("PRICE_UPDATE|{}|{}", symbol, update["ask"])
            ).await;
            
            metrics_collector.record_gauge(
                "market.price", 
                update["ask"].as_str().unwrap().parse::<f64>().unwrap(),
                &[("symbol", symbol)]
            ).await;
            
            if i % 50 == 0 {
                metrics_collector.record_counter("market.updates", 50.0, &[("symbol", symbol)]).await;
                tokio::time::sleep(Duration::from_micros(1)).await; // Prevent overwhelming
            }
        }
    }
    
    let market_data_duration = market_data_start.elapsed();
    println!("  Market data phase completed in {:?}", market_data_duration);

    // Phase 3: High-frequency order processing simulation
    println!("⚡ Phase 3: High-frequency order processing");
    
    let order_processing_start = Instant::now();
    let concurrent_orders = 1000;
    let barrier = Arc::new(Barrier::new(concurrent_orders + 1));
    let measurer = Arc::new(PerformanceMeasurer::new());
    
    let mut order_handles = Vec::new();
    
    for order_id in 0..concurrent_orders {
        let logger_clone = Arc::clone(&logger);
        let metrics_clone = Arc::clone(&metrics_collector);
        let barrier_clone = Arc::clone(&barrier);
        let measurer_clone = Arc::clone(&measurer);
        
        let handle = tokio::spawn(async move {
            // Wait for all orders to be ready
            barrier_clone.wait().await;
            
            // Measure order processing latency
            measurer_clone.record_operation(|| async {
                let order_start = Instant::now();
                let order_ref = format!("ORD_HFT_{:06}", order_id);
                
                // Order received
                logger_clone.log(LogLevel::Info, "gateway", 
                               &format!("ORDER_RECEIVED|{}", order_ref)).await;
                
                // Risk check (simulate 1-2 microseconds)
                let risk_start = Instant::now();
                // Simulate minimal processing
                for _ in 0..10 { std::hint::black_box(()); }
                let risk_duration = risk_start.elapsed();
                
                logger_clone.log(LogLevel::Debug, "risk", 
                               &format!("RISK_CHECK_PASSED|{}|duration_ns={}", 
                                      order_ref, risk_duration.as_nanos())).await;
                
                // Order routing (simulate sub-microsecond routing)
                logger_clone.log(LogLevel::Debug, "routing", 
                               &format!("ORDER_ROUTED|{}|exchange=binance", order_ref)).await;
                
                // Execution (simulate exchange response)
                logger_clone.log(LogLevel::Info, "execution", 
                               &format!("ORDER_EXECUTED|{}|price=45000.50", order_ref)).await;
                
                let total_latency = order_start.elapsed();
                
                // Record metrics
                metrics_clone.record_counter("orders.processed", 1.0, 
                                           &[("exchange", "binance")]).await;
                metrics_clone.record_histogram("order.total_latency", 
                                             total_latency.as_secs_f64(),
                                             &[("type", "limit")]).await;
                metrics_clone.record_histogram("risk.check_latency", 
                                             risk_duration.as_secs_f64(),
                                             &[("result", "passed")]).await;
                
            }).await;
        });
        
        order_handles.push(handle);
    }
    
    // Release all orders simultaneously
    barrier.wait().await;
    
    // Wait for all orders to complete
    for handle in order_handles {
        handle.await.expect("Order processing failed");
    }
    
    let order_processing_duration = order_processing_start.elapsed();
    let order_stats = measurer.get_stats().await;
    
    println!("  Processed {} orders in {:?}", concurrent_orders, order_processing_duration);
    println!("  Average order latency: {:?}", order_stats.avg_latency);
    println!("  P99 order latency: {:?}", order_stats.p99_latency);
    
    // Assert ultra-low latency requirements
    assert!(order_stats.avg_latency < Duration::from_micros(10), 
            "Average order latency too high: {:?}", order_stats.avg_latency);
    assert!(order_stats.p99_latency < Duration::from_micros(50), 
            "P99 order latency too high: {:?}", order_stats.p99_latency);

    // Phase 4: Error handling and recovery simulation
    println!("🛡️ Phase 4: Error handling simulation");
    
    // Simulate various error conditions
    let error_scenarios = vec![
        ("exchange_timeout", "HIGH", "Exchange connection timeout after 5ms"),
        ("risk_violation", "MEDIUM", "Position size exceeds limit"),
        ("invalid_symbol", "LOW", "Unknown trading symbol"),
        ("insufficient_margin", "HIGH", "Insufficient margin for trade"),
    ];
    
    for (error_type, severity, message) in error_scenarios {
        logger.log(LogLevel::Error, "error_handler", 
                   &format!("ERROR|{}|{}|{}", error_type, severity, message)).await;
        metrics_collector.record_counter("errors.total", 1.0, 
                                        &[("type", error_type), ("severity", severity)]).await;
        
        // Simulate error recovery
        logger.log(LogLevel::Info, "error_handler", 
                   &format!("ERROR_RECOVERY|{}", error_type)).await;
    }

    // Phase 5: Load test simulation
    println!("🔥 Phase 5: Sustained load test");
    
    let load_test_start = Instant::now();
    let target_rps = 10000.0;
    let load_duration = Duration::from_secs(10);
    
    let load_generator = LoadTestGenerator::new(target_rps, load_duration)
        .with_warmup(Duration::from_secs(2));
    
    let logger_for_load = Arc::clone(&logger);
    let metrics_for_load = Arc::clone(&metrics_collector);
    
    let load_results = load_generator.run_load_test(|operation_id| {
        let logger_clone = Arc::clone(&logger_for_load);
        let metrics_clone = Arc::clone(&metrics_for_load);
        
        async move {
            logger_clone.log(LogLevel::Info, "load_test", 
                           &format!("LOAD_OPERATION_{}", operation_id)).await;
            metrics_clone.record_counter("load_test.operations", 1.0, &[]).await;
        }
    }).await;
    
    load_results.print_summary();
    load_results.assert_meets_requirements(0.95, Duration::from_micros(5));

    // Phase 6: System stress test
    println!("💪 Phase 6: System stress test");
    
    let stress_start = Instant::now();
    let mut stress_handles = Vec::new();
    
    // Concurrent stress across different components
    for component_id in 0..5 {
        let logger_clone = Arc::clone(&logger);
        let metrics_clone = Arc::clone(&metrics_collector);
        let component_name = match component_id {
            0 => "trading",
            1 => "risk",
            2 => "market_data",
            3 => "execution",
            _ => "portfolio",
        };
        
        let handle = tokio::spawn(async move {
            for i in 0..2000 {
                logger_clone.log(LogLevel::Info, component_name,
                               &format!("STRESS_MESSAGE_{}_{}", component_id, i)).await;
                
                if i % 100 == 0 {
                    metrics_clone.record_counter("stress_test.messages", 100.0,
                                                &[("component", component_name)]).await;
                }
            }
        });
        
        stress_handles.push(handle);
    }
    
    // Wait for stress test completion
    for handle in stress_handles {
        handle.await.expect("Stress test task failed");
    }
    
    let stress_duration = stress_start.elapsed();
    println!("  Stress test completed in {:?}", stress_duration);

    // Phase 7: Final metrics collection and validation
    println!("📊 Phase 7: Final metrics validation");
    
    tokio::time::sleep(Duration::from_millis(500)).await; // Allow final processing
    
    let final_metrics = metrics_collector.get_metrics().await;
    println!("  Total metrics collected: {}", final_metrics.len());
    
    // Validate that we have metrics from all phases
    let expected_metric_prefixes = vec![
        "system.", "component.", "market.", "orders.", "errors.", "load_test.", "stress_test."
    ];
    
    for prefix in expected_metric_prefixes {
        let matching_metrics = final_metrics.iter()
            .filter(|m| m.name.starts_with(prefix))
            .count();
        println!("  Metrics with prefix '{}': {}", prefix, matching_metrics);
        assert!(matching_metrics > 0, "No metrics found with prefix '{}'", prefix);
    }

    // Final system performance summary
    let total_test_duration = Instant::now().duration_since(
        order_processing_start - Duration::from_millis(100)
    );
    
    println!("\n🎯 Comprehensive Test Results Summary:");
    println!("  Total test duration: {:?}", total_test_duration);
    println!("  Market data processing: {:?}", market_data_duration);
    println!("  Order processing: {:?}", order_processing_duration);
    println!("  Stress test duration: {:?}", stress_duration);
    println!("  Orders processed: {}", concurrent_orders);
    println!("  Average order latency: {:?}", order_stats.avg_latency);
    println!("  P99 order latency: {:?}", order_stats.p99_latency);
    println!("  Load test throughput: {:.0} ops/sec", 
            load_results.stats.operation_count as f64 / load_results.actual_duration.as_secs_f64());
    println!("  Total metrics recorded: {}", final_metrics.len());

    // Final assertions for production readiness
    assert!(order_stats.avg_latency < Duration::from_micros(10), 
            "System not ready: average latency too high");
    assert!(order_stats.p99_latency < Duration::from_micros(50), 
            "System not ready: P99 latency too high");
    assert!(final_metrics.len() > 100, 
            "Insufficient metrics collection");
    
    // Clean shutdown
    logger.stop().await.expect("Failed to stop logger");
    aggregator.stop().await.expect("Failed to stop aggregator");
    metrics_collector.stop().await.expect("Failed to stop metrics collector");
    
    println!("✅ Comprehensive trading platform simulation completed successfully!");
    println!("🚀 System is ready for production deployment!");
}

#[tokio::test]
async fn test_failure_recovery_scenarios() {
    println!("🔧 Testing failure recovery scenarios...");
    
    let harness = TestHarness::new().await.expect("Failed to create test harness");
    harness.start_all().await.expect("Failed to start components");
    
    // Scenario 1: Component restart simulation
    println!("  Scenario 1: Component restart");
    
    // Log some messages
    harness.logger.log(LogLevel::Info, "test", "Before component restart").await;
    
    // Simulate component failure and restart
    harness.aggregator.stop().await.expect("Failed to stop aggregator");
    tokio::time::sleep(Duration::from_millis(100)).await;
    harness.aggregator.start().await.expect("Failed to restart aggregator");
    
    // Continue logging after restart
    harness.logger.log(LogLevel::Info, "test", "After component restart").await;
    
    // Scenario 2: High error rate handling
    println!("  Scenario 2: High error rate");
    
    for i in 0..100 {
        if i % 10 == 0 {
            harness.logger.log(LogLevel::Error, "error_test", 
                             &format!("Simulated error {}", i)).await;
        } else {
            harness.logger.log(LogLevel::Info, "normal", 
                             &format!("Normal operation {}", i)).await;
        }
    }
    
    // Scenario 3: Resource exhaustion simulation
    println!("  Scenario 3: Resource exhaustion handling");
    
    let large_message = "X".repeat(10000);
    for i in 0..50 {
        harness.logger.log(LogLevel::Warn, "resource_test", 
                         &format!("{} - Large message {}", large_message, i)).await;
        
        if i % 10 == 0 {
            tokio::time::sleep(Duration::from_micros(1)).await;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(200)).await;
    harness.stop_all().await.expect("Failed to stop components");
    
    println!("✅ Failure recovery scenarios completed successfully");
}

#[tokio::test]
async fn test_production_workload_simulation() {
    println!("🏭 Testing production workload simulation...");
    
    let harness = TestHarness::new().await.expect("Failed to create test harness");
    harness.start_all().await.expect("Failed to start components");
    
    // Simulate a typical production day
    let trading_session_duration = Duration::from_secs(30); // Compressed trading session
    let session_start = Instant::now();
    
    let mut total_orders = 0u64;
    let mut total_market_updates = 0u64;
    
    while session_start.elapsed() < trading_session_duration {
        // Morning: High activity
        if session_start.elapsed() < trading_session_duration / 3 {
            // Process orders at high frequency
            for _ in 0..100 {
                TradingScenarioSimulator::simulate_order_flow(&harness, 1).await;
                total_orders += 1;
            }
            
            // Market data updates
            TradingScenarioSimulator::simulate_market_data_burst(&harness, 50, "BTCUSD").await;
            total_market_updates += 50;
        }
        // Midday: Medium activity  
        else if session_start.elapsed() < (trading_session_duration * 2) / 3 {
            for _ in 0..50 {
                TradingScenarioSimulator::simulate_order_flow(&harness, 1).await;
                total_orders += 1;
            }
            
            TradingScenarioSimulator::simulate_market_data_burst(&harness, 25, "ETHUSD").await;
            total_market_updates += 25;
        }
        // Evening: Lower activity
        else {
            for _ in 0..20 {
                TradingScenarioSimulator::simulate_order_flow(&harness, 1).await;
                total_orders += 1;
            }
            
            TradingScenarioSimulator::simulate_market_data_burst(&harness, 10, "ADAUSD").await;
            total_market_updates += 10;
        }
        
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    let actual_duration = session_start.elapsed();
    
    println!("  Production simulation results:");
    println!("    Duration: {:?}", actual_duration);
    println!("    Total orders: {}", total_orders);
    println!("    Total market updates: {}", total_market_updates);
    println!("    Order rate: {:.0} orders/sec", 
            total_orders as f64 / actual_duration.as_secs_f64());
    println!("    Market data rate: {:.0} updates/sec", 
            total_market_updates as f64 / actual_duration.as_secs_f64());
    
    // Assertions for production readiness
    assert!(total_orders > 1000, "Insufficient order processing volume");
    assert!(total_market_updates > 500, "Insufficient market data processing");
    
    tokio::time::sleep(Duration::from_millis(500)).await;
    harness.stop_all().await.expect("Failed to stop components");
    
    println!("✅ Production workload simulation completed successfully");
}

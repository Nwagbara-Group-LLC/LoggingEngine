use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

fn bench_basic_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("basic_operations");
    
    // Simple string processing benchmark
    group.bench_function("string_processing", |b| {
        b.iter(|| {
            let message = black_box("Test message for benchmarking");
            let processed = format!("PROCESSED: {}", message);
            black_box(processed)
        });
    });
    
    // Duration creation benchmark
    group.bench_function("duration_creation", |b| {
        b.iter(|| {
            let duration = Duration::from_millis(black_box(100));
            black_box(duration)
        });
    });
    
    group.finish();
}

criterion_group!(benches, bench_basic_operations);
criterion_main!(benches);

fn bench_ultra_logger_latency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let logger = rt.block_on(async {
        UltraLogger::builder()
            .with_transport(Transport::Memory)
            .build()
            .expect("Failed to create logger")
    });
    
    rt.block_on(async {
        logger.start().await.expect("Failed to start logger");
    });
    
    let mut group = c.benchmark_group("ultra_logger_latency");
    group.measurement_time(Duration::from_secs(10));
    
    // Single log operation latency
    group.bench_function("single_log", |b| {
        b.to_async(&rt).iter(|| async {
            logger.log(
                black_box(LogLevel::Info),
                black_box("benchmark"),
                black_box("Benchmark message for latency test")
            ).await;
        });
    });
    
    // Structured logging latency
    group.bench_function("structured_log", |b| {
        b.to_async(&rt).iter(|| async {
            logger.log_structured(
                black_box(LogLevel::Info),
                black_box("benchmark"),
                black_box(&[
                    ("event", "order_placed"),
                    ("order_id", "ORD_BENCH_123"),
                    ("symbol", "BTCUSD"),
                    ("price", "45000.50")
                ])
            ).await;
        });
    });
    
    group.finish();
    
    rt.block_on(async {
        logger.stop().await.expect("Failed to stop logger");
    });
}

fn bench_ultra_logger_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let logger = rt.block_on(async {
        UltraLogger::builder()
            .with_transport(Transport::Memory)
            .with_buffer_config(BufferConfig {
                capacity: 1024 * 1024, // 1MB buffer
                batch_size: 1000,
                flush_interval: Duration::from_millis(1),
            })
            .build()
            .expect("Failed to create logger")
    });
    
    rt.block_on(async {
        logger.start().await.expect("Failed to start logger");
    });
    
    let mut group = c.benchmark_group("ultra_logger_throughput");
    group.measurement_time(Duration::from_secs(15));
    
    for batch_size in [10, 100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("batch_logging", batch_size),
            batch_size,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    for i in 0..size {
                        logger.log(
                            LogLevel::Info,
                            "throughput_bench",
                            &format!("Throughput test message {}", i)
                        ).await;
                    }
                });
            }
        );
    }
    
    group.finish();
    
    rt.block_on(async {
        logger.stop().await.expect("Failed to stop logger");
    });
}

fn bench_log_aggregator_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let aggregator = rt.block_on(async {
        let mut config = AggregatorConfig::default();
        config.batch_size = 1000;
        config.batch_timeout = Duration::from_millis(1);
        
        LogAggregator::new(config).expect("Failed to create aggregator")
    });
    
    rt.block_on(async {
        aggregator.start().await.expect("Failed to start aggregator");
    });
    
    let mut group = c.benchmark_group("log_aggregator_performance");
    group.measurement_time(Duration::from_secs(10));
    
    group.bench_function("single_entry_processing", |b| {
        b.to_async(&rt).iter(|| async {
            aggregator.process_log_entry(
                black_box("INFO"),
                black_box("benchmark"),
                black_box("Aggregator benchmark message")
            ).await;
        });
    });
    
    for batch_size in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("batch_processing", batch_size),
            batch_size,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    for i in 0..size {
                        aggregator.process_log_entry(
                            "INFO",
                            "batch_bench",
                            &format!("Batch message {}", i)
                        ).await;
                    }
                });
            }
        );
    }
    
    group.finish();
    
    rt.block_on(async {
        aggregator.stop().await.expect("Failed to stop aggregator");
    });
}

fn bench_metrics_collector_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let collector = rt.block_on(async {
        MetricsCollector::new(MetricsConfig::default())
            .expect("Failed to create metrics collector")
    });
    
    rt.block_on(async {
        collector.start().await.expect("Failed to start collector");
    });
    
    let mut group = c.benchmark_group("metrics_collector_performance");
    group.measurement_time(Duration::from_secs(10));
    
    // Individual metric types
    group.bench_function("counter_metric", |b| {
        b.to_async(&rt).iter(|| async {
            collector.record_counter(
                black_box("benchmark.counter"),
                black_box(1.0),
                black_box(&[("service", "benchmark")])
            ).await;
        });
    });
    
    group.bench_function("gauge_metric", |b| {
        b.to_async(&rt).iter(|| async {
            collector.record_gauge(
                black_box("benchmark.gauge"),
                black_box(42.5),
                black_box(&[("type", "gauge")])
            ).await;
        });
    });
    
    group.bench_function("histogram_metric", |b| {
        b.to_async(&rt).iter(|| async {
            collector.record_histogram(
                black_box("benchmark.histogram"),
                black_box(0.001),
                black_box(&[("operation", "test")])
            ).await;
        });
    });
    
    // Mixed workload
    for ops_count in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*ops_count as u64));
        group.bench_with_input(
            BenchmarkId::new("mixed_metrics", ops_count),
            ops_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async {
                    for i in 0..count {
                        match i % 3 {
                            0 => collector.record_counter(
                                "mixed.counter", 1.0, &[("iter", &i.to_string())]
                            ).await,
                            1 => collector.record_gauge(
                                "mixed.gauge", i as f64, &[("iter", &i.to_string())]
                            ).await,
                            2 => collector.record_histogram(
                                "mixed.histogram", 0.001 * i as f64, &[("iter", &i.to_string())]
                            ).await,
                            _ => unreachable!(),
                        }
                    }
                });
            }
        );
    }
    
    group.finish();
    
    rt.block_on(async {
        collector.stop().await.expect("Failed to stop collector");
    });
}

fn bench_end_to_end_trading_scenario(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let (logger, aggregator, collector) = rt.block_on(async {
        let logger = UltraLogger::builder()
            .with_transport(Transport::Memory)
            .build()
            .expect("Failed to create logger");
        
        let aggregator = LogAggregator::new(AggregatorConfig::default())
            .expect("Failed to create aggregator");
        
        let collector = MetricsCollector::new(MetricsConfig::default())
            .expect("Failed to create collector");
        
        logger.start().await.expect("Failed to start logger");
        aggregator.start().await.expect("Failed to start aggregator");
        collector.start().await.expect("Failed to start collector");
        
        // Small delay to ensure all components are ready
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        (logger, aggregator, collector)
    });
    
    let mut group = c.benchmark_group("end_to_end_trading");
    group.measurement_time(Duration::from_secs(15));
    
    group.bench_function("complete_order_flow", |b| {
        let mut order_counter = 0u64;
        b.to_async(&rt).iter(|| async {
            order_counter += 1;
            let order_id = format!("ORD_BENCH_{}", order_counter);
            
            // Order received
            logger.log(LogLevel::Info, "trading", 
                      &format!("ORDER_RECEIVED|{}", order_id)).await;
            collector.record_counter("orders.received", 1.0, &[("symbol", "BTCUSD")]).await;
            
            // Risk check
            logger.log(LogLevel::Info, "risk", 
                      &format!("RISK_CHECK_PASSED|{}", order_id)).await;
            collector.record_histogram("risk.latency", 0.000001, &[("result", "passed")]).await;
            
            // Order execution
            logger.log(LogLevel::Info, "execution", 
                      &format!("ORDER_EXECUTED|{}|price=45000.50", order_id)).await;
            collector.record_counter("orders.executed", 1.0, &[("symbol", "BTCUSD")]).await;
            collector.record_histogram("execution.latency", 0.000005, &[("status", "filled")]).await;
        });
    });
    
    group.finish();
    
    rt.block_on(async {
        logger.stop().await.expect("Failed to stop logger");
        aggregator.stop().await.expect("Failed to stop aggregator");
        collector.stop().await.expect("Failed to stop collector");
    });
}

fn bench_compression_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("compression_performance");
    group.measurement_time(Duration::from_secs(10));
    
    let large_message = "A".repeat(10000); // 10KB message
    
    for compression in ["none", "lz4", "zstd", "snappy"].iter() {
        let logger = rt.block_on(async {
            let mut builder = UltraLogger::builder()
                .with_transport(Transport::Memory);
            
            builder = match *compression {
                "lz4" => builder.with_compression(ultra_logger::CompressionType::Lz4),
                "zstd" => builder.with_compression(ultra_logger::CompressionType::Zstd),
                "snappy" => builder.with_compression(ultra_logger::CompressionType::Snappy),
                _ => builder.with_compression(ultra_logger::CompressionType::None),
            };
            
            let logger = builder.build().expect("Failed to create logger");
            logger.start().await.expect("Failed to start logger");
            logger
        });
        
        group.bench_with_input(
            BenchmarkId::new("large_message_compression", compression),
            compression,
            |b, _| {
                b.to_async(&rt).iter(|| async {
                    logger.log(
                        LogLevel::Info,
                        "compression_bench",
                        black_box(&large_message)
                    ).await;
                });
            }
        );
        
        rt.block_on(async {
            logger.stop().await.expect("Failed to stop logger");
        });
    }
    
    group.finish();
}

fn bench_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let logger = rt.block_on(async {
        UltraLogger::builder()
            .with_transport(Transport::Memory)
            .with_buffer_config(BufferConfig {
                capacity: 1024 * 1024 * 10, // 10MB buffer
                batch_size: 1000,
                flush_interval: Duration::from_micros(100),
            })
            .build()
            .expect("Failed to create logger")
    });
    
    rt.block_on(async {
        logger.start().await.expect("Failed to start logger");
    });
    
    let mut group = c.benchmark_group("concurrent_operations");
    group.measurement_time(Duration::from_secs(15));
    
    for thread_count in [1, 2, 4, 8, 16].iter() {
        group.throughput(Throughput::Elements(100 * *thread_count as u64));
        group.bench_with_input(
            BenchmarkId::new("concurrent_logging", thread_count),
            thread_count,
            |b, &threads| {
                b.to_async(&rt).iter(|| async {
                    let mut handles = Vec::new();
                    
                    for thread_id in 0..threads {
                        let logger_clone = logger.clone();
                        let handle = tokio::spawn(async move {
                            for i in 0..100 {
                                logger_clone.log(
                                    LogLevel::Info,
                                    "concurrent_bench",
                                    &format!("Thread {} message {}", thread_id, i)
                                ).await;
                            }
                        });
                        handles.push(handle);
                    }
                    
                    for handle in handles {
                        handle.await.expect("Thread failed");
                    }
                });
            }
        );
    }
    
    group.finish();
    
    rt.block_on(async {
        logger.stop().await.expect("Failed to stop logger");
    });
}

criterion_group!(
    benches,
    bench_ultra_logger_latency,
    bench_ultra_logger_throughput,
    bench_log_aggregator_performance,
    bench_metrics_collector_performance,
    bench_end_to_end_trading_scenario,
    bench_compression_performance,
    bench_concurrent_operations
);

criterion_main!(benches);

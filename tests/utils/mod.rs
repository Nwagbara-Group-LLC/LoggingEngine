use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::{Barrier, Mutex};
use ultra_logger::{UltraLogger, LogLevel};
use log_aggregator::LogAggregator;
use metrics_collector::MetricsCollector;

/// Test utilities for the logging engine
pub struct TestHarness {
    pub logger: Arc<UltraLogger>,
    pub aggregator: Arc<LogAggregator>,
    pub metrics_collector: Arc<MetricsCollector>,
    pub start_time: Instant,
}

impl TestHarness {
    pub async fn new() -> anyhow::Result<Self> {
        let logger = Arc::new(
            UltraLogger::builder()
                .with_transport(ultra_logger::Transport::Memory)
                .build()?
        );

        let aggregator = Arc::new(
            LogAggregator::new(log_aggregator::AggregatorConfig::default())?
        );

        let metrics_collector = Arc::new(
            MetricsCollector::new(metrics_collector::MetricsConfig::default())?
        );

        Ok(Self {
            logger,
            aggregator,
            metrics_collector,
            start_time: Instant::now(),
        })
    }

    pub async fn start_all(&self) -> anyhow::Result<()> {
        self.logger.start().await?;
        self.aggregator.start().await?;
        self.metrics_collector.start().await?;
        
        // Small delay to ensure all components are ready
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        Ok(())
    }

    pub async fn stop_all(&self) -> anyhow::Result<()> {
        self.logger.stop().await?;
        self.aggregator.stop().await?;
        self.metrics_collector.stop().await?;
        Ok(())
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

/// Performance measurement utilities
pub struct PerformanceMeasurer {
    operation_count: AtomicU64,
    total_latency: Arc<Mutex<Duration>>,
    min_latency: Arc<Mutex<Duration>>,
    max_latency: Arc<Mutex<Duration>>,
    latencies: Arc<Mutex<Vec<Duration>>>,
}

impl PerformanceMeasurer {
    pub fn new() -> Self {
        Self {
            operation_count: AtomicU64::new(0),
            total_latency: Arc::new(Mutex::new(Duration::ZERO)),
            min_latency: Arc::new(Mutex::new(Duration::MAX)),
            max_latency: Arc::new(Mutex::new(Duration::ZERO)),
            latencies: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn record_operation<F, Fut, R>(&self, operation: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let start = Instant::now();
        let result = operation().await;
        let latency = start.elapsed();

        self.operation_count.fetch_add(1, Ordering::Relaxed);
        
        let mut total = self.total_latency.lock().await;
        *total += latency;
        
        let mut min = self.min_latency.lock().await;
        if latency < *min {
            *min = latency;
        }
        
        let mut max = self.max_latency.lock().await;
        if latency > *max {
            *max = latency;
        }
        
        let mut latencies = self.latencies.lock().await;
        latencies.push(latency);

        result
    }

    pub async fn get_stats(&self) -> PerformanceStats {
        let count = self.operation_count.load(Ordering::Relaxed);
        let total = *self.total_latency.lock().await;
        let min = *self.min_latency.lock().await;
        let max = *self.max_latency.lock().await;
        
        let mut latencies = self.latencies.lock().await;
        latencies.sort();
        
        let avg = if count > 0 {
            total / count as u32
        } else {
            Duration::ZERO
        };

        let p50 = if !latencies.is_empty() {
            latencies[latencies.len() / 2]
        } else {
            Duration::ZERO
        };

        let p95 = if !latencies.is_empty() {
            latencies[(latencies.len() * 95) / 100]
        } else {
            Duration::ZERO
        };

        let p99 = if !latencies.is_empty() {
            latencies[(latencies.len() * 99) / 100]
        } else {
            Duration::ZERO
        };

        PerformanceStats {
            operation_count: count,
            avg_latency: avg,
            min_latency: min,
            max_latency: max,
            p50_latency: p50,
            p95_latency: p95,
            p99_latency: p99,
        }
    }

    pub fn reset(&self) {
        self.operation_count.store(0, Ordering::Relaxed);
        // Note: We can't easily reset the mutexes without async, but this is primarily for testing
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub operation_count: u64,
    pub avg_latency: Duration,
    pub min_latency: Duration,
    pub max_latency: Duration,
    pub p50_latency: Duration,
    pub p95_latency: Duration,
    pub p99_latency: Duration,
}

impl PerformanceStats {
    pub fn print_summary(&self) {
        println!("Performance Summary:");
        println!("  Operations: {}", self.operation_count);
        println!("  Average Latency: {:?}", self.avg_latency);
        println!("  Min Latency: {:?}", self.min_latency);
        println!("  Max Latency: {:?}", self.max_latency);
        println!("  P50 Latency: {:?}", self.p50_latency);
        println!("  P95 Latency: {:?}", self.p95_latency);
        println!("  P99 Latency: {:?}", self.p99_latency);
    }

    pub fn assert_ultra_low_latency(&self) {
        assert!(self.avg_latency < Duration::from_micros(1), 
                "Average latency too high: {:?}", self.avg_latency);
        assert!(self.p99_latency < Duration::from_micros(5), 
                "P99 latency too high: {:?}", self.p99_latency);
    }

    pub fn assert_high_throughput(&self, min_ops_per_second: f64, duration: Duration) {
        let throughput = self.operation_count as f64 / duration.as_secs_f64();
        assert!(throughput >= min_ops_per_second, 
                "Throughput too low: {:.0} ops/s (expected >= {:.0})", 
                throughput, min_ops_per_second);
    }
}

/// Concurrent test coordinator
pub struct ConcurrentTestCoordinator {
    barrier: Arc<Barrier>,
    completed_tasks: AtomicU64,
}

impl ConcurrentTestCoordinator {
    pub fn new(task_count: usize) -> Self {
        Self {
            barrier: Arc::new(Barrier::new(task_count + 1)), // +1 for coordinator
            completed_tasks: AtomicU64::new(0),
        }
    }

    pub async fn spawn_coordinated_task<F, Fut>(&self, task: F) -> tokio::task::JoinHandle<()>
    where
        F: FnOnce(Arc<Barrier>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let barrier = Arc::clone(&self.barrier);
        let completed_counter = Arc::new(&self.completed_tasks);
        
        tokio::spawn(async move {
            // Wait for all tasks to be ready
            barrier.wait().await;
            
            // Execute the task
            task(barrier).await;
            
            // Mark task as completed
            completed_counter.fetch_add(1, Ordering::Relaxed);
        })
    }

    pub async fn start_all_tasks(&self) {
        // Release all waiting tasks
        self.barrier.wait().await;
    }

    pub async fn wait_for_completion(&self, expected_tasks: usize, timeout: Duration) -> bool {
        let start = Instant::now();
        
        while start.elapsed() < timeout {
            if self.completed_tasks.load(Ordering::Relaxed) >= expected_tasks as u64 {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        
        false
    }
}

/// Load test generator
pub struct LoadTestGenerator {
    rps: f64, // requests per second
    duration: Duration,
    warmup_duration: Duration,
}

impl LoadTestGenerator {
    pub fn new(rps: f64, duration: Duration) -> Self {
        Self {
            rps,
            duration,
            warmup_duration: Duration::from_secs(1),
        }
    }

    pub fn with_warmup(mut self, warmup_duration: Duration) -> Self {
        self.warmup_duration = warmup_duration;
        self
    }

    pub async fn run_load_test<F, Fut>(&self, operation: F) -> LoadTestResults
    where
        F: Fn(u64) -> Fut + Clone + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let interval = Duration::from_secs_f64(1.0 / self.rps);
        let measurer = Arc::new(PerformanceMeasurer::new());
        
        println!("Starting load test: {:.0} RPS for {:?} (warmup: {:?})", 
                self.rps, self.duration, self.warmup_duration);

        // Warmup phase
        let warmup_ops = (self.rps * self.warmup_duration.as_secs_f64()) as u64;
        for i in 0..warmup_ops {
            let op = operation.clone();
            tokio::spawn(async move {
                op(i).await;
            });
            tokio::time::sleep(interval).await;
        }

        println!("Warmup complete, starting measurement phase");

        // Measurement phase
        let test_start = Instant::now();
        let target_ops = (self.rps * self.duration.as_secs_f64()) as u64;
        
        for i in 0..target_ops {
            let op = operation.clone();
            let measurer_clone = Arc::clone(&measurer);
            
            tokio::spawn(async move {
                measurer_clone.record_operation(|| op(i)).await;
            });
            
            tokio::time::sleep(interval).await;
        }

        // Wait for all operations to complete
        let mut wait_time = Duration::ZERO;
        let max_wait = Duration::from_secs(30);
        
        while measurer.operation_count.load(Ordering::Relaxed) < target_ops && wait_time < max_wait {
            tokio::time::sleep(Duration::from_millis(100)).await;
            wait_time += Duration::from_millis(100);
        }

        let actual_duration = test_start.elapsed();
        let stats = measurer.get_stats().await;

        LoadTestResults {
            target_rps: self.rps,
            actual_duration,
            stats,
        }
    }
}

#[derive(Debug)]
pub struct LoadTestResults {
    pub target_rps: f64,
    pub actual_duration: Duration,
    pub stats: PerformanceStats,
}

impl LoadTestResults {
    pub fn print_summary(&self) {
        println!("Load Test Results:");
        println!("  Target RPS: {:.0}", self.target_rps);
        println!("  Actual Duration: {:?}", self.actual_duration);
        println!("  Actual RPS: {:.0}", 
                self.stats.operation_count as f64 / self.actual_duration.as_secs_f64());
        self.stats.print_summary();
    }

    pub fn assert_meets_requirements(&self, min_success_rate: f64, max_avg_latency: Duration) {
        let actual_rps = self.stats.operation_count as f64 / self.actual_duration.as_secs_f64();
        let success_rate = actual_rps / self.target_rps;
        
        assert!(success_rate >= min_success_rate, 
                "Success rate too low: {:.1}% (expected >= {:.1}%)", 
                success_rate * 100.0, min_success_rate * 100.0);
        
        assert!(self.stats.avg_latency <= max_avg_latency, 
                "Average latency too high: {:?} (expected <= {:?})", 
                self.stats.avg_latency, max_avg_latency);
    }
}

/// Trading scenario simulator
pub struct TradingScenarioSimulator;

impl TradingScenarioSimulator {
    pub async fn simulate_order_flow(
        harness: &TestHarness,
        order_count: u64,
    ) -> Vec<Duration> {
        let mut order_latencies = Vec::new();
        
        for i in 0..order_count {
            let order_id = format!("SIM_ORD_{}", i);
            let start_time = Instant::now();
            
            // Order received
            harness.logger.log(LogLevel::Info, "trading", 
                             &format!("ORDER_RECEIVED|{}", order_id)).await;
            harness.metrics_collector.record_counter("orders.received", 1.0, 
                                                   &[("symbol", "BTCUSD")]).await;
            
            // Risk validation
            harness.logger.log(LogLevel::Info, "risk", 
                             &format!("RISK_VALIDATION|{}", order_id)).await;
            harness.metrics_collector.record_histogram("risk.latency", 0.000001, 
                                                     &[("result", "passed")]).await;
            
            // Order execution
            harness.logger.log(LogLevel::Info, "execution", 
                             &format!("ORDER_EXECUTED|{}", order_id)).await;
            harness.metrics_collector.record_counter("orders.executed", 1.0, 
                                                   &[("symbol", "BTCUSD")]).await;
            
            let order_latency = start_time.elapsed();
            order_latencies.push(order_latency);
        }
        
        order_latencies
    }

    pub async fn simulate_market_data_burst(
        harness: &TestHarness,
        burst_size: u64,
        symbol: &str,
    ) {
        for i in 0..burst_size {
            let price = 45000.0 + (i as f64 * 0.01);
            
            harness.logger.log(LogLevel::Info, "market_data", 
                             &format!("PRICE_UPDATE|{}|{}", symbol, price)).await;
            harness.metrics_collector.record_gauge("market.price", price, 
                                                 &[("symbol", symbol)]).await;
            harness.metrics_collector.record_counter("market.updates", 1.0, 
                                                   &[("symbol", symbol)]).await;
        }
    }
}

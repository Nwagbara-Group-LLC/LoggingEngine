//! Metrics collection and reporting

use crate::error::{LoggingError, Result};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use dashmap::DashMap;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct LoggingMetrics {
    // Performance Metrics
    pub entries_logged: AtomicU64,
    pub entries_dropped: AtomicU64,
    pub bytes_logged: AtomicU64,
    pub flush_count: AtomicU64,
    pub error_count: AtomicU64,
    
    // Latency Metrics (microseconds)
    pub avg_log_latency_us: AtomicU64,
    pub p99_log_latency_us: AtomicU64,
    pub max_log_latency_us: AtomicU64,
    
    // Buffer Metrics
    pub buffer_utilization: AtomicU64, // percentage
    pub buffer_overflow_count: AtomicU64,
    
    // Transport Metrics
    pub transport_send_count: AtomicU64,
    pub transport_error_count: AtomicU64,
    
    // Level-specific Metrics
    level_counts: Arc<DashMap<String, AtomicU64>>,
    
    // Custom Metrics
    custom_counters: Arc<DashMap<String, AtomicU64>>,
    custom_gauges: Arc<DashMap<String, AtomicU64>>,
    
    // Historical data for percentile calculations
    latency_histogram: Arc<RwLock<Vec<u64>>>,
}

impl Default for LoggingMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl LoggingMetrics {
    pub fn new() -> Self {
        Self {
            entries_logged: AtomicU64::new(0),
            entries_dropped: AtomicU64::new(0),
            bytes_logged: AtomicU64::new(0),
            flush_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            avg_log_latency_us: AtomicU64::new(0),
            p99_log_latency_us: AtomicU64::new(0),
            max_log_latency_us: AtomicU64::new(0),
            buffer_utilization: AtomicU64::new(0),
            buffer_overflow_count: AtomicU64::new(0),
            transport_send_count: AtomicU64::new(0),
            transport_error_count: AtomicU64::new(0),
            level_counts: Arc::new(DashMap::new()),
            custom_counters: Arc::new(DashMap::new()),
            custom_gauges: Arc::new(DashMap::new()),
            latency_histogram: Arc::new(RwLock::new(Vec::with_capacity(10000))),
        }
    }
    
    // Performance Metrics
    pub fn increment_entries_logged(&self) {
        self.entries_logged.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn increment_entries_dropped(&self) {
        self.entries_dropped.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn add_bytes_logged(&self, bytes: u64) {
        self.bytes_logged.fetch_add(bytes, Ordering::Relaxed);
    }
    
    pub fn increment_flush_count(&self) {
        self.flush_count.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn increment_error_count(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }
    
    // Latency Metrics
    pub fn record_log_latency(&self, latency_us: u64) {
        // Update max latency
        loop {
            let current_max = self.max_log_latency_us.load(Ordering::Relaxed);
            if latency_us <= current_max || 
               self.max_log_latency_us.compare_exchange_weak(
                   current_max, 
                   latency_us, 
                   Ordering::Relaxed, 
                   Ordering::Relaxed
               ).is_ok() {
                break;
            }
        }
        
        // Add to histogram for percentile calculations (with sampling)
        if fastrand::u32(..100) < 10 { // 10% sampling to avoid memory bloat
            let mut histogram = self.latency_histogram.write();
            if histogram.len() < 10000 {
                histogram.push(latency_us);
            } else {
                // Replace random entry to maintain sampling
                let idx = fastrand::usize(..histogram.len());
                histogram[idx] = latency_us;
            }
        }
    }
    
    pub fn calculate_percentiles(&self) {
        let histogram = self.latency_histogram.read();
        if histogram.is_empty() {
            return;
        }
        
        let mut sorted = histogram.clone();
        sorted.sort_unstable();
        
        let p99_idx = (sorted.len() * 99) / 100;
        let p99 = sorted.get(p99_idx).copied().unwrap_or(0);
        self.p99_log_latency_us.store(p99, Ordering::Relaxed);
        
        let avg = sorted.iter().sum::<u64>() / sorted.len() as u64;
        self.avg_log_latency_us.store(avg, Ordering::Relaxed);
    }
    
    // Buffer Metrics
    pub fn set_buffer_utilization(&self, utilization_percent: u64) {
        self.buffer_utilization.store(utilization_percent, Ordering::Relaxed);
    }
    
    pub fn increment_buffer_overflow(&self) {
        self.buffer_overflow_count.fetch_add(1, Ordering::Relaxed);
    }
    
    // Transport Metrics
    pub fn increment_transport_send(&self) {
        self.transport_send_count.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn increment_transport_error(&self) {
        self.transport_error_count.fetch_add(1, Ordering::Relaxed);
    }
    
    // Level-specific Metrics
    pub fn increment_level_count(&self, level: &str) {
        self.level_counts
            .entry(level.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn get_level_count(&self, level: &str) -> u64 {
        self.level_counts
            .get(level)
            .map(|counter| counter.load(Ordering::Relaxed))
            .unwrap_or(0)
    }
    
    // Custom Metrics
    pub fn increment_counter(&self, name: &str) {
        self.custom_counters
            .entry(name.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn add_to_counter(&self, name: &str, value: u64) {
        self.custom_counters
            .entry(name.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(value, Ordering::Relaxed);
    }
    
    pub fn set_gauge(&self, name: &str, value: u64) {
        self.custom_gauges
            .entry(name.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .store(value, Ordering::Relaxed);
    }
    
    pub fn get_counter(&self, name: &str) -> u64 {
        self.custom_counters
            .get(name)
            .map(|counter| counter.load(Ordering::Relaxed))
            .unwrap_or(0)
    }
    
    pub fn get_gauge(&self, name: &str) -> u64 {
        self.custom_gauges
            .get(name)
            .map(|gauge| gauge.load(Ordering::Relaxed))
            .unwrap_or(0)
    }
    
    // Summary Methods
    pub fn get_summary(&self) -> MetricsSummary {
        self.calculate_percentiles();
        
        let mut level_summary = HashMap::new();
        for entry in self.level_counts.iter() {
            level_summary.insert(entry.key().clone(), entry.value().load(Ordering::Relaxed));
        }
        
        let mut custom_counter_summary = HashMap::new();
        for entry in self.custom_counters.iter() {
            custom_counter_summary.insert(entry.key().clone(), entry.value().load(Ordering::Relaxed));
        }
        
        let mut custom_gauge_summary = HashMap::new();
        for entry in self.custom_gauges.iter() {
            custom_gauge_summary.insert(entry.key().clone(), entry.value().load(Ordering::Relaxed));
        }
        
        MetricsSummary {
            entries_logged: self.entries_logged.load(Ordering::Relaxed),
            entries_dropped: self.entries_dropped.load(Ordering::Relaxed),
            bytes_logged: self.bytes_logged.load(Ordering::Relaxed),
            flush_count: self.flush_count.load(Ordering::Relaxed),
            error_count: self.error_count.load(Ordering::Relaxed),
            avg_log_latency_us: self.avg_log_latency_us.load(Ordering::Relaxed),
            p99_log_latency_us: self.p99_log_latency_us.load(Ordering::Relaxed),
            max_log_latency_us: self.max_log_latency_us.load(Ordering::Relaxed),
            buffer_utilization: self.buffer_utilization.load(Ordering::Relaxed),
            buffer_overflow_count: self.buffer_overflow_count.load(Ordering::Relaxed),
            transport_send_count: self.transport_send_count.load(Ordering::Relaxed),
            transport_error_count: self.transport_error_count.load(Ordering::Relaxed),
            level_counts: level_summary,
            custom_counters: custom_counter_summary,
            custom_gauges: custom_gauge_summary,
        }
    }
    
    pub fn reset_metrics(&self) {
        self.entries_logged.store(0, Ordering::Relaxed);
        self.entries_dropped.store(0, Ordering::Relaxed);
        self.bytes_logged.store(0, Ordering::Relaxed);
        self.flush_count.store(0, Ordering::Relaxed);
        self.error_count.store(0, Ordering::Relaxed);
        self.avg_log_latency_us.store(0, Ordering::Relaxed);
        self.p99_log_latency_us.store(0, Ordering::Relaxed);
        self.max_log_latency_us.store(0, Ordering::Relaxed);
        self.buffer_utilization.store(0, Ordering::Relaxed);
        self.buffer_overflow_count.store(0, Ordering::Relaxed);
        self.transport_send_count.store(0, Ordering::Relaxed);
        self.transport_error_count.store(0, Ordering::Relaxed);
        
        self.level_counts.clear();
        self.custom_counters.clear();
        self.custom_gauges.clear();
        self.latency_histogram.write().clear();
    }
}

#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub entries_logged: u64,
    pub entries_dropped: u64,
    pub bytes_logged: u64,
    pub flush_count: u64,
    pub error_count: u64,
    pub avg_log_latency_us: u64,
    pub p99_log_latency_us: u64,
    pub max_log_latency_us: u64,
    pub buffer_utilization: u64,
    pub buffer_overflow_count: u64,
    pub transport_send_count: u64,
    pub transport_error_count: u64,
    pub level_counts: HashMap<String, u64>,
    pub custom_counters: HashMap<String, u64>,
    pub custom_gauges: HashMap<String, u64>,
}

impl MetricsSummary {
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(LoggingError::SerializationError)
    }
    
    pub fn to_prometheus_format(&self) -> String {
        let mut output = String::new();
        
        // Core metrics
        output.push_str(&format!("logging_entries_total {}\n", self.entries_logged));
        output.push_str(&format!("logging_entries_dropped_total {}\n", self.entries_dropped));
        output.push_str(&format!("logging_bytes_total {}\n", self.bytes_logged));
        output.push_str(&format!("logging_flushes_total {}\n", self.flush_count));
        output.push_str(&format!("logging_errors_total {}\n", self.error_count));
        
        // Latency metrics
        output.push_str(&format!("logging_latency_avg_microseconds {}\n", self.avg_log_latency_us));
        output.push_str(&format!("logging_latency_p99_microseconds {}\n", self.p99_log_latency_us));
        output.push_str(&format!("logging_latency_max_microseconds {}\n", self.max_log_latency_us));
        
        // Buffer metrics
        output.push_str(&format!("logging_buffer_utilization_percent {}\n", self.buffer_utilization));
        output.push_str(&format!("logging_buffer_overflows_total {}\n", self.buffer_overflow_count));
        
        // Transport metrics
        output.push_str(&format!("logging_transport_sends_total {}\n", self.transport_send_count));
        output.push_str(&format!("logging_transport_errors_total {}\n", self.transport_error_count));
        
        // Level-specific metrics
        for (level, count) in &self.level_counts {
            output.push_str(&format!("logging_level_total{{level=\"{}\"}} {}\n", level, count));
        }
        
        // Custom metrics
        for (name, value) in &self.custom_counters {
            output.push_str(&format!("logging_custom_counter{{name=\"{}\"}} {}\n", name, value));
        }
        
        for (name, value) in &self.custom_gauges {
            output.push_str(&format!("logging_custom_gauge{{name=\"{}\"}} {}\n", name, value));
        }
        
        output
    }
}

pub struct MetricsReporter {
    metrics: Arc<LoggingMetrics>,
}

impl MetricsReporter {
    pub fn new(metrics: Arc<LoggingMetrics>) -> Self {
        Self { metrics }
    }
    
    pub async fn start_reporting(&self, interval_seconds: u64) {
        let metrics = Arc::clone(&self.metrics);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(interval_seconds)
            );
            
            loop {
                interval.tick().await;
                let summary = metrics.get_summary();
                
                // Log metrics summary
                println!("=== Logging Metrics ===");
                println!("Entries logged: {}", summary.entries_logged);
                println!("Entries dropped: {}", summary.entries_dropped);
                println!("Avg latency: {}μs", summary.avg_log_latency_us);
                println!("P99 latency: {}μs", summary.p99_log_latency_us);
                println!("Max latency: {}μs", summary.max_log_latency_us);
                println!("Buffer utilization: {}%", summary.buffer_utilization);
                println!("======================");
            }
        });
    }
}

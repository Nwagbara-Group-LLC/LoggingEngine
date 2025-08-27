//! Metrics Collector Library
//! 
//! High-performance metrics collection system optimized for trading applications
//! with support for counters, gauges, histograms, and custom aggregations.

use std::time::Duration;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;

/// Configuration for the metrics collector
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Buffer size for batching metrics
    pub buffer_size: usize,
    
    /// Flush interval for sending metrics
    pub flush_interval: Duration,
    
    /// Maximum retention time for metrics
    pub retention_time: Duration,
    
    /// Enable high-precision timestamps
    pub high_precision: bool,
    
    /// Maximum concurrent metric collections
    pub max_concurrent: usize,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            buffer_size: 10_000,
            flush_interval: Duration::from_millis(100),
            retention_time: Duration::from_secs(300), // 5 minutes
            high_precision: true,
            max_concurrent: 100,
        }
    }
}

/// Metric types supported by the collector
#[derive(Debug, Clone)]
pub enum MetricType {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
    Timer(Duration),
}

/// A single metric entry
#[derive(Debug, Clone)]
pub struct MetricEntry {
    pub name: String,
    pub value: MetricType,
    pub labels: Vec<(String, String)>,
    pub timestamp: std::time::SystemTime,
}

/// High-performance metrics collector
#[derive(Debug)]
pub struct MetricsCollector {
    config: MetricsConfig,
    running: Arc<RwLock<bool>>,
    metrics_buffer: Arc<RwLock<Vec<MetricEntry>>>,
}

impl MetricsCollector {
    /// Creates a new metrics collector with default configuration
    pub async fn new() -> Result<Self> {
        Self::with_config(MetricsConfig::default()).await
    }
    
    /// Creates a new metrics collector with custom configuration
    pub async fn with_config(config: MetricsConfig) -> Result<Self> {
        Ok(Self {
            config,
            running: Arc::new(RwLock::new(false)),
            metrics_buffer: Arc::new(RwLock::new(Vec::new())),
        })
    }
    
    /// Starts the metrics collection service
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(()); // Already running
        }
        
        *running = true;
        
        // Start background collection task
        let running_clone = self.running.clone();
        let buffer_clone = self.metrics_buffer.clone();
        let flush_interval = self.config.flush_interval;
        
        tokio::spawn(async move {
            while *running_clone.read().await {
                tokio::time::sleep(flush_interval).await;
                
                // Flush metrics buffer (simplified for now)
                let mut buffer = buffer_clone.write().await;
                if !buffer.is_empty() {
                    // In a real implementation, this would send metrics to storage/monitoring
                    buffer.clear();
                }
            }
        });
        
        Ok(())
    }
    
    /// Stops the metrics collection service
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        Ok(())
    }
    
    /// Records a counter metric
    pub async fn record_counter(&self, name: &str, value: u64, labels: Vec<(String, String)>) -> Result<()> {
        let entry = MetricEntry {
            name: name.to_string(),
            value: MetricType::Counter(value),
            labels,
            timestamp: std::time::SystemTime::now(),
        };
        
        let mut buffer = self.metrics_buffer.write().await;
        buffer.push(entry);
        Ok(())
    }
    
    /// Records a gauge metric
    pub async fn record_gauge(&self, name: &str, value: f64, labels: Vec<(String, String)>) -> Result<()> {
        let entry = MetricEntry {
            name: name.to_string(),
            value: MetricType::Gauge(value),
            labels,
            timestamp: std::time::SystemTime::now(),
        };
        
        let mut buffer = self.metrics_buffer.write().await;
        buffer.push(entry);
        Ok(())
    }
    
    /// Records a histogram metric
    pub async fn record_histogram(&self, name: &str, values: Vec<f64>, labels: Vec<(String, String)>) -> Result<()> {
        let entry = MetricEntry {
            name: name.to_string(),
            value: MetricType::Histogram(values),
            labels,
            timestamp: std::time::SystemTime::now(),
        };
        
        let mut buffer = self.metrics_buffer.write().await;
        buffer.push(entry);
        Ok(())
    }
    
    /// Records a timer metric
    pub async fn record_timer(&self, name: &str, duration: Duration, labels: Vec<(String, String)>) -> Result<()> {
        let entry = MetricEntry {
            name: name.to_string(),
            value: MetricType::Timer(duration),
            labels,
            timestamp: std::time::SystemTime::now(),
        };
        
        let mut buffer = self.metrics_buffer.write().await;
        buffer.push(entry);
        Ok(())
    }
    
    /// Gets current metrics count
    pub async fn get_metrics_count(&self) -> usize {
        self.metrics_buffer.read().await.len()
    }
    
    /// Checks if the collector is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_collector_creation() {
        let collector = MetricsCollector::new().await.unwrap();
        assert!(!collector.is_running().await);
    }
    
    #[tokio::test]
    async fn test_collector_lifecycle() {
        let collector = MetricsCollector::new().await.unwrap();
        
        // Start collector
        collector.start().await.unwrap();
        assert!(collector.is_running().await);
        
        // Stop collector
        collector.stop().await.unwrap();
        assert!(!collector.is_running().await);
    }
    
    #[tokio::test]
    async fn test_record_metrics() {
        let collector = MetricsCollector::new().await.unwrap();
        
        // Record different types of metrics
        collector.record_counter("requests", 100, vec![("service".to_string(), "api".to_string())]).await.unwrap();
        collector.record_gauge("cpu_usage", 75.5, vec![]).await.unwrap();
        collector.record_histogram("response_times", vec![1.0, 2.0, 3.0], vec![]).await.unwrap();
        collector.record_timer("process_time", Duration::from_millis(150), vec![]).await.unwrap();
        
        assert_eq!(collector.get_metrics_count().await, 4);
    }
}

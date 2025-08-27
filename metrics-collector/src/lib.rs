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
    pub aggregation_interval: Duration,
    pub max_metrics_in_memory: usize,
    pub prometheus_enabled: bool,
    pub prometheus_port: u16,
    pub export_file: Option<std::path::PathBuf>,
    pub export_interval: Duration,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            aggregation_interval: Duration::from_secs(60),
            max_metrics_in_memory: 10000,
            prometheus_enabled: false,
            prometheus_port: 9090,
            export_file: None,
            export_interval: Duration::from_secs(300),
        }
    }
}

/// Metric types supported by the collector
#[derive(Debug, Clone)]
pub enum MetricType {
    Counter,
    Gauge, 
    Histogram,
}

/// Transport options for metrics output
#[derive(Debug, Clone)]
pub enum Transport {
    Memory,
    Prometheus,
    File(std::path::PathBuf),
    Http { endpoint: String },
}

/// Individual metric entry
#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub labels: Vec<(String, String)>,
    pub metric_type: MetricType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Main metrics collector service
pub struct MetricsCollector {
    config: MetricsConfig,
    running: Arc<RwLock<bool>>,
    metrics: Arc<RwLock<Vec<Metric>>>,
}

impl MetricsCollector {
    pub fn new(config: MetricsConfig) -> Result<Self> {
        Ok(Self {
            config,
            running: Arc::new(RwLock::new(false)),
            metrics: Arc::new(RwLock::new(Vec::new())),
        })
    }

    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = true;
        
        // In a real implementation, this would start background tasks
        // For now, just mark as started
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        Ok(())
    }

    pub async fn record_counter(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        let metric = Metric {
            name: name.to_string(),
            value,
            labels: labels.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            metric_type: MetricType::Counter,
            timestamp: chrono::Utc::now(),
        };
        
        let mut metrics = self.metrics.write().await;
        metrics.push(metric);
        
        // Simulate minimal processing time
        tokio::time::sleep(Duration::from_nanos(10)).await;
    }

    pub async fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        let metric = Metric {
            name: name.to_string(),
            value,
            labels: labels.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            metric_type: MetricType::Gauge,
            timestamp: chrono::Utc::now(),
        };
        
        let mut metrics = self.metrics.write().await;
        metrics.push(metric);
        
        // Simulate minimal processing time
        tokio::time::sleep(Duration::from_nanos(10)).await;
    }

    pub async fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        let metric = Metric {
            name: name.to_string(),
            value,
            labels: labels.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            metric_type: MetricType::Histogram,
            timestamp: chrono::Utc::now(),
        };
        
        let mut metrics = self.metrics.write().await;
        metrics.push(metric);
        
        // Simulate minimal processing time
        tokio::time::sleep(Duration::from_nanos(10)).await;
    }

    pub async fn get_metrics(&self) -> Vec<Metric> {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    pub async fn clear_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_collector_creation() {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config).unwrap();
        assert!(!*collector.running.read().await);
    }

    #[tokio::test]
    async fn test_collector_lifecycle() {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config).unwrap();
        
        collector.start().await.unwrap();
        assert!(*collector.running.read().await);
        
        collector.stop().await.unwrap();
        assert!(!*collector.running.read().await);
    }

    #[tokio::test]
    async fn test_record_metrics() {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config).unwrap();
        
        collector.record_counter("test_counter", 1.0, &[("env", "test")]).await;
        collector.record_gauge("test_gauge", 42.0, &[("service", "test")]).await;
        collector.record_histogram("test_histogram", 0.001, &[("operation", "test")]).await;
        
        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.len(), 3);
    }
}

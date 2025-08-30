//! LoggingEngine HostBuilder
//! 
//! Primary orchestrator for the entire ultra-low latency logging infrastructure.
//! Coordinates log aggregation, metrics collection, and distributed logging services
//! optimized for high-frequency trading systems.

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, broadcast};
use tokio::signal;
use ultra_logger::{UltraLogger, LogLevel};
use log_aggregator::{LogAggregator, AggregatorConfig, Transport};
use metrics_collector::{MetricsCollector, MetricsConfig};

pub type LoggingResult<T> = anyhow::Result<T>;

/// Configuration for the entire logging engine
#[derive(Debug, Clone)]
pub struct LoggingEngineConfig {
    pub service_name: String,
    pub environment: Environment,
    pub log_level: LogLevel,
    pub aggregator_config: AggregatorConfig,
    pub metrics_config: MetricsConfig,
    pub enable_distributed_tracing: bool,
    pub enable_performance_monitoring: bool,
    pub shutdown_timeout: Duration,
}

impl Default for LoggingEngineConfig {
    fn default() -> Self {
        Self {
            service_name: "logging-engine".to_string(),
            environment: Environment::Development,
            log_level: LogLevel::Info,
            aggregator_config: AggregatorConfig::default(),
            metrics_config: MetricsConfig::default(),
            enable_distributed_tracing: true,
            enable_performance_monitoring: true,
            shutdown_timeout: Duration::from_secs(30),
        }
    }
}

/// Environment types for configuration optimization
#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Testing,
    Staging,
    Production,
}

/// Service health status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServiceStatus {
    Starting,
    Healthy,
    Degraded,
    Unhealthy,
    Stopping,
    Stopped,
}

/// Primary orchestrator for the logging engine
pub struct LoggingEngineHost {
    config: LoggingEngineConfig,
    logger: Arc<UltraLogger>,
    aggregator: Option<Arc<LogAggregator>>,
    metrics_collector: Option<Arc<MetricsCollector>>,
    status: Arc<RwLock<ServiceStatus>>,
    shutdown_tx: Option<broadcast::Sender<()>>,
}

impl LoggingEngineHost {
    /// Create a new logging engine host with default configuration
    pub fn new() -> Self {
        Self::with_config(LoggingEngineConfig::default())
    }
    
    /// Create a new logging engine host with custom configuration
    pub fn with_config(config: LoggingEngineConfig) -> Self {
        let logger = Arc::new(UltraLogger::new(config.service_name.clone()));
        let (shutdown_tx, _) = broadcast::channel(16);
        
        Self {
            config,
            logger,
            aggregator: None,
            metrics_collector: None,
            status: Arc::new(RwLock::new(ServiceStatus::Stopped)),
            shutdown_tx: Some(shutdown_tx),
        }
    }
    
    /// Start the entire logging engine infrastructure
    pub async fn start(&mut self) -> LoggingResult<()> {
        let mut status = self.status.write().await;
        *status = ServiceStatus::Starting;
        drop(status);
        
        let _ = self.logger.info("Starting Logging Engine Host...".to_string()).await;
        
        // Initialize and start log aggregator
        self.start_log_aggregator().await?;
        
        // Initialize and start metrics collector if enabled
        if self.config.enable_performance_monitoring {
            self.start_metrics_collector().await?;
        }
        
        // Mark as healthy
        let mut status = self.status.write().await;
        *status = ServiceStatus::Healthy;
        drop(status);
        
        let _ = self.logger.info("Logging Engine Host started successfully!".to_string()).await;
        self.log_startup_summary().await?;
        
        Ok(())
    }
    
    /// Start the log aggregation service
    async fn start_log_aggregator(&mut self) -> LoggingResult<()> {
        let _ = self.logger.info("Initializing Log Aggregator...".to_string()).await;
        
        // Create optimized config based on environment
        let mut config = self.config.aggregator_config.clone();
        self.optimize_aggregator_config(&mut config);
        
        let aggregator = LogAggregator::new(config)?;
        aggregator.start().await?;
        
        self.aggregator = Some(Arc::new(aggregator));
        let _ = self.logger.info("Log Aggregator started successfully".to_string()).await;
        
        Ok(())
    }
    
    /// Start the metrics collection service
    async fn start_metrics_collector(&mut self) -> LoggingResult<()> {
        let _ = self.logger.info("Initializing Metrics Collector...".to_string()).await;
        
        // Create optimized config based on environment
        let mut config = self.config.metrics_config.clone();
        self.optimize_metrics_config(&mut config);
        
        let collector = MetricsCollector::with_config(config).await?;
        collector.start().await?;
        
        self.metrics_collector = Some(Arc::new(collector));
        let _ = self.logger.info("Metrics Collector started successfully".to_string()).await;
        
        Ok(())
    }
    
    /// Optimize aggregator configuration based on environment
    fn optimize_aggregator_config(&self, config: &mut AggregatorConfig) {
        match self.config.environment {
            Environment::Production => {
                config.batch_size = 10_000;
                config.batch_timeout = Duration::from_millis(50);
                config.max_memory_usage = 500 * 1024 * 1024; // 500MB
                config.output_transport = Transport::Redis { 
                    url: "redis://redis-cluster:6379".to_string(),
                    channel: "trading-logs".to_string() 
                };
            },
            Environment::Staging => {
                config.batch_size = 5_000;
                config.batch_timeout = Duration::from_millis(100);
                config.max_memory_usage = 250 * 1024 * 1024; // 250MB
            },
            Environment::Development | Environment::Testing => {
                config.batch_size = 1_000;
                config.batch_timeout = Duration::from_millis(200);
                config.max_memory_usage = 100 * 1024 * 1024; // 100MB
            },
        }
    }
    
    /// Optimize metrics configuration based on environment
    fn optimize_metrics_config(&self, config: &mut MetricsConfig) {
        match self.config.environment {
            Environment::Production => {
                config.buffer_size = 50_000;
                config.flush_interval = Duration::from_millis(25); // Ultra-low latency
                config.high_precision = true;
                config.max_concurrent = 500;
            },
            Environment::Staging => {
                config.buffer_size = 25_000;
                config.flush_interval = Duration::from_millis(50);
                config.high_precision = true;
                config.max_concurrent = 250;
            },
            Environment::Development | Environment::Testing => {
                config.buffer_size = 10_000;
                config.flush_interval = Duration::from_millis(100);
                config.high_precision = false;
                config.max_concurrent = 100;
            },
        }
    }
    
    /// Log startup summary with configuration details
    async fn log_startup_summary(&self) -> LoggingResult<()> {
        let status = format!(
            "Logging Engine Configuration - Environment: {:?}, Services: [{}{}], Tracing: {}",
            self.config.environment,
            "LogAggregator",
            if self.config.enable_performance_monitoring { ", MetricsCollector" } else { "" },
            if self.config.enable_distributed_tracing { "Enabled" } else { "Disabled" }
        );
        
        let _ = self.logger.info(status).await;
        Ok(())
    }
    
    /// Get current service health status
    pub async fn get_status(&self) -> ServiceStatus {
        *self.status.read().await
    }
    
    /// Run the logging engine with graceful shutdown handling
    pub async fn run(&mut self) -> LoggingResult<()> {
        // Start all services
        self.start().await?;
        
        let _ = self.logger.info("Logging Engine is running. Press Ctrl+C to shutdown...".to_string()).await;
        println!("🚀 Logging Engine started successfully!");
        println!("📊 Environment: {:?}", self.config.environment);
        println!("🔧 Services running: Log Aggregator{}", 
                if self.config.enable_performance_monitoring { " + Metrics Collector" } else { "" });
        println!("Press Ctrl+C to shutdown...");
        
        // Wait for shutdown signal
        signal::ctrl_c().await?;
        
        // Graceful shutdown
        self.shutdown().await?;
        
        Ok(())
    }
    
    /// Gracefully shutdown all services
    pub async fn shutdown(&mut self) -> LoggingResult<()> {
        let mut status = self.status.write().await;
        *status = ServiceStatus::Stopping;
        drop(status);
        
        let _ = self.logger.info("Shutdown signal received, stopping Logging Engine...".to_string()).await;
        
        // Notify all services to shutdown
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }
        
        // Shutdown metrics collector first (less critical)
        if let Some(collector) = &self.metrics_collector {
            let _ = self.logger.info("Stopping Metrics Collector...".to_string()).await;
            collector.stop().await?;
            let _ = self.logger.info("Metrics Collector stopped".to_string()).await;
        }
        
        // Shutdown log aggregator (more critical, do last)
        if let Some(aggregator) = &self.aggregator {
            let _ = self.logger.info("Stopping Log Aggregator...".to_string()).await;
            aggregator.stop().await?;
            let _ = self.logger.info("Log Aggregator stopped".to_string()).await;
        }
        
        let mut status = self.status.write().await;
        *status = ServiceStatus::Stopped;
        drop(status);
        
        let _ = self.logger.info("Logging Engine shutdown completed successfully!".to_string()).await;
        println!("✅ Logging Engine shutdown completed successfully!");
        
        Ok(())
    }
    
    /// Health check endpoint for monitoring
    pub async fn health_check(&self) -> LoggingResult<HealthStatus> {
        let status = self.get_status().await;
        let aggregator_healthy = self.aggregator.is_some();
        let metrics_healthy = self.metrics_collector.is_some() || !self.config.enable_performance_monitoring;
        
        let overall_healthy = matches!(status, ServiceStatus::Healthy) && aggregator_healthy && metrics_healthy;
        
        Ok(HealthStatus {
            status,
            aggregator_healthy,
            metrics_collector_healthy: metrics_healthy,
            overall_healthy,
        })
    }
}

/// Health status response
#[derive(Debug)]
pub struct HealthStatus {
    pub status: ServiceStatus,
    pub aggregator_healthy: bool,
    pub metrics_collector_healthy: bool,
    pub overall_healthy: bool,
}

/// Builder pattern for easy configuration
pub struct LoggingEngineBuilder {
    config: LoggingEngineConfig,
}

impl LoggingEngineBuilder {
    pub fn new() -> Self {
        Self {
            config: LoggingEngineConfig::default(),
        }
    }
    
    pub fn service_name<S: Into<String>>(mut self, name: S) -> Self {
        self.config.service_name = name.into();
        self
    }
    
    pub fn environment(mut self, env: Environment) -> Self {
        self.config.environment = env;
        self
    }
    
    pub fn log_level(mut self, level: LogLevel) -> Self {
        self.config.log_level = level;
        self
    }
    
    pub fn enable_performance_monitoring(mut self, enable: bool) -> Self {
        self.config.enable_performance_monitoring = enable;
        self
    }
    
    pub fn enable_distributed_tracing(mut self, enable: bool) -> Self {
        self.config.enable_distributed_tracing = enable;
        self
    }
    
    pub fn shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.config.shutdown_timeout = timeout;
        self
    }
    
    pub fn build(self) -> LoggingEngineHost {
        LoggingEngineHost::with_config(self.config)
    }
}

impl Default for LoggingEngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_pattern() {
        let host = LoggingEngineBuilder::new()
            .service_name("test-logging-engine")
            .environment(Environment::Testing)
            .log_level(LogLevel::Debug)
            .enable_performance_monitoring(false)
            .build();
            
        assert_eq!(host.config.service_name, "test-logging-engine");
        assert_eq!(host.config.environment, Environment::Testing);
    }
    
    #[tokio::test]
    async fn test_host_creation() {
        let host = LoggingEngineHost::new();
        let status = host.get_status().await;
        assert_eq!(status, ServiceStatus::Stopped);
    }
}

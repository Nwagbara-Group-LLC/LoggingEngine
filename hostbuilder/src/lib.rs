//! LoggingEngine HostBuilder
//! 
//! Primary orchestrator for the entire ultra-low latency logging infrastructure.
//! Coordinates log aggregation, metrics collection, and distributed logging services
//! optimized for high-frequency trading systems.
//! 
//! ALL CONFIGURATION VALUES LOADED FROM ENVIRONMENT VARIABLES - NO HARD-CODING

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, broadcast};
use tokio::signal;
use ultra_logger::UltraLogger;
use log_aggregator::LogAggregator;
use metrics_collector::MetricsCollector;

// Use centralized configuration
use config::{LoggingEngineConfig, Environment, LogLevel, ConfigLoader};

pub type LoggingResult<T> = anyhow::Result<T>;

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
    /// Create a new logging engine host with configuration from environment
    pub fn new() -> LoggingResult<Self> {
        let config_instance = LoggingEngineConfig::from_env()?;
        let logger = Arc::new(UltraLogger::new(config_instance.service_name.clone()));
        let (shutdown_tx, _) = broadcast::channel(16);
        
        Ok(Self {
            logger,
            aggregator: None,
            metrics_collector: None,
            status: Arc::new(RwLock::new(ServiceStatus::Stopped)),
            shutdown_tx: Some(shutdown_tx),
            config: config_instance,
        })
    }
    
    /// Create a new logging engine host with custom configuration
    pub fn with_config(config: LoggingEngineConfig) -> LoggingResult<Self> {
        let logger = Arc::new(UltraLogger::new(config.service_name.clone()));
        let (shutdown_tx, _) = broadcast::channel(16);
        
        Ok(Self {
            logger,
            aggregator: None,
            metrics_collector: None,
            status: Arc::new(RwLock::new(ServiceStatus::Stopped)),
            shutdown_tx: Some(shutdown_tx),
            config,
        })
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
        
        // Use configuration loaded from environment variables
        let aggregator_config = self.config.aggregator.to_log_aggregator_config();
        
        let aggregator = LogAggregator::new(aggregator_config)?;
        aggregator.start().await?;
        
        self.aggregator = Some(Arc::new(aggregator));
        let _ = self.logger.info("Log Aggregator started successfully".to_string()).await;
        
        Ok(())
    }
    
    /// Start the metrics collection service
    async fn start_metrics_collector(&mut self) -> LoggingResult<()> {
        let _ = self.logger.info("Initializing Metrics Collector...".to_string()).await;
        
        // Use configuration loaded from environment variables
        let metrics_config = self.config.metrics.to_metrics_collector_config();
        
        let collector = MetricsCollector::with_config(metrics_config).await?;
        collector.start().await?;
        
        self.metrics_collector = Some(Arc::new(collector));
        let _ = self.logger.info("Metrics Collector started successfully".to_string()).await;
        
        Ok(())
    }
    
    /// Log startup summary with configuration details
    async fn log_startup_summary(&self) -> LoggingResult<()> {
        let status = format!(
            "Logging Engine Configuration - Environment: {}, Services: [{}{}], Tracing: {}",
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
    
    /// Get the service name
    pub fn get_service_name(&self) -> &str {
        &self.config.service_name
    }
    
    /// Run the logging engine with graceful shutdown handling
    pub async fn run(&mut self) -> LoggingResult<()> {
        // Start all services
        self.start().await?;
        
        let _ = self.logger.info("Logging Engine is running. Press Ctrl+C to shutdown...".to_string()).await;
        println!("🚀 Logging Engine started successfully!");
        println!("📊 Environment: {}", self.config.environment);
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

/// Builder pattern for easy configuration - loads from environment variables
pub struct LoggingEngineBuilder {
    // Optional overrides to environment variables
    service_name_override: Option<String>,
    environment_override: Option<String>,
    log_level_override: Option<String>,
    enable_performance_monitoring_override: Option<bool>,
    enable_distributed_tracing_override: Option<bool>,
    shutdown_timeout_secs_override: Option<u64>,
}

impl LoggingEngineBuilder {
    pub fn new() -> Self {
        Self {
            service_name_override: None,
            environment_override: None,
            log_level_override: None,
            enable_performance_monitoring_override: None,
            enable_distributed_tracing_override: None,
            shutdown_timeout_secs_override: None,
        }
    }
    
    pub fn service_name<S: Into<String>>(mut self, name: S) -> Self {
        self.service_name_override = Some(name.into());
        self
    }
    
    pub fn environment(mut self, env: Environment) -> Self {
        let env_str = match env {
            Environment::Production => "production",
            Environment::Staging => "staging", 
            Environment::Testing => "testing",
            Environment::Development => "development",
        };
        self.environment_override = Some(env_str.to_string());
        self
    }
    
    pub fn log_level(mut self, level: LogLevel) -> Self {
        let level_str = match level {
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn", 
            LogLevel::Error => "error",
        };
        self.log_level_override = Some(level_str.to_string());
        self
    }
    
    pub fn enable_performance_monitoring(mut self, enable: bool) -> Self {
        self.enable_performance_monitoring_override = Some(enable);
        self
    }
    
    pub fn enable_distributed_tracing(mut self, enable: bool) -> Self {
        self.enable_distributed_tracing_override = Some(enable);
        self
    }
    
    pub fn shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.shutdown_timeout_secs_override = Some(timeout.as_secs());
        self
    }
    
    pub fn build(self) -> LoggingResult<LoggingEngineHost> {
        // Load base configuration from environment
        let mut config = LoggingEngineConfig::from_env()?;
        
        // Apply overrides
        if let Some(name) = self.service_name_override {
            config.service_name = name;
        }
        if let Some(env) = self.environment_override {
            config.environment = env;
        }
        if let Some(level) = self.log_level_override {
            config.log_level = level;
        }
        if let Some(enable) = self.enable_performance_monitoring_override {
            config.enable_performance_monitoring = enable;
        }
        if let Some(enable) = self.enable_distributed_tracing_override {
            config.enable_distributed_tracing = enable;
        }
        if let Some(timeout) = self.shutdown_timeout_secs_override {
            config.shutdown_timeout_secs = timeout;
        }
        
        Ok(LoggingEngineHost::with_config(config)?)
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
            .build()
            .expect("Failed to build logging engine host");
            
        // Test that the service was configured with environment variables
        // Since config is private, we'll test through public methods
        assert_eq!(host.get_service_name(), "test-logging-engine");
    }
    
    #[tokio::test]
    async fn test_host_creation() {
        let host = LoggingEngineHost::new()
            .expect("Failed to create logging engine host");
        let status = host.get_status().await;
        assert_eq!(status, ServiceStatus::Stopped);
    }
}

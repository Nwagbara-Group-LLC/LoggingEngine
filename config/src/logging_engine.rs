//! LoggingEngine Core Configuration
//!
//! Primary configuration for the logging engine host and orchestration.

use super::*;

/// LoggingEngine configuration loaded from environment variables and ConfigMaps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingEngineConfig {
    // Service identification
    pub service_name: String,
    pub environment: String,
    pub log_level: String,
    
    // Performance settings
    pub enable_performance_monitoring: bool,
    pub enable_distributed_tracing: bool,
    pub shutdown_timeout_secs: u64,
    
    // Component configurations
    pub aggregator: AggregatorConfig,
    pub metrics: MetricsConfig,
    pub ultra_logger: UltraLoggerConfig,
}

impl ConfigLoader for LoggingEngineConfig {
    fn from_env() -> Result<Self> {
        let environment = env_string_or_default("LOGGING_ENVIRONMENT", "development");
        let env_type = Environment::from_str(&environment);
        
        Ok(Self {
            service_name: env_string_or_default("LOGGING_SERVICE_NAME", "logging-engine"),
            environment,
            log_level: env_string_or_default("LOG_LEVEL", "info"),
            enable_performance_monitoring: env_bool_or_default("ENABLE_PERFORMANCE_MONITORING", true),
            enable_distributed_tracing: env_bool_or_default("ENABLE_DISTRIBUTED_TRACING", true),
            shutdown_timeout_secs: env_var_or_default("SHUTDOWN_TIMEOUT_SECS", 30),
            aggregator: AggregatorConfig::get_defaults(&env_type),
            metrics: MetricsConfig::get_defaults(&env_type),
            ultra_logger: UltraLoggerConfig::get_defaults(&env_type),
        })
    }
    
    fn validate(&self) -> Result<()> {
        if self.service_name.is_empty() {
            return Err(anyhow!("Service name cannot be empty"));
        }
        
        if self.shutdown_timeout_secs == 0 {
            return Err(anyhow!("Shutdown timeout must be greater than 0"));
        }
        
        // Validate component configurations
        self.aggregator.validate()?;
        self.metrics.validate()?;
        self.ultra_logger.validate()?;
        
        Ok(())
    }
    
    fn get_defaults(env: &Environment) -> Self {
        Self {
            service_name: "logging-engine".to_string(),
            environment: format!("{:?}", env).to_lowercase(),
            log_level: match env {
                Environment::Production => "info",
                Environment::Staging => "info", 
                Environment::Testing => "debug",
                Environment::Development => "debug",
            }.to_string(),
            enable_performance_monitoring: match env {
                Environment::Production | Environment::Staging => true,
                Environment::Testing | Environment::Development => false,
            },
            enable_distributed_tracing: match env {
                Environment::Production | Environment::Staging => true,
                Environment::Testing | Environment::Development => false,
            },
            shutdown_timeout_secs: match env {
                Environment::Production => 60,
                Environment::Staging => 45,
                Environment::Testing => 15,
                Environment::Development => 30,
            },
            aggregator: AggregatorConfig::get_defaults(env),
            metrics: MetricsConfig::get_defaults(env),
            ultra_logger: UltraLoggerConfig::get_defaults(env),
        }
    }
}

impl LoggingEngineConfig {
    /// Convert to legacy AggregatorConfig for compatibility
    pub fn to_aggregator_config(&self) -> crate::aggregator::AggregatorConfig {
        self.aggregator.clone()
    }
    
    /// Convert to legacy MetricsConfig for compatibility
    pub fn to_metrics_config(&self) -> crate::metrics::MetricsConfig {
        self.metrics.clone()
    }
    
    /// Convert to legacy UltraLoggerConfig for compatibility
    pub fn to_ultra_logger_config(&self) -> crate::ultra_logger::UltraLoggerConfig {
        self.ultra_logger.clone()
    }
    
    /// Get the parsed Environment enum
    pub fn get_environment(&self) -> Environment {
        Environment::from_str(&self.environment)
    }
    
    /// Get the parsed LogLevel enum
    pub fn get_log_level(&self) -> LogLevel {
        LogLevel::from_str(&self.log_level)
    }
    
    /// Get shutdown timeout as Duration
    pub fn get_shutdown_timeout(&self) -> Duration {
        Duration::from_secs(self.shutdown_timeout_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_logging_engine_config_defaults() {
        let config = LoggingEngineConfig::get_defaults(&Environment::Development);
        assert_eq!(config.service_name, "logging-engine");
        assert_eq!(config.environment, "development");
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.shutdown_timeout_secs, 30);
    }
    
    #[test]
    fn test_logging_engine_config_production() {
        let config = LoggingEngineConfig::get_defaults(&Environment::Production);
        assert_eq!(config.log_level, "info");
        assert_eq!(config.enable_performance_monitoring, true);
        assert_eq!(config.enable_distributed_tracing, true);
        assert_eq!(config.shutdown_timeout_secs, 60);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = LoggingEngineConfig::get_defaults(&Environment::Development);
        assert!(config.validate().is_ok());
        
        config.service_name = String::new();
        assert!(config.validate().is_err());
        
        config.service_name = "test".to_string();
        config.shutdown_timeout_secs = 0;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_helper_methods() {
        let config = LoggingEngineConfig::get_defaults(&Environment::Production);
        assert_eq!(config.get_environment(), Environment::Production);
        assert_eq!(config.get_log_level(), LogLevel::Info);
        assert_eq!(config.get_shutdown_timeout(), Duration::from_secs(60));
    }
}

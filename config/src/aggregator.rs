//! Log Aggregator Configuration
//!
//! Configuration for the log aggregation component that handles batching,
//! buffering, and transport of log messages.

use super::*;

/// Configuration for log aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatorConfig {
    pub batch_size: usize,
    pub batch_timeout_millis: u64,
    pub max_memory_usage_bytes: usize,
    pub redis_url: String,
    pub redis_channel: String,
    pub transport_type: String,
    pub buffer_capacity: usize,
    pub max_connections: usize,
    pub connection_timeout_millis: u64,
    pub retry_attempts: usize,
    pub retry_delay_millis: u64,
}

impl ConfigLoader for AggregatorConfig {
    fn from_env() -> Result<Self> {
        let environment = Environment::from_str(&env_string_or_default("LOGGING_ENVIRONMENT", "development"));
        let defaults = Self::get_defaults(&environment);
        
        Ok(Self {
            batch_size: env_var_or_default("LOG_BATCH_SIZE", defaults.batch_size),
            batch_timeout_millis: env_var_or_default("LOG_BATCH_TIMEOUT_MS", defaults.batch_timeout_millis),
            max_memory_usage_bytes: env_var_or_default("LOG_MAX_MEMORY_BYTES", defaults.max_memory_usage_bytes),
            redis_url: env_string_or_default("REDIS_URL", &defaults.redis_url),
            redis_channel: env_string_or_default("REDIS_CHANNEL", &defaults.redis_channel),
            transport_type: env_string_or_default("LOG_TRANSPORT", &defaults.transport_type),
            buffer_capacity: env_var_or_default("LOG_BUFFER_CAPACITY", defaults.buffer_capacity),
            max_connections: env_var_or_default("LOG_MAX_CONNECTIONS", defaults.max_connections),
            connection_timeout_millis: env_var_or_default("LOG_CONNECTION_TIMEOUT_MS", defaults.connection_timeout_millis),
            retry_attempts: env_var_or_default("LOG_RETRY_ATTEMPTS", defaults.retry_attempts),
            retry_delay_millis: env_var_or_default("LOG_RETRY_DELAY_MS", defaults.retry_delay_millis),
        })
    }
    
    fn validate(&self) -> Result<()> {
        if self.batch_size == 0 {
            return Err(anyhow!("Batch size must be greater than 0"));
        }
        
        if self.batch_timeout_millis == 0 {
            return Err(anyhow!("Batch timeout must be greater than 0"));
        }
        
        if self.max_memory_usage_bytes == 0 {
            return Err(anyhow!("Max memory usage must be greater than 0"));
        }
        
        if self.redis_url.is_empty() {
            return Err(anyhow!("Redis URL cannot be empty"));
        }
        
        if self.buffer_capacity == 0 {
            return Err(anyhow!("Buffer capacity must be greater than 0"));
        }
        
        Ok(())
    }
    
    fn get_defaults(env: &Environment) -> Self {
        match env {
            Environment::Production => Self {
                batch_size: 10000,
                batch_timeout_millis: 50,
                max_memory_usage_bytes: 500 * 1024 * 1024, // 500MB
                redis_url: "redis://redis-cluster.prod.svc.cluster.local:6379".to_string(),
                redis_channel: "logs:production".to_string(),
                transport_type: "redis".to_string(),
                buffer_capacity: 65536,
                max_connections: 100,
                connection_timeout_millis: 5000,
                retry_attempts: 5,
                retry_delay_millis: 100,
            },
            Environment::Staging => Self {
                batch_size: 5000,
                batch_timeout_millis: 100,
                max_memory_usage_bytes: 250 * 1024 * 1024, // 250MB
                redis_url: "redis://redis.staging.svc.cluster.local:6379".to_string(),
                redis_channel: "logs:staging".to_string(),
                transport_type: "redis".to_string(),
                buffer_capacity: 32768,
                max_connections: 50,
                connection_timeout_millis: 3000,
                retry_attempts: 3,
                retry_delay_millis: 200,
            },
            Environment::Testing => Self {
                batch_size: 1000,
                batch_timeout_millis: 200,
                max_memory_usage_bytes: 100 * 1024 * 1024, // 100MB
                redis_url: "redis://localhost:6379".to_string(),
                redis_channel: "logs:testing".to_string(),
                transport_type: "redis".to_string(),
                buffer_capacity: 16384,
                max_connections: 25,
                connection_timeout_millis: 2000,
                retry_attempts: 2,
                retry_delay_millis: 500,
            },
            Environment::Development => Self {
                batch_size: 1000,
                batch_timeout_millis: 200,
                max_memory_usage_bytes: 100 * 1024 * 1024, // 100MB
                redis_url: "redis://localhost:6379".to_string(),
                redis_channel: "logs:development".to_string(),
                transport_type: "console".to_string(),
                buffer_capacity: 16384,
                max_connections: 25,
                connection_timeout_millis: 2000,
                retry_attempts: 2,
                retry_delay_millis: 500,
            },
        }
    }
}

impl AggregatorConfig {
    /// Convert to log-aggregator's AggregatorConfig
    pub fn to_log_aggregator_config(&self) -> log_aggregator::AggregatorConfig {
        log_aggregator::AggregatorConfig {
            batch_size: self.batch_size,
            batch_timeout: std::time::Duration::from_millis(self.batch_timeout_millis),
            max_memory_usage: self.max_memory_usage_bytes,
            output_transport: log_aggregator::Transport::Redis {
                url: self.redis_url.clone(),
                channel: self.redis_channel.clone(),
            },
            filters: vec![], // Default empty filters
        }
    }

    /// Get batch timeout as Duration
    pub fn get_batch_timeout(&self) -> Duration {
        Duration::from_millis(self.batch_timeout_millis)
    }
    
    /// Get connection timeout as Duration
    pub fn get_connection_timeout(&self) -> Duration {
        Duration::from_millis(self.connection_timeout_millis)
    }
    
    /// Get retry delay as Duration
    pub fn get_retry_delay(&self) -> Duration {
        Duration::from_millis(self.retry_delay_millis)
    }
    
    /// Get transport as enum
    pub fn get_transport(&self) -> Transport {
        match self.transport_type.to_lowercase().as_str() {
            "redis" => Transport::Redis,
            "file" => Transport::File,
            "console" => Transport::Console,
            "network" => Transport::Network,
            _ => Transport::Console,
        }
    }
    
    /// Get memory limit in MB for display
    pub fn get_memory_limit_mb(&self) -> usize {
        self.max_memory_usage_bytes / (1024 * 1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_aggregator_config_defaults() {
        let prod_config = AggregatorConfig::get_defaults(&Environment::Production);
        assert_eq!(prod_config.batch_size, 10000);
        assert_eq!(prod_config.batch_timeout_millis, 50);
        assert_eq!(prod_config.get_memory_limit_mb(), 500);
        
        let dev_config = AggregatorConfig::get_defaults(&Environment::Development);
        assert_eq!(dev_config.batch_size, 1000);
        assert_eq!(dev_config.transport_type, "console");
        assert_eq!(dev_config.get_memory_limit_mb(), 100);
    }
    
    #[test]
    fn test_aggregator_config_validation() {
        let mut config = AggregatorConfig::get_defaults(&Environment::Development);
        assert!(config.validate().is_ok());
        
        config.batch_size = 0;
        assert!(config.validate().is_err());
        
        config.batch_size = 1000;
        config.redis_url = String::new();
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_aggregator_helper_methods() {
        let config = AggregatorConfig::get_defaults(&Environment::Production);
        assert_eq!(config.get_batch_timeout(), Duration::from_millis(50));
        assert_eq!(config.get_transport(), Transport::Redis);
        assert_eq!(config.get_memory_limit_mb(), 500);
    }
}

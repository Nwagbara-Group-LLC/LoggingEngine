//! Metrics Collection Configuration
//!
//! Configuration for metrics collection, aggregation, and reporting.

use super::*;

/// Configuration for metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub buffer_size: usize,
    pub flush_interval_millis: u64,
    pub high_precision: bool,
    pub max_concurrent: usize,
    pub batch_size: usize,
    pub retention_duration_secs: u64,
    pub compression_enabled: bool,
    pub export_interval_secs: u64,
    pub prometheus_enabled: bool,
    pub prometheus_port: u16,
    pub histogram_buckets: Vec<f64>,
}

impl ConfigLoader for MetricsConfig {
    fn from_env() -> Result<Self> {
        let environment = Environment::from_str(&env_string_or_default("LOGGING_ENVIRONMENT", "development"));
        let defaults = Self::get_defaults(&environment);
        
        Ok(Self {
            buffer_size: env_var_or_default("METRICS_BUFFER_SIZE", defaults.buffer_size),
            flush_interval_millis: env_var_or_default("METRICS_FLUSH_INTERVAL_MS", defaults.flush_interval_millis),
            high_precision: env_bool_or_default("METRICS_HIGH_PRECISION", defaults.high_precision),
            max_concurrent: env_var_or_default("METRICS_MAX_CONCURRENT", defaults.max_concurrent),
            batch_size: env_var_or_default("METRICS_BATCH_SIZE", defaults.batch_size),
            retention_duration_secs: env_var_or_default("METRICS_RETENTION_SECS", defaults.retention_duration_secs),
            compression_enabled: env_bool_or_default("METRICS_COMPRESSION", defaults.compression_enabled),
            export_interval_secs: env_var_or_default("METRICS_EXPORT_INTERVAL_SECS", defaults.export_interval_secs),
            prometheus_enabled: env_bool_or_default("PROMETHEUS_ENABLED", defaults.prometheus_enabled),
            prometheus_port: env_var_or_default("PROMETHEUS_PORT", defaults.prometheus_port),
            histogram_buckets: parse_histogram_buckets(),
        })
    }
    
    fn validate(&self) -> Result<()> {
        if self.buffer_size == 0 {
            return Err(anyhow!("Buffer size must be greater than 0"));
        }
        
        if self.flush_interval_millis == 0 {
            return Err(anyhow!("Flush interval must be greater than 0"));
        }
        
        if self.max_concurrent == 0 {
            return Err(anyhow!("Max concurrent must be greater than 0"));
        }
        
        if self.batch_size == 0 {
            return Err(anyhow!("Batch size must be greater than 0"));
        }
        
        if self.prometheus_port == 0 {
            return Err(anyhow!("Prometheus port must be greater than 0"));
        }
        
        Ok(())
    }
    
    fn get_defaults(env: &Environment) -> Self {
        match env {
            Environment::Production => Self {
                buffer_size: 16384,
                flush_interval_millis: 100,
                high_precision: true,
                max_concurrent: 1000,
                batch_size: 5000,
                retention_duration_secs: 86400, // 24 hours
                compression_enabled: true,
                export_interval_secs: 30,
                prometheus_enabled: true,
                prometheus_port: 9090,
                histogram_buckets: vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
            },
            Environment::Staging => Self {
                buffer_size: 8192,
                flush_interval_millis: 200,
                high_precision: true,
                max_concurrent: 500,
                batch_size: 2500,
                retention_duration_secs: 43200, // 12 hours
                compression_enabled: true,
                export_interval_secs: 60,
                prometheus_enabled: true,
                prometheus_port: 9090,
                histogram_buckets: vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
            },
            Environment::Testing => Self {
                buffer_size: 4096,
                flush_interval_millis: 500,
                high_precision: false,
                max_concurrent: 100,
                batch_size: 1000,
                retention_duration_secs: 3600, // 1 hour
                compression_enabled: false,
                export_interval_secs: 120,
                prometheus_enabled: false,
                prometheus_port: 9091,
                histogram_buckets: vec![0.001, 0.01, 0.1, 1.0, 10.0],
            },
            Environment::Development => Self {
                buffer_size: 4096,
                flush_interval_millis: 500,
                high_precision: false,
                max_concurrent: 100,
                batch_size: 1000,
                retention_duration_secs: 3600, // 1 hour
                compression_enabled: false,
                export_interval_secs: 120,
                prometheus_enabled: false,
                prometheus_port: 9091,
                histogram_buckets: vec![0.001, 0.01, 0.1, 1.0, 10.0],
            },
        }
    }
}

impl MetricsConfig {
    /// Convert to metrics-collector's MetricsConfig
    pub fn to_metrics_collector_config(&self) -> metrics_collector::MetricsConfig {
        metrics_collector::MetricsConfig {
            buffer_size: self.buffer_size,
            flush_interval: Duration::from_millis(self.flush_interval_millis),
            retention_time: Duration::from_secs(self.retention_duration_secs),
            high_precision: self.high_precision,
            max_concurrent: self.max_concurrent,
        }
    }

    /// Get flush interval as Duration
    pub fn get_flush_interval(&self) -> Duration {
        Duration::from_millis(self.flush_interval_millis)
    }
    
    /// Get retention duration as Duration
    pub fn get_retention_duration(&self) -> Duration {
        Duration::from_secs(self.retention_duration_secs)
    }
    
    /// Get export interval as Duration
    pub fn get_export_interval(&self) -> Duration {
        Duration::from_secs(self.export_interval_secs)
    }
    
    /// Get buffer size in KB for display
    pub fn get_buffer_size_kb(&self) -> usize {
        self.buffer_size / 1024
    }
}

/// Parse histogram buckets from environment variable
fn parse_histogram_buckets() -> Vec<f64> {
    env::var("METRICS_HISTOGRAM_BUCKETS")
        .ok()
        .and_then(|s| {
            s.split(',')
                .map(|bucket| bucket.trim().parse::<f64>())
                .collect::<Result<Vec<_>, _>>()
                .ok()
        })
        .unwrap_or_else(|| vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0])
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metrics_config_defaults() {
        let prod_config = MetricsConfig::get_defaults(&Environment::Production);
        assert_eq!(prod_config.buffer_size, 16384);
        assert_eq!(prod_config.flush_interval_millis, 100);
        assert_eq!(prod_config.high_precision, true);
        assert_eq!(prod_config.prometheus_enabled, true);
        
        let dev_config = MetricsConfig::get_defaults(&Environment::Development);
        assert_eq!(dev_config.buffer_size, 4096);
        assert_eq!(dev_config.high_precision, false);
        assert_eq!(dev_config.prometheus_enabled, false);
    }
    
    #[test]
    fn test_metrics_config_validation() {
        let mut config = MetricsConfig::get_defaults(&Environment::Development);
        assert!(config.validate().is_ok());
        
        config.buffer_size = 0;
        assert!(config.validate().is_err());
        
        config.buffer_size = 4096;
        config.flush_interval_millis = 0;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_metrics_helper_methods() {
        let config = MetricsConfig::get_defaults(&Environment::Production);
        assert_eq!(config.get_flush_interval(), Duration::from_millis(100));
        assert_eq!(config.get_retention_duration(), Duration::from_secs(86400));
        assert_eq!(config.get_buffer_size_kb(), 16);
    }
    
    #[test]
    fn test_histogram_buckets_parsing() {
        env::set_var("METRICS_HISTOGRAM_BUCKETS", "0.1,0.5,1.0,5.0");
        let buckets = parse_histogram_buckets();
        assert_eq!(buckets, vec![0.1, 0.5, 1.0, 5.0]);
        
        env::remove_var("METRICS_HISTOGRAM_BUCKETS");
        let default_buckets = parse_histogram_buckets();
        assert!(!default_buckets.is_empty());
    }
}

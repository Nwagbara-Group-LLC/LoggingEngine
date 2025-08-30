//! Benchmark Configuration
//!
//! Configuration for performance benchmarks and testing parameters.

use super::*;

/// Configuration for benchmark tests and demonstrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    // Test 1: Ultra-High Throughput Test
    pub throughput_test_message_count: u64,
    pub throughput_test_chunk_count: usize,
    pub throughput_test_sleep_between_batches_ms: u64,
    
    // Test 2: Batch Processing Efficiency
    pub batch_test_message_count: u64,
    pub batch_test_expected_batch_size: u64,
    pub batch_test_sleep_before_check_ms: u64,
    
    // Test 3: Memory Pool and Lock-Free Operations
    pub memory_test_iterations: u64,
    pub memory_test_sleep_ms: u64,
    
    // Performance targets and thresholds
    pub target_latency_us: f64,
    pub target_p99_latency_us: f64,
    pub target_throughput_per_sec: u64,
    pub target_memory_mb: u64,
    pub target_reliability_percent: f64,
    
    // CLI defaults
    pub default_shutdown_timeout_secs: u64,
    pub default_run_duration_secs: u64,
    
    // Demonstration messages
    pub demo_btc_price: f64,
    pub demo_btc_volume: f64,
}

impl ConfigLoader for BenchmarkConfig {
    fn from_env() -> Result<Self> {
        let environment = Environment::from_str(&env_string_or_default("LOGGING_ENVIRONMENT", "development"));
        let defaults = Self::get_defaults(&environment);
        
        Ok(Self {
            // Test 1 configuration
            throughput_test_message_count: env_var_or_default("BENCH_THROUGHPUT_MESSAGE_COUNT", defaults.throughput_test_message_count),
            throughput_test_chunk_count: env_var_or_default("BENCH_THROUGHPUT_CHUNK_COUNT", defaults.throughput_test_chunk_count),
            throughput_test_sleep_between_batches_ms: env_var_or_default("BENCH_THROUGHPUT_SLEEP_MS", defaults.throughput_test_sleep_between_batches_ms),
                
            // Test 2 configuration
            batch_test_message_count: env_var_or_default("BENCH_BATCH_MESSAGE_COUNT", defaults.batch_test_message_count),
            batch_test_expected_batch_size: env_var_or_default("BENCH_BATCH_EXPECTED_SIZE", defaults.batch_test_expected_batch_size),
            batch_test_sleep_before_check_ms: env_var_or_default("BENCH_BATCH_SLEEP_MS", defaults.batch_test_sleep_before_check_ms),
                
            // Test 3 configuration
            memory_test_iterations: env_var_or_default("BENCH_MEMORY_ITERATIONS", defaults.memory_test_iterations),
            memory_test_sleep_ms: env_var_or_default("BENCH_MEMORY_SLEEP_MS", defaults.memory_test_sleep_ms),
                
            // Performance targets
            target_latency_us: env_var_or_default("TARGET_LATENCY_US", defaults.target_latency_us),
            target_p99_latency_us: env_var_or_default("TARGET_P99_LATENCY_US", defaults.target_p99_latency_us),
            target_throughput_per_sec: env_var_or_default("TARGET_THROUGHPUT_PER_SEC", defaults.target_throughput_per_sec),
            target_memory_mb: env_var_or_default("TARGET_MEMORY_MB", defaults.target_memory_mb),
            target_reliability_percent: env_var_or_default("TARGET_RELIABILITY_PERCENT", defaults.target_reliability_percent),
                
            // CLI defaults
            default_shutdown_timeout_secs: env_var_or_default("DEFAULT_SHUTDOWN_TIMEOUT_SECS", defaults.default_shutdown_timeout_secs),
            default_run_duration_secs: env_var_or_default("DEFAULT_RUN_DURATION_SECS", defaults.default_run_duration_secs),
                
            // Demo configuration
            demo_btc_price: env_var_or_default("DEMO_BTC_PRICE", defaults.demo_btc_price),
            demo_btc_volume: env_var_or_default("DEMO_BTC_VOLUME", defaults.demo_btc_volume),
        })
    }
    
    fn validate(&self) -> Result<()> {
        if self.throughput_test_message_count == 0 {
            return Err(anyhow!("Throughput test message count must be greater than 0"));
        }
        
        if self.throughput_test_chunk_count == 0 {
            return Err(anyhow!("Throughput test chunk count must be greater than 0"));
        }
        
        if self.batch_test_message_count == 0 {
            return Err(anyhow!("Batch test message count must be greater than 0"));
        }
        
        if self.target_latency_us <= 0.0 {
            return Err(anyhow!("Target latency must be greater than 0"));
        }
        
        if self.target_throughput_per_sec == 0 {
            return Err(anyhow!("Target throughput must be greater than 0"));
        }
        
        if self.target_reliability_percent < 0.0 || self.target_reliability_percent > 100.0 {
            return Err(anyhow!("Target reliability must be between 0 and 100"));
        }
        
        Ok(())
    }
    
    fn get_defaults(env: &Environment) -> Self {
        match env {
            Environment::Production => Self {
                throughput_test_message_count: 1_000_000,
                throughput_test_chunk_count: 20,
                throughput_test_sleep_between_batches_ms: 50,
                batch_test_message_count: 6400,
                batch_test_expected_batch_size: 64,
                batch_test_sleep_before_check_ms: 25,
                memory_test_iterations: 10_000,
                memory_test_sleep_ms: 10,
                target_latency_us: 0.5,
                target_p99_latency_us: 2.0,
                target_throughput_per_sec: 5_000_000,
                target_memory_mb: 1000,
                target_reliability_percent: 99.99,
                default_shutdown_timeout_secs: 60,
                default_run_duration_secs: 300,
                demo_btc_price: 50000.00,
                demo_btc_volume: 1.5,
            },
            Environment::Staging => Self {
                throughput_test_message_count: 500_000,
                throughput_test_chunk_count: 10,
                throughput_test_sleep_between_batches_ms: 75,
                batch_test_message_count: 3200,
                batch_test_expected_batch_size: 32,
                batch_test_sleep_before_check_ms: 50,
                memory_test_iterations: 5_000,
                memory_test_sleep_ms: 20,
                target_latency_us: 1.0,
                target_p99_latency_us: 5.0,
                target_throughput_per_sec: 2_000_000,
                target_memory_mb: 500,
                target_reliability_percent: 99.9,
                default_shutdown_timeout_secs: 45,
                default_run_duration_secs: 180,
                demo_btc_price: 48000.00,
                demo_btc_volume: 2.0,
            },
            Environment::Testing => Self {
                throughput_test_message_count: 10_000,
                throughput_test_chunk_count: 5,
                throughput_test_sleep_between_batches_ms: 100,
                batch_test_message_count: 320,
                batch_test_expected_batch_size: 16,
                batch_test_sleep_before_check_ms: 100,
                memory_test_iterations: 1_000,
                memory_test_sleep_ms: 50,
                target_latency_us: 5.0,
                target_p99_latency_us: 20.0,
                target_throughput_per_sec: 100_000,
                target_memory_mb: 100,
                target_reliability_percent: 99.0,
                default_shutdown_timeout_secs: 15,
                default_run_duration_secs: 60,
                demo_btc_price: 45000.00,
                demo_btc_volume: 0.5,
            },
            Environment::Development => Self {
                throughput_test_message_count: 100_000,
                throughput_test_chunk_count: 10,
                throughput_test_sleep_between_batches_ms: 100,
                batch_test_message_count: 640,
                batch_test_expected_batch_size: 64,
                batch_test_sleep_before_check_ms: 50,
                memory_test_iterations: 1_000,
                memory_test_sleep_ms: 25,
                target_latency_us: 1.0,
                target_p99_latency_us: 5.0,
                target_throughput_per_sec: 1_000_000,
                target_memory_mb: 500,
                target_reliability_percent: 99.99,
                default_shutdown_timeout_secs: 30,
                default_run_duration_secs: 60,
                demo_btc_price: 50000.00,
                demo_btc_volume: 1.5,
            },
        }
    }
}

impl BenchmarkConfig {
    /// Get chunk size for throughput test
    pub fn throughput_chunk_size(&self) -> u64 {
        self.throughput_test_message_count / self.throughput_test_chunk_count as u64
    }
    
    /// Get sleep duration between batches
    pub fn throughput_sleep_duration(&self) -> Duration {
        Duration::from_millis(self.throughput_test_sleep_between_batches_ms)
    }
    
    /// Get batch test sleep duration
    pub fn batch_sleep_duration(&self) -> Duration {
        Duration::from_millis(self.batch_test_sleep_before_check_ms)
    }
    
    /// Get memory test sleep duration
    pub fn memory_sleep_duration(&self) -> Duration {
        Duration::from_millis(self.memory_test_sleep_ms)
    }
    
    /// Get default shutdown timeout
    pub fn default_shutdown_timeout(&self) -> Duration {
        Duration::from_secs(self.default_shutdown_timeout_secs)
    }
    
    /// Format demo BTC message
    pub fn demo_btc_message(&self) -> String {
        format!("BTCUSD|{:.2}|{:.1}|BUY|{}", 
                self.demo_btc_price, 
                self.demo_btc_volume,
                chrono::Utc::now().timestamp())
    }
    
    /// Get expected batches for batch test
    pub fn expected_batches(&self) -> u64 {
        self.batch_test_message_count / self.batch_test_expected_batch_size
    }
    
    /// Get throughput display string
    pub fn throughput_display(&self) -> String {
        if self.target_throughput_per_sec >= 1_000_000 {
            format!("{}M", self.target_throughput_per_sec / 1_000_000)
        } else if self.target_throughput_per_sec >= 1_000 {
            format!("{}K", self.target_throughput_per_sec / 1_000)
        } else {
            format!("{}", self.target_throughput_per_sec)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_benchmark_config_defaults() {
        let prod_config = BenchmarkConfig::get_defaults(&Environment::Production);
        assert_eq!(prod_config.throughput_test_message_count, 1_000_000);
        assert_eq!(prod_config.target_latency_us, 0.5);
        assert_eq!(prod_config.target_throughput_per_sec, 5_000_000);
        
        let dev_config = BenchmarkConfig::get_defaults(&Environment::Development);
        assert_eq!(dev_config.throughput_test_message_count, 100_000);
        assert_eq!(dev_config.target_latency_us, 1.0);
        assert_eq!(dev_config.target_memory_mb, 500);
        assert_eq!(dev_config.default_shutdown_timeout_secs, 30);
    }
    
    #[test]
    fn test_benchmark_config_validation() {
        let mut config = BenchmarkConfig::get_defaults(&Environment::Development);
        assert!(config.validate().is_ok());
        
        config.throughput_test_message_count = 0;
        assert!(config.validate().is_err());
        
        config.throughput_test_message_count = 100_000;
        config.target_latency_us = 0.0;
        assert!(config.validate().is_err());
        
        config.target_latency_us = 1.0;
        config.target_reliability_percent = 150.0;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_benchmark_computed_values() {
        let config = BenchmarkConfig::get_defaults(&Environment::Development);
        
        // Test computed values
        assert_eq!(config.throughput_chunk_size(), 10_000);
        assert_eq!(config.expected_batches(), 10);
        assert_eq!(config.throughput_display(), "1M");
        
        // Test durations
        assert_eq!(config.throughput_sleep_duration(), Duration::from_millis(100));
        assert_eq!(config.batch_sleep_duration(), Duration::from_millis(50));
        assert_eq!(config.default_shutdown_timeout(), Duration::from_secs(30));
    }
    
    #[test]
    fn test_throughput_display() {
        let mut config = BenchmarkConfig::get_defaults(&Environment::Development);
        
        config.target_throughput_per_sec = 5_000_000;
        assert_eq!(config.throughput_display(), "5M");
        
        config.target_throughput_per_sec = 500_000;
        assert_eq!(config.throughput_display(), "500K");
        
        config.target_throughput_per_sec = 500;
        assert_eq!(config.throughput_display(), "500");
    }
    
    #[test]
    fn test_environment_variable_override() {
        // Set a test environment variable
        env::set_var("BENCH_THROUGHPUT_MESSAGE_COUNT", "50000");
        
        let config = BenchmarkConfig::from_env().unwrap();
        assert_eq!(config.throughput_test_message_count, 50_000);
        assert_eq!(config.throughput_chunk_size(), 5_000);
        
        // Clean up
        env::remove_var("BENCH_THROUGHPUT_MESSAGE_COUNT");
    }
    
    #[test]
    fn test_demo_message_format() {
        let mut config = BenchmarkConfig::get_defaults(&Environment::Development);
        config.demo_btc_price = 45000.50;
        config.demo_btc_volume = 2.3;
        
        let message = config.demo_btc_message();
        assert!(message.contains("BTCUSD"));
        assert!(message.contains("45000.50"));
        assert!(message.contains("2.3"));
        assert!(message.contains("BUY"));
    }
}

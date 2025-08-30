//! Ultra Logger Configuration
//!
//! Configuration for the ultra-low latency logger component.

use super::*;

/// Configuration for the ultra-low latency logger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UltraLoggerConfig {
    /// Log level filter
    pub level: String,
    
    /// Buffer configuration
    pub buffer: BufferConfig,
    
    /// Transport configuration
    pub transport: TransportConfig,
    
    /// Compression settings
    pub compression: CompressionConfig,
    
    /// Performance tuning
    pub performance: PerformanceConfig,
    
    /// Output destinations
    pub outputs: Vec<OutputConfig>,
    
    /// Metrics configuration
    pub metrics: UltraLoggerMetricsConfig,
    
    /// Tracing configuration
    pub tracing: TracingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferConfig {
    /// Ring buffer size (must be power of 2)
    pub ring_buffer_size: usize,
    
    /// Batch size for processing
    pub batch_size: usize,
    
    /// Flush interval in microseconds
    pub flush_interval_micros: u64,
    
    /// Maximum memory usage in bytes
    pub max_memory_bytes: usize,
    
    /// Pre-allocate buffers
    pub pre_allocate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    /// Transport type (redis, file, console, network)
    pub transport_type: String,
    
    /// Connection pool size
    pub pool_size: usize,
    
    /// Connection timeout
    pub timeout_millis: u64,
    
    /// Retry configuration
    pub retry_attempts: usize,
    pub retry_delay_millis: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Enable compression
    pub enabled: bool,
    
    /// Compression algorithm (gzip, lz4, zstd)
    pub algorithm: String,
    
    /// Compression level (1-9)
    pub level: u8,
    
    /// Minimum size to compress
    pub min_size_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Use SIMD optimizations
    pub simd_enabled: bool,
    
    /// Lock-free operations
    pub lock_free: bool,
    
    /// Memory pool size
    pub memory_pool_size: usize,
    
    /// Worker thread count
    pub worker_threads: usize,
    
    /// CPU affinity
    pub cpu_affinity: Option<Vec<usize>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Output type (file, console, syslog, network)
    pub output_type: String,
    
    /// Output destination (file path, host:port, etc.)
    pub destination: String,
    
    /// Output format (json, text, binary)
    pub format: String,
    
    /// Buffering configuration
    pub buffered: bool,
    pub buffer_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UltraLoggerMetricsConfig {
    /// Enable internal metrics
    pub enabled: bool,
    
    /// Metrics collection interval
    pub collection_interval_millis: u64,
    
    /// Histogram tracking
    pub histogram_enabled: bool,
    
    /// Memory usage tracking
    pub memory_tracking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Enable distributed tracing
    pub enabled: bool,
    
    /// Tracing service name
    pub service_name: String,
    
    /// Sampling rate (0.0 - 1.0)
    pub sampling_rate: f64,
    
    /// Jaeger endpoint
    pub jaeger_endpoint: Option<String>,
}

impl ConfigLoader for UltraLoggerConfig {
    fn from_env() -> Result<Self> {
        let environment = Environment::from_str(&env_string_or_default("LOGGING_ENVIRONMENT", "development"));
        let defaults = Self::get_defaults(&environment);
        
        Ok(Self {
            level: env_string_or_default("ULTRA_LOG_LEVEL", &defaults.level),
            buffer: BufferConfig::from_env_with_defaults(&defaults.buffer),
            transport: TransportConfig::from_env_with_defaults(&defaults.transport),
            compression: CompressionConfig::from_env_with_defaults(&defaults.compression),
            performance: PerformanceConfig::from_env_with_defaults(&defaults.performance),
            outputs: parse_outputs_from_env(),
            metrics: UltraLoggerMetricsConfig::from_env_with_defaults(&defaults.metrics),
            tracing: TracingConfig::from_env_with_defaults(&defaults.tracing),
        })
    }
    
    fn validate(&self) -> Result<()> {
        // Validate ring buffer size is power of 2
        if !self.buffer.ring_buffer_size.is_power_of_two() {
            return Err(anyhow!("Ring buffer size must be a power of 2"));
        }
        
        if self.buffer.batch_size == 0 {
            return Err(anyhow!("Batch size must be greater than 0"));
        }
        
        if self.buffer.max_memory_bytes == 0 {
            return Err(anyhow!("Max memory bytes must be greater than 0"));
        }
        
        if self.performance.worker_threads == 0 {
            return Err(anyhow!("Worker threads must be greater than 0"));
        }
        
        if self.tracing.sampling_rate < 0.0 || self.tracing.sampling_rate > 1.0 {
            return Err(anyhow!("Sampling rate must be between 0.0 and 1.0"));
        }
        
        Ok(())
    }
    
    fn get_defaults(env: &Environment) -> Self {
        let (ring_buffer_size, batch_size, memory_mb, pool_size, workers) = match env {
            Environment::Production => (65536, 64, 256, 1000, 8),
            Environment::Staging => (32768, 32, 128, 500, 4),
            Environment::Testing => (16384, 16, 64, 100, 2),
            Environment::Development => (16384, 16, 64, 100, 2),
        };
        
        Self {
            level: match env {
                Environment::Production | Environment::Staging => "info",
                Environment::Testing | Environment::Development => "debug",
            }.to_string(),
            buffer: BufferConfig {
                ring_buffer_size,
                batch_size,
                flush_interval_micros: match env {
                    Environment::Production => 100,
                    Environment::Staging => 200,
                    _ => 500,
                },
                max_memory_bytes: memory_mb * 1024 * 1024,
                pre_allocate: matches!(env, Environment::Production | Environment::Staging),
            },
            transport: TransportConfig {
                transport_type: "redis".to_string(),
                pool_size,
                timeout_millis: match env {
                    Environment::Production => 5000,
                    Environment::Staging => 3000,
                    _ => 1000,
                },
                retry_attempts: match env {
                    Environment::Production => 5,
                    Environment::Staging => 3,
                    _ => 2,
                },
                retry_delay_millis: 100,
            },
            compression: CompressionConfig {
                enabled: matches!(env, Environment::Production | Environment::Staging),
                algorithm: "lz4".to_string(),
                level: 1,
                min_size_bytes: 1024,
            },
            performance: PerformanceConfig {
                simd_enabled: matches!(env, Environment::Production | Environment::Staging),
                lock_free: true,
                memory_pool_size: pool_size,
                worker_threads: workers,
                cpu_affinity: None,
            },
            outputs: vec![OutputConfig {
                output_type: "console".to_string(),
                destination: "stdout".to_string(),
                format: "json".to_string(),
                buffered: true,
                buffer_size: 8192,
            }],
            metrics: UltraLoggerMetricsConfig {
                enabled: matches!(env, Environment::Production | Environment::Staging),
                collection_interval_millis: 1000,
                histogram_enabled: matches!(env, Environment::Production),
                memory_tracking: true,
            },
            tracing: TracingConfig {
                enabled: matches!(env, Environment::Production | Environment::Staging),
                service_name: "ultra-logger".to_string(),
                sampling_rate: match env {
                    Environment::Production => 0.1,
                    Environment::Staging => 0.5,
                    _ => 1.0,
                },
                jaeger_endpoint: match env {
                    Environment::Production => Some("http://jaeger.prod.svc.cluster.local:14268".to_string()),
                    Environment::Staging => Some("http://jaeger.staging.svc.cluster.local:14268".to_string()),
                    _ => None,
                },
            },
        }
    }
}

// Implementation helpers for sub-configs
impl BufferConfig {
    fn from_env_with_defaults(defaults: &BufferConfig) -> Self {
        Self {
            ring_buffer_size: env_var_or_default("ULTRA_RING_BUFFER_SIZE", defaults.ring_buffer_size),
            batch_size: env_var_or_default("ULTRA_BATCH_SIZE", defaults.batch_size),
            flush_interval_micros: env_var_or_default("ULTRA_FLUSH_INTERVAL_MICROS", defaults.flush_interval_micros),
            max_memory_bytes: env_var_or_default("ULTRA_MAX_MEMORY_BYTES", defaults.max_memory_bytes),
            pre_allocate: env_bool_or_default("ULTRA_PRE_ALLOCATE", defaults.pre_allocate),
        }
    }
}

impl TransportConfig {
    fn from_env_with_defaults(defaults: &TransportConfig) -> Self {
        Self {
            transport_type: env_string_or_default("ULTRA_TRANSPORT_TYPE", &defaults.transport_type),
            pool_size: env_var_or_default("ULTRA_POOL_SIZE", defaults.pool_size),
            timeout_millis: env_var_or_default("ULTRA_TIMEOUT_MS", defaults.timeout_millis),
            retry_attempts: env_var_or_default("ULTRA_RETRY_ATTEMPTS", defaults.retry_attempts),
            retry_delay_millis: env_var_or_default("ULTRA_RETRY_DELAY_MS", defaults.retry_delay_millis),
        }
    }
}

impl CompressionConfig {
    fn from_env_with_defaults(defaults: &CompressionConfig) -> Self {
        Self {
            enabled: env_bool_or_default("ULTRA_COMPRESSION_ENABLED", defaults.enabled),
            algorithm: env_string_or_default("ULTRA_COMPRESSION_ALGORITHM", &defaults.algorithm),
            level: env_var_or_default("ULTRA_COMPRESSION_LEVEL", defaults.level),
            min_size_bytes: env_var_or_default("ULTRA_COMPRESSION_MIN_SIZE", defaults.min_size_bytes),
        }
    }
}

impl PerformanceConfig {
    fn from_env_with_defaults(defaults: &PerformanceConfig) -> Self {
        Self {
            simd_enabled: env_bool_or_default("ULTRA_SIMD_ENABLED", defaults.simd_enabled),
            lock_free: env_bool_or_default("ULTRA_LOCK_FREE", defaults.lock_free),
            memory_pool_size: env_var_or_default("ULTRA_MEMORY_POOL_SIZE", defaults.memory_pool_size),
            worker_threads: env_var_or_default("ULTRA_WORKER_THREADS", defaults.worker_threads),
            cpu_affinity: parse_cpu_affinity(),
        }
    }
}

impl UltraLoggerMetricsConfig {
    fn from_env_with_defaults(defaults: &UltraLoggerMetricsConfig) -> Self {
        Self {
            enabled: env_bool_or_default("ULTRA_METRICS_ENABLED", defaults.enabled),
            collection_interval_millis: env_var_or_default("ULTRA_METRICS_INTERVAL_MS", defaults.collection_interval_millis),
            histogram_enabled: env_bool_or_default("ULTRA_HISTOGRAM_ENABLED", defaults.histogram_enabled),
            memory_tracking: env_bool_or_default("ULTRA_MEMORY_TRACKING", defaults.memory_tracking),
        }
    }
}

impl TracingConfig {
    fn from_env_with_defaults(defaults: &TracingConfig) -> Self {
        Self {
            enabled: env_bool_or_default("ULTRA_TRACING_ENABLED", defaults.enabled),
            service_name: env_string_or_default("ULTRA_SERVICE_NAME", &defaults.service_name),
            sampling_rate: env_var_or_default("ULTRA_SAMPLING_RATE", defaults.sampling_rate),
            jaeger_endpoint: env::var("ULTRA_JAEGER_ENDPOINT").ok().or_else(|| defaults.jaeger_endpoint.clone()),
        }
    }
}

fn parse_outputs_from_env() -> Vec<OutputConfig> {
    // Default to console output if not specified
    vec![OutputConfig {
        output_type: env_string_or_default("ULTRA_OUTPUT_TYPE", "console"),
        destination: env_string_or_default("ULTRA_OUTPUT_DESTINATION", "stdout"),
        format: env_string_or_default("ULTRA_OUTPUT_FORMAT", "json"),
        buffered: env_bool_or_default("ULTRA_OUTPUT_BUFFERED", true),
        buffer_size: env_var_or_default("ULTRA_OUTPUT_BUFFER_SIZE", 8192),
    }]
}

fn parse_cpu_affinity() -> Option<Vec<usize>> {
    env::var("ULTRA_CPU_AFFINITY")
        .ok()
        .and_then(|s| {
            s.split(',')
                .map(|cpu| cpu.trim().parse::<usize>())
                .collect::<Result<Vec<_>, _>>()
                .ok()
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ultra_logger_config_defaults() {
        let prod_config = UltraLoggerConfig::get_defaults(&Environment::Production);
        assert_eq!(prod_config.buffer.ring_buffer_size, 65536);
        assert_eq!(prod_config.buffer.batch_size, 64);
        assert_eq!(prod_config.performance.simd_enabled, true);
        assert_eq!(prod_config.compression.enabled, true);
        
        let dev_config = UltraLoggerConfig::get_defaults(&Environment::Development);
        assert_eq!(dev_config.buffer.ring_buffer_size, 16384);
        assert_eq!(dev_config.performance.simd_enabled, false);
        assert_eq!(dev_config.compression.enabled, false);
    }
    
    #[test]
    fn test_ultra_logger_config_validation() {
        let mut config = UltraLoggerConfig::get_defaults(&Environment::Development);
        assert!(config.validate().is_ok());
        
        config.buffer.ring_buffer_size = 100; // Not power of 2
        assert!(config.validate().is_err());
        
        config.buffer.ring_buffer_size = 16384;
        config.buffer.batch_size = 0;
        assert!(config.validate().is_err());
        
        config.buffer.batch_size = 16;
        config.tracing.sampling_rate = 1.5; // Invalid range
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_buffer_config_power_of_two() {
        assert!(16384_usize.is_power_of_two());
        assert!(65536_usize.is_power_of_two());
        assert!(!100_usize.is_power_of_two());
    }
    
    #[test]
    fn test_cpu_affinity_parsing() {
        env::set_var("ULTRA_CPU_AFFINITY", "0,1,2,3");
        let affinity = parse_cpu_affinity();
        assert_eq!(affinity, Some(vec![0, 1, 2, 3]));
        
        env::remove_var("ULTRA_CPU_AFFINITY");
        let no_affinity = parse_cpu_affinity();
        assert_eq!(no_affinity, None);
    }
}

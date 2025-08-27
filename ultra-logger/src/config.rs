//! Configuration module for ultra-low latency logging

use serde::{Deserialize, Serialize};

/// Configuration for the ultra-low latency logger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggerConfig {
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
    pub metrics: MetricsConfig,
    
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
    /// Transport type: "file", "kafka", "redis", "tcp", "udp"
    pub transport_type: String,
    
    /// Connection settings
    pub connection: ConnectionConfig,
    
    /// Retry configuration
    pub retry: RetryConfig,
    
    /// Queue settings
    pub queue_size: usize,
    
    /// Async or sync mode
    pub async_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// Host/endpoint
    pub host: String,
    
    /// Port
    pub port: u16,
    
    /// Username
    pub username: Option<String>,
    
    /// Password
    pub password: Option<String>,
    
    /// Connection timeout
    pub timeout_ms: u64,
    
    /// Additional options
    pub options: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_retries: u32,
    
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
    
    /// Exponential backoff factor
    pub backoff_factor: f64,
    
    /// Maximum retry delay
    pub max_retry_delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Enable compression
    pub enabled: bool,
    
    /// Compression algorithm: "lz4", "zstd", "gzip"
    pub algorithm: String,
    
    /// Compression level (1-9)
    pub level: u32,
    
    /// Minimum size to compress
    pub min_size_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Number of worker threads
    pub worker_threads: usize,
    
    /// Use lock-free data structures
    pub lock_free: bool,
    
    /// Memory pool size
    pub memory_pool_size: usize,
    
    /// CPU affinity (optional)
    pub cpu_affinity: Option<Vec<usize>>,
    
    /// Use io_uring (Linux only)
    pub use_io_uring: bool,
    
    /// NUMA awareness
    pub numa_aware: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Output type: "stdout", "stderr", "file", "kafka", "redis"
    pub output_type: String,
    
    /// Output path/destination
    pub destination: String,
    
    /// Format: "json", "logfmt", "custom"
    pub format: String,
    
    /// Rotation settings for file outputs
    pub rotation: Option<RotationConfig>,
    
    /// Filter expression
    pub filter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationConfig {
    /// Maximum file size in bytes
    pub max_size_bytes: usize,
    
    /// Maximum number of files to keep
    pub max_files: usize,
    
    /// Rotation interval
    pub interval: String, // "hourly", "daily", "weekly"
    
    /// Compress rotated files
    pub compress: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,
    
    /// Metrics endpoint
    pub endpoint: String,
    
    /// Metrics port
    pub port: u16,
    
    /// Collection interval in seconds
    pub collection_interval_seconds: u64,
    
    /// Histogram buckets for latency
    pub latency_buckets: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Enable distributed tracing
    pub enabled: bool,
    
    /// Jaeger endpoint
    pub jaeger_endpoint: Option<String>,
    
    /// Service name
    pub service_name: String,
    
    /// Trace sample rate (0.0 to 1.0)
    pub sample_rate: f64,
    
    /// Enable trace correlation
    pub correlation_enabled: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            buffer: BufferConfig::default(),
            transport: TransportConfig::default(),
            compression: CompressionConfig::default(),
            performance: PerformanceConfig::default(),
            outputs: vec![OutputConfig::default()],
            metrics: MetricsConfig::default(),
            tracing: TracingConfig::default(),
        }
    }
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            ring_buffer_size: 1_048_576, // 1MB ring buffer
            batch_size: 1000,
            flush_interval_micros: 100, // 100 microseconds
            max_memory_bytes: 100 * 1024 * 1024, // 100MB
            pre_allocate: true,
        }
    }
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            transport_type: "file".to_string(),
            connection: ConnectionConfig::default(),
            retry: RetryConfig::default(),
            queue_size: 10_000,
            async_mode: true,
        }
    }
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 9092,
            username: None,
            password: None,
            timeout_ms: 5000,
            options: std::collections::HashMap::new(),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay_ms: 100,
            backoff_factor: 2.0,
            max_retry_delay_ms: 5000,
        }
    }
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: "lz4".to_string(),
            level: 1, // Fast compression
            min_size_bytes: 1024, // Only compress if >1KB
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            worker_threads: num_cpus::get(),
            lock_free: true,
            memory_pool_size: 10 * 1024 * 1024, // 10MB
            cpu_affinity: None,
            use_io_uring: false, // Platform dependent
            numa_aware: false,
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            output_type: "stdout".to_string(),
            destination: "/dev/stdout".to_string(),
            format: "json".to_string(),
            rotation: None,
            filter: None,
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "/metrics".to_string(),
            port: 9090,
            collection_interval_seconds: 10,
            latency_buckets: vec![
                0.000_001, // 1μs
                0.000_010, // 10μs  
                0.000_100, // 100μs
                0.001,     // 1ms
                0.010,     // 10ms
                0.100,     // 100ms
                1.0,       // 1s
            ],
        }
    }
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            jaeger_endpoint: Some("http://localhost:14268".to_string()),
            service_name: "trading-platform".to_string(),
            sample_rate: 1.0,
            correlation_enabled: true,
        }
    }
}

/// Load configuration from file
impl LoggerConfig {
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: LoggerConfig = toml::from_str(&contents)?;
        Ok(config)
    }
    
    pub fn from_env() -> anyhow::Result<Self> {
        let mut config = Self::default();
        
        if let Ok(level) = std::env::var("LOG_LEVEL") {
            config.level = level;
        }
        
        if let Ok(workers) = std::env::var("LOG_WORKER_THREADS") {
            config.performance.worker_threads = workers.parse()?;
        }
        
        if let Ok(buffer_size) = std::env::var("LOG_BUFFER_SIZE") {
            config.buffer.ring_buffer_size = buffer_size.parse()?;
        }
        
        Ok(config)
    }
    
    pub fn validate(&self) -> anyhow::Result<()> {
        // Validate buffer size is power of 2
        if !self.buffer.ring_buffer_size.is_power_of_two() {
            anyhow::bail!("Ring buffer size must be power of 2");
        }
        
        // Validate worker threads
        if self.performance.worker_threads == 0 {
            anyhow::bail!("Worker threads must be > 0");
        }
        
        // Validate compression level
        if self.compression.level > 9 {
            anyhow::bail!("Compression level must be 1-9");
        }
        
        Ok(())
    }
}

//! Ultra-low latency logger implementation

use crate::config::LoggerConfig;
use crate::error::{LoggingError, Result};
use crate::buffer::RingBuffer;
use crate::transport::Transport;
use crate::metrics::LoggingMetrics;

use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use crossbeam_channel::{unbounded, Receiver, Sender};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use dashmap::DashMap;
use once_cell::sync::OnceCell;

/// Global logger instance
static GLOBAL_LOGGER: OnceCell<UltraLogger> = OnceCell::new();

/// Log levels optimized for trading systems
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum LogLevel {
    /// Critical system errors that require immediate attention
    Critical = 0,
    /// Errors that affect trading operations
    Error = 1,
    /// Warnings about potential issues
    Warn = 2,
    /// General information about trading operations
    Info = 3,
    /// Debugging information
    Debug = 4,
    /// Detailed tracing information
    Trace = 5,
    /// Market data events (special category)
    MarketData = 6,
    /// Trade execution events (special category)
    Trade = 7,
    /// Order events (special category)
    Order = 8,
    /// Risk management events (special category)
    Risk = 9,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "critical" | "crit" => Ok(Self::Critical),
            "error" | "err" => Ok(Self::Error),
            "warn" | "warning" => Ok(Self::Warn),
            "info" => Ok(Self::Info),
            "debug" => Ok(Self::Debug),
            "trace" => Ok(Self::Trace),
            "market_data" | "market" => Ok(Self::MarketData),
            "trade" => Ok(Self::Trade),
            "order" => Ok(Self::Order),
            "risk" => Ok(Self::Risk),
            _ => Err(LoggingError::InvalidLogLevel(s.to_string())),
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Critical => "CRIT",
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Debug => "DEBUG",
            Self::Trace => "TRACE",
            Self::MarketData => "MARKET",
            Self::Trade => "TRADE",
            Self::Order => "ORDER",
            Self::Risk => "RISK",
        }
    }
}

/// High-performance log entry optimized for zero-copy operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Unique log entry ID
    pub id: Uuid,
    
    /// Timestamp in nanoseconds since UNIX epoch
    pub timestamp_nanos: u64,
    
    /// Log level
    pub level: LogLevel,
    
    /// Service/component name
    pub service: String,
    
    /// Log message
    pub message: String,
    
    /// Structured fields
    pub fields: DashMap<String, LogValue>,
    
    /// Trace ID for correlation
    pub trace_id: Option<String>,
    
    /// Span ID for distributed tracing
    pub span_id: Option<String>,
    
    /// Thread ID where log was created
    pub thread_id: u64,
    
    /// Source location (file:line)
    pub location: Option<String>,
}

/// Optimized log value types for high performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Bytes(Vec<u8>),
    Null,
}

impl LogEntry {
    pub fn new(level: LogLevel, service: &str, message: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp_nanos: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
            level,
            service: service.to_string(),
            message: message.to_string(),
            fields: DashMap::new(),
            trace_id: None,
            span_id: None,
            thread_id: std::thread::current().id().as_u64().get(),
            location: None,
        }
    }
    
    pub fn with_field(mut self, key: &str, value: LogValue) -> Self {
        self.fields.insert(key.to_string(), value);
        self
    }
    
    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.trace_id = Some(trace_id);
        self
    }
    
    pub fn with_span_id(mut self, span_id: String) -> Self {
        self.span_id = Some(span_id);
        self
    }
    
    pub fn with_location(mut self, file: &str, line: u32) -> Self {
        self.location = Some(format!("{}:{}", file, line));
        self
    }
    
    /// Serialize to JSON with SIMD optimization
    pub fn to_json_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::with_capacity(1024); // Pre-allocate
        simd_json::to_writer(&mut buffer, self)
            .map_err(|e| LoggingError::SerializationError(e.to_string()))?;
        Ok(buffer)
    }
    
    /// Calculate memory size for metrics
    pub fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>() +
        self.service.len() +
        self.message.len() +
        self.fields.iter().map(|entry| {
            entry.key().len() + entry.value().memory_size()
        }).sum::<usize>() +
        self.trace_id.as_ref().map_or(0, |s| s.len()) +
        self.span_id.as_ref().map_or(0, |s| s.len()) +
        self.location.as_ref().map_or(0, |s| s.len())
    }
}

impl LogValue {
    pub fn memory_size(&self) -> usize {
        match self {
            Self::String(s) => s.len(),
            Self::Integer(_) => 8,
            Self::Float(_) => 8,
            Self::Boolean(_) => 1,
            Self::Bytes(b) => b.len(),
            Self::Null => 0,
        }
    }
}

/// Ultra-low latency logger with lock-free operations
pub struct UltraLogger {
    config: LoggerConfig,
    buffer: Arc<RingBuffer<LogEntry>>,
    transport: Arc<dyn Transport + Send + Sync>,
    metrics: Arc<LoggingMetrics>,
    log_sender: Sender<LogEntry>,
    _log_receiver: Receiver<LogEntry>,
    sequence: AtomicU64,
    is_running: Arc<std::sync::atomic::AtomicBool>,
}

impl UltraLogger {
    /// Initialize the global logger
    pub fn init(config: LoggerConfig) -> Result<()> {
        config.validate()?;
        
        let logger = Self::new(config)?;
        
        GLOBAL_LOGGER.set(logger)
            .map_err(|_| LoggingError::AlreadyInitialized)?;
        
        Ok(())
    }
    
    /// Get the global logger instance
    pub fn global() -> &'static UltraLogger {
        GLOBAL_LOGGER.get().expect("Logger not initialized")
    }
    
    /// Create a new logger instance
    pub fn new(config: LoggerConfig) -> Result<Self> {
        let buffer = Arc::new(RingBuffer::new(config.buffer.ring_buffer_size)?);
        let transport = crate::transport::create_transport(&config.transport)?;
        let metrics = Arc::new(LoggingMetrics::new(&config.metrics)?);
        let (log_sender, log_receiver) = unbounded();
        
        let logger = Self {
            config,
            buffer,
            transport,
            metrics,
            log_sender,
            _log_receiver: log_receiver,
            sequence: AtomicU64::new(0),
            is_running: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        };
        
        // Start background processing
        logger.start_background_processing()?;
        
        Ok(logger)
    }
    
    /// Log an entry with microsecond precision
    #[inline]
    pub fn log(&self, entry: LogEntry) -> Result<()> {
        let start = Instant::now();
        
        // Fast path: try to write directly to ring buffer
        if self.buffer.try_write(entry.clone()).is_ok() {
            self.metrics.record_log_latency(start.elapsed());
            self.metrics.increment_logs_processed();
            return Ok(());
        }
        
        // Slow path: send through channel if buffer is full
        self.log_sender.send(entry)
            .map_err(|_| LoggingError::ChannelSendError)?;
        
        self.metrics.record_log_latency(start.elapsed());
        self.metrics.increment_logs_queued();
        
        Ok(())
    }
    
    /// Log with level and message (convenience method)
    #[inline]
    pub fn log_simple(&self, level: LogLevel, service: &str, message: &str) -> Result<()> {
        let entry = LogEntry::new(level, service, message);
        self.log(entry)
    }
    
    /// Log with structured fields
    #[inline]  
    pub fn log_with_fields(
        &self, 
        level: LogLevel, 
        service: &str, 
        message: &str,
        fields: Vec<(&str, LogValue)>
    ) -> Result<()> {
        let mut entry = LogEntry::new(level, service, message);
        for (key, value) in fields {
            entry.fields.insert(key.to_string(), value);
        }
        self.log(entry)
    }
    
    /// Get next sequence number
    pub fn next_sequence(&self) -> u64 {
        self.sequence.fetch_add(1, Ordering::Relaxed)
    }
    
    /// Flush all pending logs
    pub async fn flush(&self) -> Result<()> {
        // Process any remaining entries in the ring buffer
        while let Some(entry) = self.buffer.try_read() {
            self.transport.send(entry).await?;
        }
        
        // Flush transport
        self.transport.flush().await?;
        
        Ok(())
    }
    
    /// Get logger metrics
    pub fn metrics(&self) -> &LoggingMetrics {
        &self.metrics
    }
    
    /// Start background processing workers
    fn start_background_processing(&self) -> Result<()> {
        let buffer = self.buffer.clone();
        let transport = self.transport.clone();
        let metrics = self.metrics.clone();
        let is_running = self.is_running.clone();
        let batch_size = self.config.buffer.batch_size;
        let flush_interval = std::time::Duration::from_micros(self.config.buffer.flush_interval_micros);
        
        // Start background worker
        tokio::spawn(async move {
            let mut batch = Vec::with_capacity(batch_size);
            let mut last_flush = Instant::now();
            
            while is_running.load(Ordering::Relaxed) {
                let mut processed = 0;
                
                // Collect batch
                while batch.len() < batch_size {
                    if let Some(entry) = buffer.try_read() {
                        batch.push(entry);
                        processed += 1;
                    } else {
                        break;
                    }
                }
                
                // Send batch if we have entries or it's time to flush
                if !batch.is_empty() && (batch.len() >= batch_size || last_flush.elapsed() >= flush_interval) {
                    let batch_start = Instant::now();
                    
                    // Send batch
                    if let Err(e) = transport.send_batch(&batch).await {
                        eprintln!("Failed to send log batch: {}", e);
                        metrics.increment_transport_errors();
                    } else {
                        metrics.record_batch_latency(batch_start.elapsed());
                        metrics.add_logs_processed(batch.len() as u64);
                    }
                    
                    batch.clear();
                    last_flush = Instant::now();
                }
                
                // Yield if no work was done
                if processed == 0 {
                    tokio::time::sleep(std::time::Duration::from_micros(1)).await;
                }
            }
        });
        
        Ok(())
    }
    
    /// Shutdown the logger
    pub async fn shutdown(&self) -> Result<()> {
        self.is_running.store(false, Ordering::Relaxed);
        
        // Final flush
        self.flush().await?;
        
        // Shutdown transport
        self.transport.shutdown().await?;
        
        Ok(())
    }
}

/// Convenience macros for ultra-fast logging
#[macro_export]
macro_rules! ultra_log {
    ($level:expr, $service:expr, $message:expr) => {
        if let Some(logger) = $crate::GLOBAL_LOGGER.get() {
            let _ = logger.log_simple($level, $service, $message);
        }
    };
    
    ($level:expr, $service:expr, $message:expr, $($key:expr => $value:expr),+) => {
        if let Some(logger) = $crate::GLOBAL_LOGGER.get() {
            let fields = vec![$(($key, $value)),+];
            let _ = logger.log_with_fields($level, $service, $message, fields);
        }
    };
}

#[macro_export]
macro_rules! ultra_error {
    ($service:expr, $message:expr) => {
        $crate::ultra_log!($crate::LogLevel::Error, $service, $message);
    };
    ($service:expr, $message:expr, $($key:expr => $value:expr),+) => {
        $crate::ultra_log!($crate::LogLevel::Error, $service, $message, $($key => $value),+);
    };
}

#[macro_export]
macro_rules! ultra_warn {
    ($service:expr, $message:expr) => {
        $crate::ultra_log!($crate::LogLevel::Warn, $service, $message);
    };
    ($service:expr, $message:expr, $($key:expr => $value:expr),+) => {
        $crate::ultra_log!($crate::LogLevel::Warn, $service, $message, $($key => $value),+);
    };
}

#[macro_export]
macro_rules! ultra_info {
    ($service:expr, $message:expr) => {
        $crate::ultra_log!($crate::LogLevel::Info, $service, $message);
    };
    ($service:expr, $message:expr, $($key:expr => $value:expr),+) => {
        $crate::ultra_log!($crate::LogLevel::Info, $service, $message, $($key => $value),+);
    };
}

#[macro_export]
macro_rules! ultra_debug {
    ($service:expr, $message:expr) => {
        $crate::ultra_log!($crate::LogLevel::Debug, $service, $message);
    };
    ($service:expr, $message:expr, $($key:expr => $value:expr),+) => {
        $crate::ultra_log!($crate::LogLevel::Debug, $service, $message, $($key => $value),+);
    };
}

// Trading-specific logging macros
#[macro_export]
macro_rules! log_trade {
    ($service:expr, $message:expr, $($key:expr => $value:expr),+) => {
        $crate::ultra_log!($crate::LogLevel::Trade, $service, $message, $($key => $value),+);
    };
}

#[macro_export]
macro_rules! log_order {
    ($service:expr, $message:expr, $($key:expr => $value:expr),+) => {
        $crate::ultra_log!($crate::LogLevel::Order, $service, $message, $($key => $value),+);
    };
}

#[macro_export]
macro_rules! log_market_data {
    ($service:expr, $message:expr, $($key:expr => $value:expr),+) => {
        $crate::ultra_log!($crate::LogLevel::MarketData, $service, $message, $($key => $value),+);
    };
}

unsafe impl Send for UltraLogger {}
unsafe impl Sync for UltraLogger {}

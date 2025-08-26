//! Simplified Ultra-Logger for testing

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogValue {
    String(String),
    Number(f64),
    Bool(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub service: String,
    pub message: String,
    pub fields: HashMap<String, LogValue>,
}

impl LogEntry {
    pub fn new(level: LogLevel, service: String, message: String) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            service,
            message,
            fields: HashMap::new(),
        }
    }
    
    pub fn with_field(mut self, key: String, value: LogValue) -> Self {
        self.fields.insert(key, value);
        self
    }
}

#[derive(Debug)]
pub struct UltraLogger {
    service: String,
}

impl UltraLogger {
    pub fn new(service: String) -> Self {
        Self { service }
    }
    
    pub async fn log(&self, level: LogLevel, message: String) -> Result<()> {
        let entry = LogEntry::new(level, self.service.clone(), message);
        // Simple stdout logging for testing
        println!("{}", serde_json::to_string(&entry)?);
        Ok(())
    }
    
    pub async fn debug(&self, message: String) -> Result<()> {
        self.log(LogLevel::Debug, message).await
    }
    
    pub async fn info(&self, message: String) -> Result<()> {
        self.log(LogLevel::Info, message).await
    }
    
    pub async fn warn(&self, message: String) -> Result<()> {
        self.log(LogLevel::Warn, message).await
    }
    
    pub async fn error(&self, message: String) -> Result<()> {
        self.log(LogLevel::Error, message).await
    }
    
    pub async fn flush(&self) -> Result<()> {
        // No buffering in simple version
        Ok(())
    }
    
    pub async fn shutdown(&self) -> Result<()> {
        self.flush().await
    }
}

impl Default for UltraLogger {
    fn default() -> Self {
        Self::new("default".to_string())
    }
}

// Re-exports for compatibility with tests
pub use LogEntry as Entry;
pub use UltraLogger as Logger;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_logging() {
        let logger = UltraLogger::new("test-service".to_string());
        
        let result = logger.info("Test info message".to_string()).await;
        assert!(result.is_ok(), "Info logging should succeed");
        
        let result = logger.debug("Test debug message".to_string()).await;
        assert!(result.is_ok(), "Debug logging should succeed");
    }

    #[tokio::test]
    async fn test_logger_lifecycle() {
        let logger = UltraLogger::new("lifecycle-test".to_string());
        
        let result = logger.flush().await;
        assert!(result.is_ok(), "Flush should succeed");
        
        let result = logger.shutdown().await;
        assert!(result.is_ok(), "Shutdown should succeed");
    }

    #[test]
    fn test_default_logger() {
        let logger = UltraLogger::default();
        assert_eq!(logger.service, "default");
    }

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(
            LogLevel::Info, 
            "test-service".to_string(), 
            "Test message".to_string()
        );
        
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.service, "test-service");
        assert_eq!(entry.message, "Test message");
        assert!(entry.fields.is_empty());
    }

    #[test]
    fn test_log_entry_with_fields() {
        let entry = LogEntry::new(
            LogLevel::Warn, 
            "test".to_string(), 
            "Test".to_string()
        )
        .with_field("key1".to_string(), LogValue::String("value1".to_string()))
        .with_field("key2".to_string(), LogValue::Number(42.0));
        
        assert_eq!(entry.fields.len(), 2);
        assert!(entry.fields.contains_key("key1"));
        assert!(entry.fields.contains_key("key2"));
    }
}

//! Simplified Ultra-Logger for testing

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

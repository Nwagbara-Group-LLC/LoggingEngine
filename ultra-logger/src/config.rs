//! Simple configuration for ultra-logger

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main logger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggerConfig {
    /// Log level filter (debug, info, warn, error)
    pub level: String,
    
    /// Transport configuration
    pub transport: TransportConfig,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            transport: TransportConfig::default(),
        }
    }
}

/// Transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    /// Transport type: "stdout", "file", "elasticsearch"
    pub transport_type: String,
    
    /// Connection settings
    pub connection: ConnectionConfig,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            transport_type: "stdout".to_string(),
            connection: ConnectionConfig::default(),
        }
    }
}

/// Connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// Host/endpoint
    pub host: String,
    
    /// Port
    pub port: u16,
    
    /// Username (optional)
    pub username: Option<String>,
    
    /// Password (optional)
    pub password: Option<String>,
    
    /// Additional options
    pub options: HashMap<String, String>,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 9200,
            username: None,
            password: None,
            options: HashMap::new(),
        }
    }
}

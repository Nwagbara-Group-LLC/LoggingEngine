//! Centralized Configuration Module for LoggingEngine
//!
//! This module consolidates all configuration structs and provides a unified
//! interface for configuration management across the entire LoggingEngine project.
//! All values are sourced from environment variables and Kubernetes ConfigMaps.

pub mod logging_engine;
pub mod benchmark;
pub mod ultra_logger;
pub mod aggregator;
pub mod metrics;

use std::env;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

// Re-export all configuration structs for easy access
pub use logging_engine::*;
pub use benchmark::*;
pub use ultra_logger::*;
pub use aggregator::*;
pub use metrics::*;

/// Environment types for configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Environment {
    Production,
    Staging,
    Testing,
    Development,
}

impl Environment {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "prod" | "production" => Environment::Production,
            "stage" | "staging" => Environment::Staging,
            "test" | "testing" => Environment::Testing,
            "dev" | "development" | _ => Environment::Development,
        }
    }
}

/// Log levels for the entire system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warn" | "warning" => LogLevel::Warn,
            "error" => LogLevel::Error,
            _ => LogLevel::Info,
        }
    }
}

/// Transport types for log delivery
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Transport {
    Redis,
    File,
    Console,
    Network,
}

/// Unified configuration loader trait
pub trait ConfigLoader {
    /// Load configuration from environment variables
    fn from_env() -> Result<Self>
    where
        Self: Sized;
        
    /// Validate configuration values
    fn validate(&self) -> Result<()>;
    
    /// Get environment-specific defaults
    fn get_defaults(env: &Environment) -> Self
    where
        Self: Sized;
}

/// Helper function to parse environment variable with fallback
pub fn env_var_or_default<T>(key: &str, default: T) -> T
where
    T: std::str::FromStr,
{
    env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

/// Helper function to parse duration from milliseconds
pub fn duration_from_millis_env(key: &str, default_ms: u64) -> Duration {
    Duration::from_millis(env_var_or_default(key, default_ms))
}

/// Helper function to parse duration from seconds
pub fn duration_from_secs_env(key: &str, default_secs: u64) -> Duration {
    Duration::from_secs(env_var_or_default(key, default_secs))
}

/// Helper function to get environment string with fallback
pub fn env_string_or_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Helper function to parse boolean from environment
pub fn env_bool_or_default(key: &str, default: bool) -> bool {
    env::var(key)
        .ok()
        .and_then(|s| match s.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Some(true),
            "false" | "0" | "no" | "off" => Some(false),
            _ => None,
        })
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_from_str() {
        assert_eq!(Environment::from_str("production"), Environment::Production);
        assert_eq!(Environment::from_str("staging"), Environment::Staging);
        assert_eq!(Environment::from_str("testing"), Environment::Testing);
        assert_eq!(Environment::from_str("development"), Environment::Development);
        assert_eq!(Environment::from_str("invalid"), Environment::Development);
    }

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("debug"), LogLevel::Debug);
        assert_eq!(LogLevel::from_str("info"), LogLevel::Info);
        assert_eq!(LogLevel::from_str("warn"), LogLevel::Warn);
        assert_eq!(LogLevel::from_str("error"), LogLevel::Error);
        assert_eq!(LogLevel::from_str("invalid"), LogLevel::Info);
    }

    #[test]
    fn test_env_var_helpers() {
        // Test with actual environment variable
        env::set_var("TEST_VAR", "42");
        assert_eq!(env_var_or_default("TEST_VAR", 0u64), 42);
        
        // Test with default
        assert_eq!(env_var_or_default("NONEXISTENT_VAR", 100u64), 100);
        
        // Clean up
        env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_env_bool_helper() {
        env::set_var("TEST_BOOL_TRUE", "true");
        env::set_var("TEST_BOOL_FALSE", "false");
        env::set_var("TEST_BOOL_INVALID", "maybe");
        
        assert_eq!(env_bool_or_default("TEST_BOOL_TRUE", false), true);
        assert_eq!(env_bool_or_default("TEST_BOOL_FALSE", true), false);
        assert_eq!(env_bool_or_default("TEST_BOOL_INVALID", false), false);
        assert_eq!(env_bool_or_default("NONEXISTENT_BOOL", true), true);
        
        // Clean up
        env::remove_var("TEST_BOOL_TRUE");
        env::remove_var("TEST_BOOL_FALSE");
        env::remove_var("TEST_BOOL_INVALID");
    }
}

use ultra_logger::{UltraLogger, LogLevel, LogValue};
use tokio;

#[tokio::test]
async fn test_ultra_logger_basic_functionality() {
    let logger = UltraLogger::new("test-service".to_string());
    
    // Test basic logging
    let result = logger.info("Test info message".to_string()).await;
    assert!(result.is_ok(), "Info logging should succeed");
    
    let result = logger.debug("Test debug message".to_string()).await;
    assert!(result.is_ok(), "Debug logging should succeed");
    
    let result = logger.warn("Test warn message".to_string()).await;
    assert!(result.is_ok(), "Warn logging should succeed");
    
    let result = logger.error("Test error message".to_string()).await;
    assert!(result.is_ok(), "Error logging should succeed");
}

#[tokio::test]
async fn test_ultra_logger_lifecycle() {
    let logger = UltraLogger::new("lifecycle-test".to_string());
    
    // Test flush
    let result = logger.flush().await;
    assert!(result.is_ok(), "Flush should succeed");
    
    // Test shutdown
    let result = logger.shutdown().await;
    assert!(result.is_ok(), "Shutdown should succeed");
}

#[test]
fn test_default_logger() {
    let logger = UltraLogger::default();
    // Should create without error
    assert_eq!(logger.service, "default");
}

/// Test fixtures for the logging engine test suite
use serde_json::json;
use std::collections::HashMap;

/// Sample trading order data for testing
pub struct OrderFixtures;

impl OrderFixtures {
    pub fn sample_buy_order() -> serde_json::Value {
        json!({
            "order_id": "ORD_BUY_12345",
            "symbol": "BTCUSD",
            "side": "BUY",
            "order_type": "LIMIT",
            "quantity": "0.5",
            "price": "45000.50",
            "timestamp": "2025-08-26T10:30:00.123456Z",
            "client_id": "CLIENT_001",
            "portfolio": "MAIN"
        })
    }

    pub fn sample_sell_order() -> serde_json::Value {
        json!({
            "order_id": "ORD_SELL_67890",
            "symbol": "ETHUSD",
            "side": "SELL",
            "order_type": "MARKET",
            "quantity": "2.0",
            "timestamp": "2025-08-26T10:31:15.789012Z",
            "client_id": "CLIENT_002",
            "portfolio": "HEDGE"
        })
    }

    pub fn sample_order_execution() -> serde_json::Value {
        json!({
            "execution_id": "EXEC_12345",
            "order_id": "ORD_BUY_12345",
            "symbol": "BTCUSD",
            "executed_quantity": "0.5",
            "executed_price": "45000.25",
            "execution_time": "2025-08-26T10:30:01.234567Z",
            "exchange": "binance",
            "commission": "0.025",
            "liquidity": "MAKER"
        })
    }

    pub fn generate_orders(count: usize) -> Vec<serde_json::Value> {
        let symbols = vec!["BTCUSD", "ETHUSD", "ADAUSD", "DOTUSD", "LINKUSD"];
        let sides = vec!["BUY", "SELL"];
        let order_types = vec!["LIMIT", "MARKET", "STOP_LOSS"];
        
        (0..count)
            .map(|i| {
                let symbol = &symbols[i % symbols.len()];
                let side = &sides[i % sides.len()];
                let order_type = &order_types[i % order_types.len()];
                
                json!({
                    "order_id": format!("ORD_GEN_{:06}", i),
                    "symbol": symbol,
                    "side": side,
                    "order_type": order_type,
                    "quantity": format!("{:.3}", (i as f64 + 1.0) * 0.1),
                    "price": format!("{:.2}", 40000.0 + (i as f64 * 100.0)),
                    "timestamp": format!("2025-08-26T10:{:02}:{:02}.{:06}Z", 
                                       30 + (i / 60), i % 60, i * 1000),
                    "client_id": format!("CLIENT_{:03}", (i % 10) + 1),
                    "portfolio": if i % 3 == 0 { "MAIN" } else { "HEDGE" }
                })
            })
            .collect()
    }
}

/// Sample market data fixtures
pub struct MarketDataFixtures;

impl MarketDataFixtures {
    pub fn sample_price_update() -> serde_json::Value {
        json!({
            "symbol": "BTCUSD",
            "bid": "44999.75",
            "ask": "45000.25",
            "timestamp": "2025-08-26T10:30:00.123456Z",
            "volume_24h": "1234567.89",
            "change_24h": "+2.35%"
        })
    }

    pub fn sample_trade() -> serde_json::Value {
        json!({
            "trade_id": "TRD_12345",
            "symbol": "BTCUSD",
            "price": "45000.00",
            "quantity": "0.1",
            "timestamp": "2025-08-26T10:30:00.123456Z",
            "side": "BUY",
            "exchange": "binance"
        })
    }

    pub fn sample_orderbook_snapshot() -> serde_json::Value {
        json!({
            "symbol": "BTCUSD",
            "timestamp": "2025-08-26T10:30:00.123456Z",
            "sequence": 123456789,
            "bids": [
                ["44999.50", "0.5"],
                ["44999.00", "1.2"],
                ["44998.50", "0.8"]
            ],
            "asks": [
                ["45000.50", "0.3"],
                ["45001.00", "0.9"],
                ["45001.50", "1.1"]
            ]
        })
    }

    pub fn generate_price_stream(count: usize, symbol: &str, base_price: f64) -> Vec<serde_json::Value> {
        (0..count)
            .map(|i| {
                let price_variation = (i as f64 * 0.01) - (count as f64 * 0.005);
                let price = base_price + price_variation;
                let bid = price - 0.25;
                let ask = price + 0.25;
                
                json!({
                    "symbol": symbol,
                    "bid": format!("{:.2}", bid),
                    "ask": format!("{:.2}", ask),
                    "timestamp": format!("2025-08-26T10:30:{:02}.{:06}Z", 
                                       i % 60, i * 1000),
                    "volume_24h": format!("{:.2}", 1000000.0 + (i as f64 * 100.0)),
                    "change_24h": format!("{:+.2}%", price_variation / base_price * 100.0)
                })
            })
            .collect()
    }
}

/// Risk management test fixtures
pub struct RiskFixtures;

impl RiskFixtures {
    pub fn sample_position() -> serde_json::Value {
        json!({
            "position_id": "POS_12345",
            "symbol": "BTCUSD",
            "quantity": "1.5",
            "avg_price": "44750.00",
            "unrealized_pnl": "375.75",
            "realized_pnl": "123.45",
            "timestamp": "2025-08-26T10:30:00.123456Z"
        })
    }

    pub fn sample_risk_metrics() -> serde_json::Value {
        json!({
            "portfolio": "MAIN",
            "total_equity": "100000.00",
            "available_margin": "75000.00",
            "used_margin": "25000.00",
            "margin_ratio": "0.25",
            "var_1d": "2500.00",
            "max_drawdown": "5000.00",
            "timestamp": "2025-08-26T10:30:00.123456Z"
        })
    }

    pub fn sample_risk_violation() -> serde_json::Value {
        json!({
            "violation_id": "RISK_VIOL_001",
            "rule": "MAX_POSITION_SIZE",
            "severity": "HIGH",
            "message": "Position size exceeds 10% of portfolio",
            "current_value": "12000.00",
            "threshold": "10000.00",
            "timestamp": "2025-08-26T10:30:00.123456Z"
        })
    }
}

/// System performance test fixtures
pub struct PerformanceFixtures;

impl PerformanceFixtures {
    pub fn sample_latency_measurement() -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        metrics.insert("order_received_to_risk_check_us".to_string(), 1.5);
        metrics.insert("risk_check_to_routing_us".to_string(), 0.8);
        metrics.insert("routing_to_exchange_us".to_string(), 2.3);
        metrics.insert("exchange_ack_latency_us".to_string(), 15.7);
        metrics.insert("total_order_latency_us".to_string(), 20.3);
        metrics
    }

    pub fn sample_throughput_metrics() -> HashMap<String, u64> {
        let mut metrics = HashMap::new();
        metrics.insert("orders_per_second".to_string(), 12500);
        metrics.insert("market_data_updates_per_second".to_string(), 50000);
        metrics.insert("risk_checks_per_second".to_string(), 15000);
        metrics.insert("log_messages_per_second".to_string(), 100000);
        metrics
    }

    pub fn sample_system_metrics() -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        metrics.insert("cpu_usage_percent".to_string(), 15.5);
        metrics.insert("memory_usage_percent".to_string(), 45.2);
        metrics.insert("network_rx_mbps".to_string(), 125.8);
        metrics.insert("network_tx_mbps".to_string(), 89.3);
        metrics.insert("disk_io_iops".to_string(), 2500.0);
        metrics
    }
}

/// Error and exception test fixtures
pub struct ErrorFixtures;

impl ErrorFixtures {
    pub fn sample_system_error() -> serde_json::Value {
        json!({
            "error_id": "ERR_SYS_001",
            "error_type": "CONNECTION_ERROR",
            "component": "exchange_connector",
            "severity": "HIGH",
            "message": "Failed to establish connection to exchange API",
            "error_code": "CONN_TIMEOUT",
            "timestamp": "2025-08-26T10:30:00.123456Z",
            "stack_trace": "at ExchangeConnector.connect() line 45\\nat OrderRouter.route() line 123"
        })
    }

    pub fn sample_validation_error() -> serde_json::Value {
        json!({
            "error_id": "ERR_VAL_001",
            "error_type": "VALIDATION_ERROR",
            "component": "order_validator",
            "severity": "MEDIUM",
            "message": "Order quantity exceeds maximum allowed",
            "error_code": "QUANTITY_EXCEEDED",
            "order_id": "ORD_12345",
            "requested_quantity": "100.0",
            "max_allowed": "10.0",
            "timestamp": "2025-08-26T10:30:00.123456Z"
        })
    }

    pub fn sample_timeout_error() -> serde_json::Value {
        json!({
            "error_id": "ERR_TIMEOUT_001",
            "error_type": "TIMEOUT_ERROR",
            "component": "risk_engine",
            "severity": "HIGH",
            "message": "Risk check timed out after 5ms",
            "error_code": "RISK_TIMEOUT",
            "timeout_ms": 5,
            "timestamp": "2025-08-26T10:30:00.123456Z"
        })
    }
}

/// Log message fixtures
pub struct LogFixtures;

impl LogFixtures {
    pub fn sample_info_log() -> (&'static str, &'static str, &'static str) {
        ("INFO", "trading_engine", "Order processing completed successfully")
    }

    pub fn sample_debug_log() -> (&'static str, &'static str, &'static str) {
        ("DEBUG", "market_data", "Received 1000 price updates in batch")
    }

    pub fn sample_warn_log() -> (&'static str, &'static str, &'static str) {
        ("WARN", "risk_engine", "Position approaching risk limit")
    }

    pub fn sample_error_log() -> (&'static str, &'static str, &'static str) {
        ("ERROR", "exchange_connector", "Failed to submit order to exchange")
    }

    pub fn generate_log_burst(count: usize) -> Vec<(String, String, String)> {
        let levels = vec!["DEBUG", "INFO", "WARN", "ERROR"];
        let modules = vec!["trading", "risk", "market_data", "execution", "portfolio"];
        
        (0..count)
            .map(|i| {
                let level = levels[i % levels.len()].to_string();
                let module = modules[i % modules.len()].to_string();
                let message = format!("Test log message {} from burst", i);
                (level, module, message)
            })
            .collect()
    }

    pub fn trading_lifecycle_logs() -> Vec<(String, String, String)> {
        vec![
            ("INFO".to_string(), "order_gateway".to_string(), "Order received from client".to_string()),
            ("DEBUG".to_string(), "order_validator".to_string(), "Order validation started".to_string()),
            ("INFO".to_string(), "order_validator".to_string(), "Order validation passed".to_string()),
            ("DEBUG".to_string(), "risk_engine".to_string(), "Risk check initiated".to_string()),
            ("INFO".to_string(), "risk_engine".to_string(), "Risk check passed".to_string()),
            ("DEBUG".to_string(), "order_router".to_string(), "Routing order to exchange".to_string()),
            ("INFO".to_string(), "exchange_connector".to_string(), "Order sent to exchange".to_string()),
            ("INFO".to_string(), "exchange_connector".to_string(), "Order acknowledgment received".to_string()),
            ("INFO".to_string(), "execution_handler".to_string(), "Order filled notification".to_string()),
            ("INFO".to_string(), "portfolio_manager".to_string(), "Position updated".to_string()),
        ]
    }
}

/// Configuration fixtures for testing
pub struct ConfigFixtures;

impl ConfigFixtures {
    pub fn ultra_low_latency_config() -> serde_json::Value {
        json!({
            "buffer_size": 1048576,
            "batch_size": 100,
            "flush_interval_ns": 100,
            "compression": "none",
            "transport": "memory",
            "enable_structured_logging": true,
            "log_level": "INFO"
        })
    }

    pub fn high_throughput_config() -> serde_json::Value {
        json!({
            "buffer_size": 10485760,
            "batch_size": 10000,
            "flush_interval_ms": 10,
            "compression": "lz4",
            "transport": "redis",
            "enable_structured_logging": false,
            "log_level": "WARN"
        })
    }

    pub fn production_config() -> serde_json::Value {
        json!({
            "buffer_size": 5242880,
            "batch_size": 1000,
            "flush_interval_ms": 5,
            "compression": "zstd",
            "transport": "file",
            "enable_structured_logging": true,
            "log_level": "INFO",
            "file_rotation": true,
            "max_file_size": "100MB",
            "retention_days": 30
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_fixtures() {
        let order = OrderFixtures::sample_buy_order();
        assert_eq!(order["symbol"], "BTCUSD");
        assert_eq!(order["side"], "BUY");
    }

    #[test]
    fn test_generated_orders() {
        let orders = OrderFixtures::generate_orders(5);
        assert_eq!(orders.len(), 5);
        
        for (i, order) in orders.iter().enumerate() {
            assert_eq!(order["order_id"], format!("ORD_GEN_{:06}", i));
        }
    }

    #[test]
    fn test_market_data_fixtures() {
        let price_update = MarketDataFixtures::sample_price_update();
        assert_eq!(price_update["symbol"], "BTCUSD");
        assert!(price_update["bid"].as_str().unwrap().parse::<f64>().is_ok());
    }

    #[test]
    fn test_log_fixtures() {
        let lifecycle_logs = LogFixtures::trading_lifecycle_logs();
        assert_eq!(lifecycle_logs.len(), 10);
        assert_eq!(lifecycle_logs[0].0, "INFO");
    }
}

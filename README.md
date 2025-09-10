# LoggingEngine

> Ultra-high-performance logging solution for trading systems and high-frequency applications

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Cargo](https://img.shields.io/badge/cargo-workspace-green.svg)](Cargo.toml)

## Overview

LoggingEngine is a streamlined, ultra-fast logging solution specifically designed for high-frequency trading systems where microsecond latency and massive throughput are critical. Built in Rust with a focus on zero-allocation logging and lock-free operations.

### Key Features

- **🚀 Ultra-Fast Performance**: Sub-microsecond logging latency
- **📈 High Throughput**: Handle 100K+ messages per second
- **🔄 Async-First**: Built on Tokio with non-blocking operations
- **🎯 Trading-Optimized**: Designed for financial market data and order flows
- **📤 Multiple Transports**: stdout, file, and Elasticsearch support
- **⚙️ Simple Configuration**: YAML-based configuration system
- **🧵 Lock-Free**: Flume channels for maximum concurrency
- **📊 Structured Logging**: JSON serialization with serde

## Architecture

The LoggingEngine follows a simple, focused architecture:

```
LoggingEngine/
├── ultra-logger/           # Core logging library
│   ├── src/
│   │   ├── lib.rs         # Main UltraLogger implementation
│   │   ├── config.rs      # Configuration types
│   │   ├── transport.rs   # Transport layer
│   │   └── error.rs       # Error handling
│   └── Cargo.toml
├── examples/              # Usage examples
├── src/                   # Workspace-level exports
└── Cargo.toml            # Workspace configuration
```

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
logging-engine = { path = "../LoggingEngine" }
# Or directly use the core library
ultra-logger = { path = "../LoggingEngine/ultra-logger" }
```

### Basic Usage

```rust
use ultra_logger::{UltraLogger, LoggerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create with default configuration (stdout transport)
    let logger = UltraLogger::new(LoggerConfig::default()).await?;
    
    // Log trading events
    logger.info("Order received", &[
        ("symbol", "BTCUSD"),
        ("side", "BUY"),
        ("quantity", "1.5"),
        ("price", "45000.00")
    ]).await?;
    
    logger.warn("Market volatility detected", &[
        ("symbol", "ETHUSD"),
        ("volatility", "0.15")
    ]).await?;
    
    Ok(())
}
```

### Configuration

Create a `config.yaml`:

```yaml
level: "info"
transport:
  transport_type: "elasticsearch"
  connection:
    host: "localhost"
    port: 9200
    username: "elastic"
    password: "password"
    options:
      index_pattern: "trading-logs-%Y.%m.%d"
      bulk_size: "1000"
```

Load and use:

```rust
use ultra_logger::{UltraLogger, LoggerConfig};

let config = LoggerConfig::from_file("config.yaml")?;
let logger = UltraLogger::new(config).await?;
```

## Performance Characteristics

The LoggingEngine is optimized for extreme performance:

| Metric | Target | Typical |
|--------|---------|---------|
| Latency | < 1μs | ~0.3μs |
| Throughput | 100K+/sec | 250K/sec |
| Memory | Minimal | ~10MB |
| CPU Impact | < 1% | ~0.3% |

### Performance Features

- **Zero-Copy Operations**: Minimal allocations in hot paths
- **Batch Processing**: Efficient bulk operations for high throughput
- **Lock-Free Channels**: Flume-based message passing
- **SIMD Optimization**: Fast JSON serialization with simd-json
- **Custom Allocator**: Optional mimalloc for reduced memory fragmentation

## Transport Options

### 1. Stdout Transport
```yaml
transport:
  transport_type: "stdout"
```

Perfect for development and debugging.

### 2. File Transport
```yaml
transport:
  transport_type: "file"
  connection:
    host: "/var/log/trading"
    options:
      rotation: "daily"
      max_size: "100MB"
```

High-performance file logging with rotation.

### 3. Elasticsearch Transport
```yaml
transport:
  transport_type: "elasticsearch"
  connection:
    host: "elasticsearch.example.com"
    port: 9200
    username: "logger"
    password: "secret"
    options:
      index_pattern: "trading-logs-%Y.%m.%d"
      bulk_size: "1000"
      flush_interval: "1s"
```

Real-time log aggregation and search.

## Trading System Integration

### Order Flow Logging
```rust
// Log order lifecycle events
logger.info("order_received", &[
    ("order_id", order.id),
    ("symbol", order.symbol),
    ("side", order.side),
    ("quantity", &order.quantity.to_string()),
    ("price", &order.price.to_string()),
    ("timestamp", &order.timestamp.to_rfc3339())
]).await?;

logger.info("order_filled", &[
    ("order_id", order.id),
    ("fill_price", &fill.price.to_string()),
    ("fill_quantity", &fill.quantity.to_string()),
    ("execution_time", &execution_duration.as_micros().to_string())
]).await?;
```

### Market Data Logging
```rust
// Log market events
logger.debug("tick_received", &[
    ("symbol", tick.symbol),
    ("bid", &tick.bid.to_string()),
    ("ask", &tick.ask.to_string()),
    ("volume", &tick.volume.to_string()),
    ("latency_us", &processing_time.as_micros().to_string())
]).await?;
```

### Risk Management Events
```rust
logger.warn("risk_limit_approached", &[
    ("account", account.id),
    ("metric", "position_size"),
    ("current", &position.size.to_string()),
    ("limit", &risk_limit.to_string()),
    ("utilization_pct", &utilization.to_string())
]).await?;
```

## Dependencies

The LoggingEngine maintains a minimal dependency footprint for maximum performance:

### Core Dependencies
- `tokio` - Async runtime
- `serde` - Serialization framework
- `serde_json` - JSON serialization
- `chrono` - Date/time handling
- `flume` - High-performance channels
- `async-trait` - Async trait support
- `thiserror` - Error handling

### Performance Dependencies
- `simd-json` - SIMD-accelerated JSON parsing
- `ahash` - Fast hashing algorithm
- `mimalloc` - Memory allocator optimization

## Development

### Building
```bash
# Build the workspace
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings
```

### Examples
```bash
# Run basic usage example
cargo run --example basic_usage

# Run with specific transport
RUST_LOG=debug cargo run --example elasticsearch_transport
```

### Testing
```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --tests

# Performance tests
cargo test --bench performance_suite
```

## Monitoring and Observability

### Built-in Metrics
The logger exposes internal metrics for monitoring:

- Messages per second
- Average latency
- Queue depth
- Transport errors
- Memory usage

### Health Checks
```rust
// Check logger health
let health = logger.health_check().await?;
if !health.is_healthy {
    eprintln!("Logger health issues: {:?}", health.issues);
}
```

### Debugging
```rust
// Enable debug mode
let config = LoggerConfig {
    level: "debug".to_string(),
    ..Default::default()
};

// Check internal state
logger.dump_stats().await?;
```

## Production Deployment

### Configuration Best Practices

1. **Set appropriate log levels**: Use `info` or `warn` in production
2. **Configure transport properly**: Use Elasticsearch for production aggregation
3. **Set resource limits**: Configure memory and CPU limits
4. **Enable monitoring**: Use built-in health checks and metrics

### Docker Deployment
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/trading-system /usr/local/bin/
COPY config.yaml /etc/logging/config.yaml
CMD ["trading-system", "--config", "/etc/logging/config.yaml"]
```

### Kubernetes Integration
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: logging-config
data:
  config.yaml: |
    level: "info"
    transport:
      transport_type: "elasticsearch"
      connection:
        host: "elasticsearch-service"
        port: 9200
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: trading-system
spec:
  replicas: 3
  selector:
    matchLabels:
      app: trading-system
  template:
    metadata:
      labels:
        app: trading-system
    spec:
      containers:
      - name: trading-system
        image: trading-system:latest
        volumeMounts:
        - name: config
          mountPath: /etc/logging
        resources:
          requests:
            memory: "50Mi"
            cpu: "10m"
          limits:
            memory: "200Mi"
            cpu: "100m"
      volumes:
      - name: config
        configMap:
          name: logging-config
```

## Troubleshooting

### Common Issues

1. **High Latency**: Check transport configuration and network connectivity
2. **Memory Usage**: Verify batch sizes and flush intervals
3. **Lost Messages**: Check queue capacity and error handling
4. **Connection Errors**: Verify transport endpoint availability

### Debug Commands
```bash
# Check logger configuration
cargo run --example config_check

# Test transport connectivity
cargo run --example transport_test

# Run performance benchmark
cargo bench transport_performance
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/new-transport`)
3. Commit your changes (`git commit -am 'Add new transport'`)
4. Push to the branch (`git push origin feature/new-transport`)
5. Create a Pull Request

### Development Guidelines

- Follow Rust best practices and idioms
- Maintain backward compatibility
- Add tests for new features
- Update documentation
- Run `cargo fmt` and `cargo clippy`

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

For questions, issues, or feature requests:

- Create an issue on GitHub
- Contact: Nwagbara Group LLC
- Documentation: See `docs/` directory for detailed guides

---

**Built for Speed. Designed for Trading. Powered by Rust.**

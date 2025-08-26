# LoggingEngine Test Suite

This directory contains a comprehensive test suite for the Ultra-Low Latency LoggingEngine, designed specifically for high-frequency trading systems.

## 📁 Test Structure

```
tests/
├── integration/           # Integration tests
│   ├── mod.rs            # Main integration test coordinator
│   ├── test_ultra_logger.rs
│   ├── test_log_aggregator.rs
│   ├── test_metrics_collector.rs
│   └── test_end_to_end.rs
├── benchmarks/           # Performance benchmarks
│   └── mod.rs           # Criterion-based benchmarks
├── e2e/                 # End-to-end tests
│   └── comprehensive_test.rs
├── fixtures/            # Test data and utilities
│   └── mod.rs          # Sample data generators
├── utils/              # Test utilities
│   └── mod.rs          # Test harness and helpers
├── property_tests.rs   # Property-based tests
└── mod.rs             # Test module coordinator
```

## 🧪 Test Categories

### 1. Unit Tests
- **Location**: Individual component `src/` directories
- **Purpose**: Test individual functions and methods
- **Run**: `cargo test --lib --bins`

### 2. Integration Tests
- **Location**: `tests/integration/`
- **Purpose**: Test component interactions and system behavior
- **Coverage**:
  - Ultra-logger functionality (latency, throughput, compression)
  - Log aggregator batch processing and filtering
  - Metrics collector performance and accuracy
  - Cross-component communication
- **Run**: `cargo test --test integration`

### 3. Property-Based Tests
- **Location**: `tests/property_tests.rs`
- **Purpose**: Test system properties with randomized inputs
- **Coverage**:
  - Arbitrary log message handling
  - Buffer configuration robustness
  - Concurrent access invariants
  - Memory management under stress
- **Run**: `cargo test --test property_tests`

### 4. Benchmarks
- **Location**: `tests/benchmarks/`
- **Purpose**: Measure and validate performance characteristics
- **Metrics**:
  - Single log operation latency (target: <1μs)
  - Throughput under load (target: >100K ops/sec)
  - Compression efficiency
  - Concurrent operation scaling
- **Run**: `cargo bench`

### 5. End-to-End Tests
- **Location**: `tests/e2e/`
- **Purpose**: Validate complete trading scenarios
- **Scenarios**:
  - Full trading pipeline simulation
  - Market data processing bursts
  - Error handling and recovery
  - Production workload simulation
- **Run**: `cargo test --test e2e`

## 🚀 Running Tests

### Quick Start
```powershell
# Run all tests
.\run_tests.ps1

# Run specific test type
.\run_tests.ps1 integration
.\run_tests.ps1 benchmarks
.\run_tests.ps1 e2e

# Run with verbose output
.\run_tests.ps1 -Verbose

# Run performance tests only
.\run_tests.ps1 performance
```

### Manual Test Execution
```bash
# Unit tests
cargo test --lib --bins

# Integration tests
cargo test --test integration

# Property-based tests  
cargo test --test property_tests

# End-to-end tests
cargo test --test e2e

# Benchmarks
cargo bench --bench ultra_logger_benchmarks

# All tests with coverage
cargo tarpaulin --out Html
```

## 📊 Performance Targets

### Ultra-Low Latency Requirements
- **Average Logging Latency**: <1 microsecond
- **P99 Logging Latency**: <5 microseconds  
- **Trading Pipeline Latency**: <50 microseconds (order-to-ack)

### High Throughput Requirements
- **Log Operations**: >100,000 ops/second
- **Market Data Processing**: >50,000 updates/second
- **Order Processing**: >10,000 orders/second

### System Scalability
- **Concurrent Threads**: Up to 16 threads
- **Memory Usage**: <100MB baseline
- **CPU Usage**: <20% under normal load

## 🔧 Test Configuration

### Environment Variables
```bash
# Skip slow tests (useful for CI)
export SKIP_SLOW_TESTS=1

# Skip Redis-dependent tests
export SKIP_REDIS_TESTS=1

# Skip Kafka-dependent tests  
export SKIP_KAFKA_TESTS=1

# Enable debug logging
export RUST_LOG=debug

# Enable backtrace on panics
export RUST_BACKTRACE=1
```

### Test Profiles
- **Development**: Fast compilation, basic optimization
- **Release**: Full optimization, production-like performance
- **Bench**: Optimized for benchmarking with debug info

## 🏗️ Test Utilities

### TestHarness
Provides coordinated startup/shutdown of all logging components:
```rust
let harness = TestHarness::new().await?;
harness.start_all().await?;

// Run your tests

harness.stop_all().await?;
```

### PerformanceMeasurer
Measures operation latency and throughput:
```rust
let measurer = PerformanceMeasurer::new();
measurer.record_operation(|| async {
    // Your operation here
}).await;

let stats = measurer.get_stats().await;
stats.assert_ultra_low_latency();
```

### LoadTestGenerator
Generates sustained load for performance testing:
```rust
let generator = LoadTestGenerator::new(10000.0, Duration::from_secs(30));
let results = generator.run_load_test(|op_id| async {
    // Your load test operation
}).await;
```

### TradingScenarioSimulator
Simulates realistic trading workflows:
```rust
let latencies = TradingScenarioSimulator::simulate_order_flow(&harness, 1000).await;
TradingScenarioSimulator::simulate_market_data_burst(&harness, 10000, "BTCUSD").await;
```

## 📋 Test Data Fixtures

### Order Fixtures
- Sample buy/sell orders
- Order execution data
- Generated order sequences

### Market Data Fixtures  
- Price updates
- Order book snapshots
- Trade data streams

### Performance Fixtures
- Latency measurements
- Throughput metrics
- System resource usage

### Error Fixtures
- System errors
- Validation failures
- Timeout scenarios

## 🎯 Test Coverage Goals

- **Unit Test Coverage**: >90%
- **Integration Coverage**: >85%
- **Critical Path Coverage**: 100%
- **Error Path Coverage**: >80%

## 🚨 Failure Thresholds

### Performance Failures
Tests fail if:
- Average latency >1μs
- P99 latency >5μs  
- Throughput <100K ops/sec
- Memory usage >200MB
- Any operation >50μs

### Reliability Failures
Tests fail if:
- Message loss detected
- Component crash/deadlock
- Resource leak detected
- Recovery time >100ms

## 🔍 Debugging Test Failures

### Common Issues
1. **Timing Issues**: Increase timeout values or add delays
2. **Resource Contention**: Run fewer concurrent tests
3. **Environment Dependencies**: Check Redis/Kafka availability
4. **Memory Pressure**: Increase available memory or reduce test load

### Debug Commands
```bash
# Run single test with detailed output
cargo test test_name -- --nocapture

# Run with debug logging
RUST_LOG=debug cargo test test_name

# Run with memory debugging
cargo test --features=debug-memory test_name

# Run benchmarks with detailed profiling  
cargo bench --bench ultra_logger_benchmarks -- --profile-time=30
```

## 📈 Continuous Integration

The test suite is integrated with GitHub Actions:
- **PR Tests**: Fast subset focusing on correctness
- **Main Branch**: Full test suite including benchmarks
- **Release**: Comprehensive validation with performance regression detection

### CI Test Matrix
- **Rust Versions**: Stable, Beta, MSRV
- **Operating Systems**: Linux, Windows, macOS  
- **Configurations**: Debug, Release, Various feature flags

## 📚 Adding New Tests

### Guidelines
1. **Name tests descriptively**: `test_ultra_logger_concurrent_access`
2. **Use appropriate category**: Unit vs Integration vs E2E
3. **Include performance assertions**: Latency and throughput checks
4. **Add error scenarios**: Test failure cases and recovery
5. **Use fixtures**: Consistent test data across tests
6. **Document complex tests**: Explain what and why

### Example Test Structure
```rust
#[tokio::test]
async fn test_new_feature() {
    // Arrange
    let harness = TestHarness::new().await.expect("Setup failed");
    harness.start_all().await.expect("Start failed");
    
    // Act
    let result = perform_operation(&harness).await;
    
    // Assert
    assert!(result.is_ok());
    assert!(result.latency < Duration::from_micros(1));
    
    // Cleanup
    harness.stop_all().await.expect("Cleanup failed");
}
```

## 🏆 Test Quality Metrics

The test suite tracks:
- **Execution Time**: Total and per-category timing
- **Flakiness**: Test stability over multiple runs  
- **Coverage**: Code and branch coverage metrics
- **Performance**: Regression tracking over time
- **Resource Usage**: Memory and CPU consumption during tests

---

**🚀 Ready for High-Frequency Trading Production Deployment!** 

This comprehensive test suite ensures the LoggingEngine meets the stringent requirements of high-frequency trading systems with ultra-low latency, high throughput, and bulletproof reliability.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ultra_logger::{UltraLogger, LogLevel};

fn bench_ultra_logger_sync(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    // Create logger once outside the benchmark loop
    let logger = rt.block_on(async {
        UltraLogger::new("benchmark".to_string())
    });
    
    c.bench_function("ultra_logger_info", |b| {
        b.iter(|| {
            rt.block_on(async {
                logger.info(black_box("Benchmark message".to_string())).await.unwrap();
            });
        });
    });
}

fn bench_log_entry_creation(c: &mut Criterion) {
    c.bench_function("log_entry_creation", |b| {
        b.iter(|| {
            ultra_logger::LogEntry::new(
                black_box(LogLevel::Info),
                black_box("service".to_string()),
                black_box("message".to_string()),
                black_box(1u64)
            )
        });
    });
}

criterion_group!(benches, bench_ultra_logger_sync, bench_log_entry_creation);
criterion_main!(benches);

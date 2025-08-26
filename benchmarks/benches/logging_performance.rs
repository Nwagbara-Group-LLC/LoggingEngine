use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ultra_logger::{UltraLogger, LogLevel};
use tokio::runtime::Runtime;

fn bench_ultra_logger_sync(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let logger = UltraLogger::new("benchmark".to_string());
    
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
                black_box("message".to_string())
            )
        });
    });
}

criterion_group!(benches, bench_ultra_logger_sync, bench_log_entry_creation);
criterion_main!(benches);

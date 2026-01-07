use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_error_creation(c: &mut Criterion) {
    c.bench_function("create_permanent_error", |b| {
        b.iter(|| {
            erract::Error::permanent(
                black_box(erract::ErrorKind::NotFound),
                black_box("test message"),
            )
        })
    });

    c.bench_function("create_temporary_error", |b| {
        b.iter(|| {
            erract::Error::temporary(
                black_box(erract::ErrorKind::Timeout),
                black_box("test message"),
            )
        })
    });

    c.bench_function("error_with_context", |b| {
        b.iter(|| {
            erract::Error::permanent(erract::ErrorKind::NotFound, "test message")
                .with_context("key", "value")
        })
    });
}

criterion_group!(benches, bench_error_creation);
criterion_main!(benches);

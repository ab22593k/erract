use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn bench_small_context(c: &mut Criterion) {
    let mut group = c.benchmark_group("small_context");

    group.bench_function("erract/1_item", |b| {
        b.iter(|| {
            let mut err = erract::Error::not_found();
            err = err.with_context(black_box("key"), black_box("val"));
            let ctx = err.context();
            black_box(ctx);
            err
        })
    });

    group.bench_function("anyhow/1_item", |b| {
        b.iter(|| {
            let mut err = anyhow::anyhow!("not found");
            err = err.context(black_box("key: val"));
            let chain: Vec<_> = err.chain().collect();
            black_box(chain);
            err
        })
    });

    group.finish();
}

fn bench_large_context(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_context");

    group.bench_function("erract/100_items", |b| {
        b.iter(|| {
            let mut err = erract::Error::not_found();
            for _ in 0..100 {
                err = err.with_context(black_box("key"), black_box("val"));
            }
            let ctx = err.context();
            black_box(ctx);
            err
        })
    });

    group.bench_function("anyhow/100_items", |b| {
        b.iter(|| {
            let mut err = anyhow::anyhow!("not found");
            for _ in 0..100 {
                err = err.context(black_box("key: val"));
            }
            let chain: Vec<_> = err.chain().collect();
            black_box(chain);
            err
        })
    });

    group.finish();
}

criterion_group!(benches, bench_small_context, bench_large_context);
criterion_main!(benches);

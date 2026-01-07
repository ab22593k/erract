//! Comprehensive benchmarks comparing erract vs anyhow.
//!
//! Run with: `cargo bench`
//! Run specific group: `cargo bench -- error_creation`
//!
//! For anyhow backtrace comparison:
//! ```bash
//! RUST_BACKTRACE=1 cargo bench
//! ```

use criterion::{Criterion, black_box, criterion_group, criterion_main};

// ============================================================================
// GROUP 1: Error Creation
// ============================================================================

fn bench_error_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_creation");

    // --- Basic error creation ---

    group.bench_function("erract/basic", |b| {
        b.iter(|| {
            erract::Error::permanent(
                black_box(erract::ErrorKind::NotFound),
                black_box("resource not found"),
            )
        })
    });

    group.bench_function("anyhow/basic", |b| {
        b.iter(|| anyhow::anyhow!(black_box("resource not found")))
    });

    // --- Error with format string ---

    group.bench_function("erract/formatted", |b| {
        let id = 12345u32;
        b.iter(|| {
            erract::Error::permanent(
                erract::ErrorKind::NotFound,
                format!("user {} not found", black_box(id)),
            )
        })
    });

    group.bench_function("anyhow/formatted", |b| {
        let id = 12345u32;
        b.iter(|| anyhow::anyhow!("user {} not found", black_box(id)))
    });

    // --- Temporary vs permanent (erract only) ---

    group.bench_function("erract/temporary", |b| {
        b.iter(|| {
            erract::Error::temporary(
                black_box(erract::ErrorKind::Timeout),
                black_box("operation timed out"),
            )
        })
    });

    group.finish();
}

// ============================================================================
// GROUP 2: Context Addition
// ============================================================================

fn bench_context(c: &mut Criterion) {
    let mut group = c.benchmark_group("context_addition");

    // --- Single context ---

    group.bench_function("erract/single", |b| {
        b.iter(|| {
            erract::Error::permanent(erract::ErrorKind::NotFound, "not found")
                .with_context(black_box("user_id"), black_box("12345"))
        })
    });

    group.bench_function("anyhow/single", |b| {
        b.iter(|| anyhow::anyhow!("not found").context(black_box("user_id: 12345")))
    });

    // --- Triple context ---

    group.bench_function("erract/triple", |b| {
        b.iter(|| {
            erract::Error::permanent(erract::ErrorKind::NotFound, "not found")
                .with_context(black_box("user_id"), black_box("12345"))
                .with_context(black_box("operation"), black_box("lookup"))
                .with_context(black_box("timestamp"), black_box("2024-01-01"))
        })
    });

    group.bench_function("anyhow/triple", |b| {
        b.iter(|| {
            anyhow::anyhow!("not found")
                .context(black_box("user_id: 12345"))
                .context(black_box("operation: lookup"))
                .context(black_box("timestamp: 2024-01-01"))
        })
    });

    // --- With operation (erract only) ---

    group.bench_function("erract/with_operation", |b| {
        b.iter(|| {
            erract::Error::permanent(erract::ErrorKind::NotFound, "not found")
                .with_operation(black_box("fetch_user"))
                .with_context(black_box("user_id"), black_box("12345"))
        })
    });

    group.finish();
}

// ============================================================================
// GROUP 3: Error Propagation
// ============================================================================

mod erract_propagation {
    use erract::prelude::*;

    #[inline(never)]
    pub fn depth_1() -> erract::Result<()> {
        Err(Error::permanent(ErrorKind::NotFound, "inner error").raise())
    }

    #[inline(never)]
    pub fn depth_2() -> erract::Result<()> {
        depth_1().or_raise(|| Error::temporary(ErrorKind::Unexpected, "level 2"))?;
        Ok(())
    }

    #[inline(never)]
    pub fn depth_3() -> erract::Result<()> {
        depth_2().or_raise(|| Error::temporary(ErrorKind::Unexpected, "level 3"))?;
        Ok(())
    }

    #[inline(never)]
    pub fn depth_4() -> erract::Result<()> {
        depth_3().or_raise(|| Error::temporary(ErrorKind::Unexpected, "level 4"))?;
        Ok(())
    }

    #[inline(never)]
    pub fn depth_5() -> erract::Result<()> {
        depth_4().or_raise(|| Error::permanent(ErrorKind::Unexpected, "level 5"))?;
        Ok(())
    }
}

mod anyhow_propagation {
    use anyhow::{Context, Result, anyhow};

    #[inline(never)]
    pub fn depth_1() -> Result<()> {
        Err(anyhow!("inner error"))
    }

    #[inline(never)]
    pub fn depth_2() -> Result<()> {
        depth_1().context("level 2")?;
        Ok(())
    }

    #[inline(never)]
    pub fn depth_3() -> Result<()> {
        depth_2().context("level 3")?;
        Ok(())
    }

    #[inline(never)]
    pub fn depth_4() -> Result<()> {
        depth_3().context("level 4")?;
        Ok(())
    }

    #[inline(never)]
    pub fn depth_5() -> Result<()> {
        depth_4().context("level 5")?;
        Ok(())
    }
}

fn bench_propagation(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_propagation");

    // --- Depth 1 (single error) ---

    group.bench_function("erract/depth_1", |b| {
        b.iter(|| {
            let _ = black_box(erract_propagation::depth_1());
        })
    });

    group.bench_function("anyhow/depth_1", |b| {
        b.iter(|| {
            let _ = black_box(anyhow_propagation::depth_1());
        })
    });

    // --- Depth 3 ---

    group.bench_function("erract/depth_3", |b| {
        b.iter(|| {
            let _ = black_box(erract_propagation::depth_3());
        })
    });

    group.bench_function("anyhow/depth_3", |b| {
        b.iter(|| {
            let _ = black_box(anyhow_propagation::depth_3());
        })
    });

    // --- Depth 5 ---

    group.bench_function("erract/depth_5", |b| {
        b.iter(|| {
            let _ = black_box(erract_propagation::depth_5());
        })
    });

    group.bench_function("anyhow/depth_5", |b| {
        b.iter(|| {
            let _ = black_box(anyhow_propagation::depth_5());
        })
    });

    group.finish();
}

// ============================================================================
// GROUP 4: Error Display / Formatting
// ============================================================================

fn bench_display(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_display");

    // Pre-create errors for display benchmarks
    let erract_error = erract::Error::permanent(erract::ErrorKind::NotFound, "user not found")
        .with_context("user_id", "12345")
        .with_context("operation", "lookup")
        .with_operation("user_service");

    let anyhow_error = anyhow::anyhow!("user not found")
        .context("operation: lookup")
        .context("user_id: 12345");

    // --- to_string ---

    group.bench_function("erract/to_string", |b| {
        b.iter(|| black_box(erract_error.to_string()))
    });

    group.bench_function("anyhow/to_string", |b| {
        b.iter(|| black_box(anyhow_error.to_string()))
    });

    // --- Debug format ---

    group.bench_function("erract/debug", |b| {
        b.iter(|| black_box(format!("{erract_error:?}")))
    });

    group.bench_function("anyhow/debug", |b| {
        b.iter(|| black_box(format!("{anyhow_error:?}")))
    });

    // --- erract-specific formats ---

    group.bench_function("erract/to_json", |b| {
        b.iter(|| black_box(erract_error.to_json()))
    });

    group.bench_function("erract/to_machine_string", |b| {
        b.iter(|| black_box(erract_error.to_machine_string()))
    });

    group.finish();
}

// ============================================================================
// GROUP 5: Error Inspection / Querying
// ============================================================================

fn bench_inspection(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_inspection");

    // Pre-create errors
    let erract_error =
        erract::Error::temporary(erract::ErrorKind::Timeout, "timeout").with_context("ms", "5000");

    // --- Retryable check (erract) ---

    group.bench_function("erract/is_retryable", |b| {
        b.iter(|| black_box(erract_error.is_retryable()))
    });

    group.bench_function("erract/is_permanent", |b| {
        b.iter(|| black_box(erract_error.is_permanent()))
    });

    group.bench_function("erract/kind_check", |b| {
        b.iter(|| black_box(erract_error.kind() == &erract::ErrorKind::Timeout))
    });

    // --- anyhow downcast (for comparison) ---

    #[derive(Debug)]
    struct CustomError;
    impl std::fmt::Display for CustomError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "custom error")
        }
    }
    impl std::error::Error for CustomError {}

    let anyhow_typed: anyhow::Error = CustomError.into();

    group.bench_function("anyhow/downcast_ref", |b| {
        b.iter(|| black_box(anyhow_typed.downcast_ref::<CustomError>().is_some()))
    });

    group.finish();
}

// ============================================================================
// GROUP 6: Memory Footprint (informational)
// ============================================================================

fn bench_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_footprint");

    // These are not really benchmarks, but we use criterion to report them
    // consistently. We measure allocation by creating and dropping.

    group.bench_function("erract/error_size", |b| {
        b.iter(std::mem::size_of::<erract::Error>)
    });

    group.bench_function("anyhow/error_size", |b| {
        b.iter(std::mem::size_of::<anyhow::Error>)
    });

    group.bench_function("erract/exn_size", |b| {
        b.iter(std::mem::size_of::<exn::Exn<erract::Error>>)
    });

    group.finish();

    // Print sizes for reference
    println!("\n=== Memory Footprint (stack size) ===");
    println!(
        "erract::Error:           {} bytes",
        std::mem::size_of::<erract::Error>()
    );
    println!(
        "anyhow::Error:           {} bytes",
        std::mem::size_of::<anyhow::Error>()
    );
    println!(
        "exn::Exn<erract::Error>: {} bytes",
        std::mem::size_of::<exn::Exn<erract::Error>>()
    );
    println!(
        "erract::ErrorKind:       {} bytes",
        std::mem::size_of::<erract::ErrorKind>()
    );
    println!(
        "erract::ErrorStatus:     {} bytes",
        std::mem::size_of::<erract::ErrorStatus>()
    );
}

// ============================================================================
// GROUP 7: Real-world Scenario
// ============================================================================

mod erract_scenario {
    use erract::prelude::*;

    #[inline(never)]
    fn validate_input(input: &str) -> erract::Result<()> {
        if input.is_empty() {
            return Err(
                Error::permanent(ErrorKind::Validation, "input cannot be empty")
                    .with_context("field", "username")
                    .raise(),
            );
        }
        Ok(())
    }

    #[inline(never)]
    fn fetch_user(id: u32) -> erract::Result<String> {
        // Simulate some work
        std::hint::black_box(id);
        Ok(format!("User{id}"))
    }

    #[inline(never)]
    fn process_request(id: u32, input: &str) -> erract::Result<String> {
        validate_input(input)
            .or_raise(|| Error::permanent(ErrorKind::Validation, "validation failed"))?;

        let user = fetch_user(id)
            .or_raise(|| Error::permanent(ErrorKind::Unexpected, "failed to fetch user"))?;

        Ok(format!("Processed: {user} with input: {input}"))
    }

    pub fn run_success() -> erract::Result<String> {
        process_request(42, "valid_input")
    }

    pub fn run_validation_error() -> erract::Result<String> {
        process_request(42, "")
    }

    pub fn run_not_found_error() -> erract::Result<String> {
        process_request(0, "valid_input")
    }
}

mod anyhow_scenario {
    use anyhow::{Context, Result, anyhow};

    #[inline(never)]
    fn validate_input(input: &str) -> Result<()> {
        if input.is_empty() {
            return Err(anyhow!("input cannot be empty"));
        }
        Ok(())
    }

    #[inline(never)]
    fn fetch_user(id: u32) -> Result<String> {
        // Simulate some work
        std::hint::black_box(id);
        if id == 0 {
            return Err(anyhow!("user not found: {id}"));
        }
        Ok(format!("User{id}"))
    }

    #[inline(never)]
    fn process_request(id: u32, input: &str) -> Result<String> {
        validate_input(input).context("validation failed")?;

        let user = fetch_user(id).context("failed to fetch user")?;

        Ok(format!("Processed: {user} with input: {input}"))
    }

    pub fn run_success() -> Result<String> {
        process_request(42, "valid_input")
    }

    pub fn run_validation_error() -> Result<String> {
        process_request(42, "")
    }

    pub fn run_not_found_error() -> Result<String> {
        process_request(0, "valid_input")
    }
}

fn bench_scenario(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world_scenario");

    // --- Success path ---

    group.bench_function("erract/success", |b| {
        b.iter(|| black_box(erract_scenario::run_success()))
    });

    group.bench_function("anyhow/success", |b| {
        b.iter(|| black_box(anyhow_scenario::run_success()))
    });

    // --- Validation error path ---

    group.bench_function("erract/validation_error", |b| {
        b.iter(|| black_box(erract_scenario::run_validation_error()))
    });

    group.bench_function("anyhow/validation_error", |b| {
        b.iter(|| black_box(anyhow_scenario::run_validation_error()))
    });

    // --- Not found error path ---

    group.bench_function("erract/not_found_error", |b| {
        b.iter(|| black_box(erract_scenario::run_not_found_error()))
    });

    group.bench_function("anyhow/not_found_error", |b| {
        b.iter(|| black_box(anyhow_scenario::run_not_found_error()))
    });

    group.finish();
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    benches,
    bench_error_creation,
    bench_context,
    bench_propagation,
    bench_display,
    bench_inspection,
    bench_memory,
    bench_scenario,
);

criterion_main!(benches);

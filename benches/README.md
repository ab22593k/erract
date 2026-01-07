# Performance Benchmarks: erract vs anyhow

This directory contains comprehensive benchmarks comparing `erract` with `anyhow`, the most popular error handling crate in the Rust ecosystem.

## Quick Start

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark group
cargo bench -- error_creation
cargo bench -- context_addition
cargo bench -- error_propagation
cargo bench -- error_display
cargo bench -- error_inspection
cargo bench -- memory_footprint
cargo bench -- real_world_scenario
```

## Benchmark Groups

### 1. Error Creation (`error_creation`)

Compares the cost of creating errors:

- **Basic**: Simple error with static message
- **Formatted**: Error with dynamic format string
- **Temporary**: erract-specific temporary error creation

### 2. Context Addition (`context_addition`)

Compares adding context to errors:

- **Single**: One context item
- **Triple**: Three context items chained
- **With Operation**: erract-specific operation annotation

### 3. Error Propagation (`error_propagation`)

Compares error propagation through call stacks:

- **Depth 1**: Single function returning error
- **Depth 3**: Three levels of error wrapping
- **Depth 5**: Five levels of error wrapping

This tests the overhead of `?` operator and context wrapping.

### 4. Error Display (`error_display`)

Compares error formatting:

- **to_string**: Display trait formatting
- **debug**: Debug trait formatting
- **to_json**: erract-specific JSON output
- **to_machine_string**: erract-specific machine-readable output

### 5. Error Inspection (`error_inspection`)

Compares querying error properties:

- **is_retryable**: erract retry semantics check
- **is_permanent**: erract permanence check
- **kind_check**: erract error kind comparison
- **downcast_ref**: anyhow type downcasting

### 6. Memory Footprint (`memory_footprint`)

Reports stack sizes of error types:

- `erract::Error`
- `anyhow::Error`
- `exn::Exn<erract::Error>`

### 7. Real-world Scenario (`real_world_scenario`)

Simulates a realistic request processing flow:

- **Success path**: No errors
- **Validation error**: Early error with context
- **Not found error**: Error in nested function

## Backtrace Comparison

anyhow captures backtraces when enabled. To compare performance with/without:

```bash
# Without backtraces (default, faster)
cargo bench -- --save-baseline no_backtrace

# With backtraces enabled (slower, more debug info)
RUST_BACKTRACE=1 cargo bench -- --save-baseline with_backtrace

# Compare the two
cargo bench -- --baseline no_backtrace
```

## Reproducible Results

### Environment Setup

For consistent benchmarking:

```bash
# 1. Close unnecessary applications

# 2. (Linux) Set CPU governor to performance mode
sudo cpupower frequency-set --governor performance

# 3. (Linux) Disable CPU frequency scaling
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# 4. Run benchmarks multiple times
for i in {1..3}; do
    cargo bench -- --save-baseline run_$i
done
```

### Recommended Benchmark Command

```bash
# Full benchmark with HTML report
cargo bench --all-features

# View HTML report
open target/criterion/report/index.html  # macOS
xdg-open target/criterion/report/index.html  # Linux
```

### CI/Automation

For CI integration, use JSON output:

```bash
cargo bench -- --format json > benchmark_results.json
```

## Expected Results

Based on architectural differences:

| Benchmark          | erract    | anyhow  | Reason                       |
| ------------------ | --------- | ------- | ---------------------------- |
| Basic creation     | ~equal    | ~equal  | Both allocate String         |
| Formatted creation | ~equal    | ~equal  | Both use format!             |
| Single context     | Faster    | Slower  | Vec push vs allocation       |
| Triple context     | Faster    | Slower  | Vec grows vs 3 allocations   |
| Propagation (deep) | Faster    | Slower  | #[track_caller] vs backtrace |
| to_string          | ~equal    | ~equal  | Both format strings          |
| is_retryable       | Very fast | N/A     | Direct field access          |
| Memory (stack)     | Larger    | Smaller | Structured data vs pointer   |

## Key Architectural Differences

### erract

- Uses `#[track_caller]` for zero-cost location tracking
- Stores context as `Vec<(Cow<str>, String)>` - efficient for multiple items
- Structured error kinds enable O(1) retryability checks
- Error trees via `exn` crate

### anyhow

- Captures full backtraces (when enabled) - significant overhead
- Wraps errors in trait objects - one allocation per context
- No structured error kinds - requires downcasting or string parsing
- Error chain via `source()` method

## Interpreting Results

Criterion reports:

- **Time**: Lower is better
- **Throughput**: Higher is better
- **Change**: Percentage vs baseline (if saved)

## Extending Benchmarks

To add new benchmarks:

1. Add a new function in `benchmark.rs`:

   ```rust
   fn bench_my_scenario(c: &mut Criterion) {
       let mut group = c.benchmark_group("my_scenario");
       group.bench_function("erract/case", |b| { ... });
       group.bench_function("anyhow/case", |b| { ... });
       group.finish();
   }
   ```

2. Add to criterion group:

   ```rust
   criterion_group!(
       benches,
       // ... existing benchmarks
       bench_my_scenario,
   );
   ```

3. Run: `cargo bench -- my_scenario`

## Troubleshooting

### Benchmarks run too slowly

```bash
# Use fewer samples
cargo bench -- --sample-size 10
```

### Results are inconsistent

```bash
# Increase measurement time
cargo bench -- --measurement-time 10
```

### Want to compare specific commits

```bash
git checkout main
cargo bench -- --save-baseline main

git checkout feature-branch
cargo bench -- --baseline main
```

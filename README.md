# erract - Structured Error Handling for Rust

[![Crates.io](https://img.shields.io/crates/v/erract.svg)](https://crates.io/crates/erract)
[![Documentation](https://docs.rs/erract/badge.svg)](https://docs.rs/erract)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

**erract** provides production-ready error handling that solves real problems:

- **Actionable errors**: Categorized by what callers can do (retry, fail, etc.)
- **Explicit retry semantics**: No guessing from error messages
- **Rich context**: Automatic location capture with zero overhead
- **Type safety**: Enforce context at module boundaries

## Philosophy

Two audiences, two needs:

- **Machines**: Need flat structures, clear error kinds, predictable codes
- **Humans**: Need rich context, call paths, business-level information

This library combines the best ideas from:
- [Apache OpenDAL's error design](https://github.com/apache/opendal/pull/977)
- [The exn crate](https://github.com/fast/exn)
- Years of production error handling experience

## Quick Start

```rust
use erract::prelude::*;

fn process_user(id: u32) -> erract::Result<String> {
    // Use static constructors for zero-allocation common errors
    let user = lookup_user(id)
        .or_raise(|| Error::not_found())?
        .ok_or_else(|| {
            Error::permanent(ErrorKind::NotFound, format!("user {} not found", id))
                .with_context("user_id", id.to_string())
                .raise()
        })?;
    
    Ok(user)
}

fn lookup_user(id: u32) -> erract::Result<Option<String>> {
    if id == 0 {
        // Static message - zero allocation
        return Err(Error::not_found().raise());
    }
    Ok(Some(format!("User{}", id)))
}
```

## Features

- **Zero runtime overhead**: Uses `#[track_caller]` instead of expensive backtraces
- **Domain-specific errors**: HTTP, Database, and Storage error kinds
- **Error trees**: Support for multiple concurrent failures
- **Type safety**: Compiler enforces context at module boundaries
- **Zero-copy static strings**: Use `Cow<'static, str>` for static messages
- **Optimized serialization**: Fast JSON and machine-readable output

## API Highlights

### Static Constructors (Zero Allocation)

```rust
// Common errors with pre-defined static messages
let err = Error::not_found();           // "not found"
let err = Error::timeout();             // "operation timed out"
let err = Error::permission_denied();   // "permission denied"
let err = Error::validation_failed();   // "validation failed"
let err = Error::unexpected();          // "unexpected error"
```

### Context Methods

```rust
// For string context (zero-copy for static strings)
error.with_context("key", "value")

// For non-string values (uses ToString)
error.with_context_value("count", 42)
```

### Serialization

```rust
let error = Error::not_found()
    .with_context("resource", "user")
    .with_operation("fetch");

// Human-readable
println!("{}", error);  // "not found (operation: fetch) [resource: user]"

// Machine-readable
error.to_machine_string()  // "kind=not_found;status=permanent;message=not found;..."

// JSON (optimized, ~220ns)
error.to_json()  // {"kind":"not_found","status":"permanent",...}
```

## Comparison with anyhow

erract and [anyhow](https://github.com/dtolnay/anyhow) serve different use cases:

| Aspect | erract | anyhow |
|--------|--------|--------|
| **Design Goal** | Production services with structured errors | Quick prototyping, application code |
| **Retry Semantics** | Explicit (`is_retryable()`, `is_permanent()`) | None (manual parsing) |
| **Error Kind** | Structured enum (`ErrorKind`) | Dynamic (downcast required) |
| **Context Model** | Key-value pairs | String messages |
| **Machine Output** | Built-in JSON, machine strings | Manual formatting |
| **Location Tracking** | `#[track_caller]` (zero cost) | Backtrace (optional overhead) |

### Performance Benchmarks

Run `cargo bench` to reproduce. Results on Linux x86_64:

| Benchmark | erract | anyhow | Winner |
|-----------|--------|--------|--------|
| Basic error creation | 22 ns | 30 ns | erract (+27%) |
| Formatted error | 81 ns | 106 ns | erract (+24%) |
| Single context | 24 ns | 58 ns | erract (+59%) |
| Triple context | 190 ns | 115 ns | anyhow (+39%) |
| to_string | 268 ns | 52 ns | anyhow |
| to_json | 221 ns | N/A | erract only |
| `is_retryable()` check | <1 ns | N/A | erract only |

**Key Insights:**
- **erract excels** at error creation, single context, and structured queries
- **anyhow excels** at deep error propagation and display formatting
- **Memory**: erract::Error is 112 bytes (structured), anyhow::Error is 8 bytes (pointer)
- **Trade-off**: erract uses more stack space for richer structured data

### When to Choose

**Choose erract when:**
- Building production services with retry logic
- Need machine-parseable errors for alerting/metrics
- Want explicit error semantics at compile time
- Multiple teams consume your error data

**Choose anyhow when:**
- Building CLI tools or quick prototypes
- Error handling is human-only (logs, not machines)
- Minimal boilerplate is priority
- Don't need structured error classification

See [benches/README.md](benches/README.md) for detailed benchmark methodology.

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

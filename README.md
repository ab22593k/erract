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
    let error = || Error::permanent(
        ErrorKind::NotFound,
        format!("user not found: {}", id),
    );

    lookup_user(id)
        .or_raise(error)?
        .ok_or_else(|| {
            Error::permanent(
                ErrorKind::NotFound,
                "user not found".to_string(),
            ).raise()
        })
}

fn lookup_user(id: u32) -> erract::Result<Option<String>> {
    Ok(None)
}
```

## Features

- **Zero runtime overhead**: Uses `#[track_caller]` instead of expensive backtraces
- **Domain-specific errors**: HTTP, Database, and Storage error kinds
- **Error trees**: Support for multiple concurrent failures
- **Type safety**: Compiler enforces context at module boundaries

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

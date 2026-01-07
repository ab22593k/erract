#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::all)]

//! # erract - Structured Error Handling for Rust
//!
//! erract provides production-ready error handling that solves real problems:
//!
//! - **Actionable errors**: Categorized by what callers can do (retry, fail, etc.)
//! - **Explicit retry semantics**: No guessing from error messages
//! - **Rich context**: Automatic location capture with zero overhead
//! - **Type safety**: Enforce context at module boundaries
//!
//! ## Philosophy
//!
//! Two audiences, two needs:
//!
//! - **Machines**: Need flat structures, clear error kinds, predictable codes
//! - **Humans**: Need rich context, call paths, business-level information
//!
//! This library combines the best ideas from:
//! - [Apache OpenDAL's error design](https://github.com/apache/opendal/pull/977)
//! - [The exn crate](https://github.com/fast/exn)
//! - Years of production error handling experience
//!
//! ## Quick Start
//!
//! ```
//! use erract::prelude::*;
//!
//! fn process_user(id: u32) -> erract::Result<String> {
//!     let error = || Error::permanent(
//!         ErrorKind::NotFound,
//!         format!("user not found: {}", id),
//!     );
//!
//!     lookup_user(id)
//!         .or_raise(error)?
//!         .ok_or_else(|| {
//!             Error::permanent(
//!                 ErrorKind::NotFound,
//!                 "user not found".to_string(),
//!             ).raise()
//!         })
//! }
//!
//! fn lookup_user(id: u32) -> erract::Result<Option<String>> {
//!     Ok(None)
//! }
//! # fn main() {}
//! ```
//!
//! ## Features
//!
//! - **Zero runtime overhead**: Uses `#[track_caller]` instead of expensive backtraces
//! - **Domain-specific errors**: HTTP, Database, and Storage error kinds
//! - **Error trees**: Support for multiple concurrent failures
//! - **Type safety**: Compiler enforces context at module boundaries

/// Context utilities for adding key-value pairs to errors.
pub mod context;

/// Core error type and builders.
pub mod error;

/// Error tree traversal utilities.
pub mod extract;

/// Domain-specific error kinds categorized by action.
pub mod kind;

/// Explicit retry semantics for errors.
pub mod status;

/// Common imports for using erract.
pub mod prelude;

/// Conversions from standard library error types.
pub mod convert;

/// HTTP-specific error kinds.
#[cfg(feature = "http")]
pub mod http;

/// Database-specific error kinds.
#[cfg(feature = "db")]
pub mod db;

/// Storage-specific error kinds.
#[cfg(feature = "storage")]
pub mod storage;

pub use crate::context::AddContext;
pub use crate::error::{Error, ErrorBuilder};
pub use crate::extract::{
    count_errors, count_frames, has_permanent, has_retryable, is_all_retryable,
};
pub use crate::kind::ErrorKind;
pub use crate::status::ErrorStatus;

// Re-export exn for convenience
pub use exn;

// Type alias for convenience
/// Result type using `exn::Exn<Error>` as the error type
pub type Result<T> = std::result::Result<T, exn::Exn<Error>>;

/// Equivalent to `Ok::<_, Exn<Error>>(value)`.
///
/// This simplifies creation of an `erract::Result` in places where type inference cannot deduce the
/// error type.
#[inline]
pub fn ok<T>(value: T) -> Result<T> {
    Ok(value)
}

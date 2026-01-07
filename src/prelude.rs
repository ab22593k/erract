//! Common imports for using erract.
//!
//! This prelude provides convenient access to the most commonly used
//! types and traits in the erract library.
//!
//! # Example
//!
//! ```ignore
//! use erract::prelude::*;
//!
//! fn example() -> erract::Result<()> {
//!     Err(Error::permanent(ErrorKind::NotFound, "not found").raise())
//!         .with_context("key", "value")
//!         .or_raise(|| Error::temporary(ErrorKind::Unexpected, "wrapper"))
//! }
//! ```
pub use crate::context::AddContext;
pub use crate::error::{Error, ErrorBuilder};
pub use crate::extract::{count_errors, count_frames};
pub use crate::kind::ErrorKind;
pub use crate::status::ErrorStatus;
pub use exn::{ResultExt, bail, ensure};

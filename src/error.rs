use std::borrow::Cow;
use std::error::Error as ErrorTrait;
use std::fmt::{self, Write};

use smallvec::SmallVec;

use crate::{ErrorKind, ErrorStatus};

pub use self::builder::ErrorBuilder;

/// Type alias for context storage.
/// Uses SmallVec to avoid heap allocation for errors with 0-1 context items.
/// Most errors have zero or one context item, so this optimizes the common case.
pub type ContextVec = SmallVec<[(Cow<'static, str>, Cow<'static, str>); 1]>;

/// Core error type for the erract library.
///
/// This struct represents a single error with:
/// - An actionable [`ErrorKind`] describing what the caller can do
/// - An explicit [`ErrorStatus`] describing retry semantics
/// - A human-readable message
/// - Optional operation name for debugging
/// - Key-value context for troubleshooting
///
/// # Memory Layout
///
/// The error is optimized for minimal allocations:
/// - Message uses `Cow<'static, str>` for zero-copy static strings
/// - Context uses `SmallVec` to inline 0-2 items without heap allocation
///
/// # Examples
///
/// ```
/// use erract::{Error, ErrorKind, ErrorStatus};
///
/// let error = Error::permanent(
///     ErrorKind::NotFound,
///     "user not found: 12345"
/// ).with_context("user_id", "12345")
///  .with_operation("lookup_user");
///
/// assert!(!error.is_retryable());
/// assert_eq!(error.kind(), &ErrorKind::NotFound);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    kind: ErrorKind,
    status: ErrorStatus,
    message: Cow<'static, str>,
    operation: Option<&'static str>,
    pub(crate) context: ContextVec,
}

impl Error {
    // ========================================================================
    // Common pre-allocated error constructors (zero allocation for message)
    // ========================================================================

    /// Creates a "not found" error with zero message allocation.
    #[inline]
    pub fn not_found() -> Self {
        Self::permanent_static(ErrorKind::NotFound, "not found")
    }

    /// Creates a "permission denied" error with zero message allocation.
    #[inline]
    pub fn permission_denied() -> Self {
        Self::permanent_static(ErrorKind::PermissionDenied, "permission denied")
    }

    /// Creates a "timeout" error with zero message allocation.
    #[inline]
    pub fn timeout() -> Self {
        Self::temporary_static(ErrorKind::Timeout, "operation timed out")
    }

    /// Creates a "validation failed" error with zero message allocation.
    #[inline]
    pub fn validation_failed() -> Self {
        Self::permanent_static(ErrorKind::Validation, "validation failed")
    }

    /// Creates an "unexpected error" with zero message allocation.
    #[inline]
    pub fn unexpected() -> Self {
        Self::permanent_static(ErrorKind::Unexpected, "unexpected error")
    }

    // ========================================================================
    // Static message constructors (zero allocation)
    // ========================================================================

    /// Creates a permanent error with a static message (zero allocation).
    #[inline]
    pub fn permanent_static(kind: ErrorKind, message: &'static str) -> Self {
        Error {
            kind,
            status: ErrorStatus::Permanent,
            message: Cow::Borrowed(message),
            operation: None,
            context: SmallVec::new(),
        }
    }

    /// Creates a temporary error with a static message (zero allocation).
    #[inline]
    pub fn temporary_static(kind: ErrorKind, message: &'static str) -> Self {
        Error {
            kind,
            status: ErrorStatus::Temporary,
            message: Cow::Borrowed(message),
            operation: None,
            context: SmallVec::new(),
        }
    }

    /// Creates a persistent error with a static message (zero allocation).
    #[inline]
    pub fn persistent_static(kind: ErrorKind, message: &'static str) -> Self {
        Error {
            kind,
            status: ErrorStatus::Persistent,
            message: Cow::Borrowed(message),
            operation: None,
            context: SmallVec::new(),
        }
    }

    // ========================================================================
    // Dynamic message constructors
    // ========================================================================

    /// Creates a permanent error that cannot be retried.
    #[inline]
    pub fn permanent(kind: ErrorKind, message: impl Into<Cow<'static, str>>) -> Self {
        Error {
            kind,
            status: ErrorStatus::Permanent,
            message: message.into(),
            operation: None,
            context: SmallVec::new(),
        }
    }

    /// Creates a temporary error that is safe to retry.
    #[inline]
    pub fn temporary(kind: ErrorKind, message: impl Into<Cow<'static, str>>) -> Self {
        Error {
            kind,
            status: ErrorStatus::Temporary,
            message: message.into(),
            operation: None,
            context: SmallVec::new(),
        }
    }

    /// Creates a persistent error that was already retried.
    #[inline]
    pub fn persistent(kind: ErrorKind, message: impl Into<Cow<'static, str>>) -> Self {
        Error {
            kind,
            status: ErrorStatus::Persistent,
            message: message.into(),
            operation: None,
            context: SmallVec::new(),
        }
    }

    /// Creates an error with a specific status.
    #[inline]
    pub fn new(
        kind: ErrorKind,
        status: ErrorStatus,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Error {
            kind,
            status,
            message: message.into(),
            operation: None,
            context: SmallVec::new(),
        }
    }

    /// Returns a builder for configuring this error.
    #[inline]
    pub fn builder(
        kind: ErrorKind,
        status: ErrorStatus,
        message: impl Into<Cow<'static, str>>,
    ) -> ErrorBuilder {
        ErrorBuilder::new(kind, status, message)
    }

    // ========================================================================
    // Accessors
    // ========================================================================

    /// Returns a reference to the error kind.
    #[inline(always)]
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// Returns a reference to the error status.
    #[inline(always)]
    pub fn status(&self) -> &ErrorStatus {
        &self.status
    }

    /// Returns a reference to the error message.
    #[inline(always)]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the operation name if set.
    #[inline(always)]
    pub fn operation(&self) -> Option<&'static str> {
        self.operation
    }

    /// Returns a reference to the context key-value pairs.
    #[inline(always)]
    pub fn context(&self) -> &[(Cow<'static, str>, Cow<'static, str>)] {
        &self.context
    }

    /// Returns `true` if this error is safe to retry.
    #[inline(always)]
    pub fn is_retryable(&self) -> bool {
        self.status.is_retryable()
    }

    /// Returns `true` if this error is permanent.
    #[inline(always)]
    pub fn is_permanent(&self) -> bool {
        self.status.is_permanent()
    }

    // ========================================================================
    // Builder methods
    // ========================================================================

    /// Sets the operation name for this error.
    #[inline]
    #[must_use]
    pub fn with_operation(mut self, operation: &'static str) -> Self {
        self.operation = Some(operation);
        self
    }

    /// Adds a key-value pair to the error context.
    #[inline]
    #[must_use]
    pub fn with_context(
        mut self,
        key: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> Self {
        self.context.push((key.into(), value.into()));
        self
    }

    /// Adds a key-value pair where value is converted via ToString.
    ///
    /// Use this when the value needs to be converted from a non-string type.
    #[inline]
    #[must_use]
    pub fn with_context_value(
        mut self,
        key: impl Into<Cow<'static, str>>,
        value: impl ToString,
    ) -> Self {
        self.context
            .push((key.into(), Cow::Owned(value.to_string())));
        self
    }

    /// Converts this error into an Exn for context-aware propagation.
    ///
    /// This method is available when using `exn::Exn` as the error type.
    /// It allows the error to be wrapped in the exn error tree structure.
    #[track_caller]
    pub fn raise(self) -> exn::Exn<Self>
    where
        Self: ErrorTrait + 'static,
    {
        exn::Exn::new(self)
    }
}

// ============================================================================
// Display implementation (optimized to minimize allocations)
// ============================================================================

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)?;
        if let Some(op) = self.operation {
            f.write_str(" (operation: ")?;
            f.write_str(op)?;
            f.write_char(')')?;
        }
        if !self.context.is_empty() {
            f.write_str(" [")?;
            let mut first = true;
            for (key, value) in &self.context {
                if !first {
                    f.write_str(", ")?;
                }
                f.write_str(key)?;
                f.write_str(": ")?;
                f.write_str(value)?;
                first = false;
            }
            f.write_char(']')?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

// ============================================================================
// Serialization methods (optimized)
// ============================================================================

impl Error {
    /// Returns a machine-readable string representation of this error.
    ///
    /// The format is: `kind={kind};status={status};message={message};operation={operation};context={context}`
    ///
    /// This is useful for logging systems that require structured output.
    ///
    /// # Examples
    ///
    /// ```
    /// use erract::{Error, ErrorKind, ErrorStatus};
    ///
    /// let error = Error::permanent(ErrorKind::NotFound, "user not found")
    ///     .with_context("user_id", "123")
    ///     .with_operation("fetch_user");
    ///
    /// let machine = error.to_machine_string();
    /// assert!(machine.contains("kind=not_found"));
    /// assert!(machine.contains("status=permanent"));
    /// ```
    pub fn to_machine_string(&self) -> String {
        // Pre-calculate approximate capacity to reduce reallocations
        let capacity = 64 + self.message.len() + self.context.len() * 32;
        let mut output = String::with_capacity(capacity);

        output.push_str("kind=");
        output.push_str(&self.kind.to_machine_string());
        output.push_str(";status=");
        output.push_str(&self.status.to_machine_string());
        output.push_str(";message=");
        output.push_str(&self.message);

        if let Some(op) = self.operation {
            output.push_str(";operation=");
            output.push_str(op);
        }

        if !self.context.is_empty() {
            output.push_str(";context=[");
            let mut first = true;
            for (k, v) in &self.context {
                if !first {
                    output.push(',');
                }
                output.push_str(k);
                output.push('=');
                output.push_str(v);
                first = false;
            }
            output.push(']');
        }

        output
    }

    /// Returns a JSON representation of this error.
    ///
    /// This is useful for APIs that need to return structured error responses.
    ///
    /// # Examples
    ///
    /// ```
    /// use erract::{Error, ErrorKind, ErrorStatus};
    ///
    /// let error = Error::permanent(ErrorKind::NotFound, "user not found")
    ///     .with_context("user_id", "123");
    ///
    /// let json = error.to_json();
    /// assert!(json.contains("\"kind\""));
    /// assert!(json.contains("\"status\""));
    /// ```
    pub fn to_json(&self) -> String {
        // Pre-calculate approximate capacity
        let capacity = 128 + self.message.len() + self.context.len() * 48;
        let mut json = String::with_capacity(capacity);

        json.push_str(r#"{"kind":""#);
        json.push_str(&self.kind.to_machine_string());
        json.push_str(r#"","status":""#);
        json.push_str(&self.status.to_machine_string());
        json.push_str(r#"","message":""#);
        write_escaped(&mut json, &self.message);
        json.push('"');

        if let Some(op) = self.operation {
            json.push_str(r#","operation":""#);
            json.push_str(op);
            json.push('"');
        }

        if !self.context.is_empty() {
            json.push_str(r#","context":{"#);
            let mut first = true;
            for (k, v) in &self.context {
                if !first {
                    json.push(',');
                }
                json.push('"');
                write_escaped(&mut json, k);
                json.push_str(r#"":""#);
                write_escaped(&mut json, v);
                json.push('"');
                first = false;
            }
            json.push('}');
        }

        json.push('}');
        json
    }

    /// Writes JSON to the provided buffer, avoiding allocation.
    ///
    /// This is useful when you want to write to an existing buffer.
    pub fn write_json(&self, buf: &mut String) {
        buf.push_str(r#"{"kind":""#);
        buf.push_str(&self.kind.to_machine_string());
        buf.push_str(r#"","status":""#);
        buf.push_str(&self.status.to_machine_string());
        buf.push_str(r#"","message":""#);
        write_escaped(buf, &self.message);
        buf.push('"');

        if let Some(op) = self.operation {
            buf.push_str(r#","operation":""#);
            buf.push_str(op);
            buf.push('"');
        }

        if !self.context.is_empty() {
            buf.push_str(r#","context":{"#);
            let mut first = true;
            for (k, v) in &self.context {
                if !first {
                    buf.push(',');
                }
                buf.push('"');
                write_escaped(buf, k);
                buf.push_str(r#"":""#);
                write_escaped(buf, v);
                buf.push('"');
                first = false;
            }
            buf.push('}');
        }

        buf.push('}');
    }
}

/// Helper function to write JSON-escaped strings efficiently.
#[inline]
fn write_escaped(buf: &mut String, s: &str) {
    for c in s.chars() {
        match c {
            '"' => buf.push_str(r#"\""#),
            '\\' => buf.push_str(r#"\\"#),
            '\n' => buf.push_str(r#"\n"#),
            '\r' => buf.push_str(r#"\r"#),
            '\t' => buf.push_str(r#"\t"#),
            c if c.is_control() => {
                // Unicode escape for control characters
                let _ = write!(buf, r#"\u{:04x}"#, c as u32);
            }
            c => buf.push(c),
        }
    }
}

// ============================================================================
// Builder
// ============================================================================

mod builder {
    use std::borrow::Cow;
    use std::fmt;

    use smallvec::SmallVec;

    use super::{Error, ErrorKind, ErrorStatus};

    /// Builder for configuring [`Error`] with additional context.
    ///
    /// # Examples
    ///
    /// ```
    /// use erract::{Error, ErrorKind, ErrorStatus};
    ///
    /// let error = Error::builder(
    ///     ErrorKind::NotFound,
    ///     ErrorStatus::Permanent,
    ///     "resource not found"
    /// ).with_operation("fetch_resource")
    ///  .with_context("resource_id", "abc123")
    ///  .with_context("resource_type", "user")
    ///  .build();
    /// ```
    #[derive(Debug)]
    pub struct ErrorBuilder {
        error: Error,
    }

    impl ErrorBuilder {
        /// Creates a new builder with the required error parameters.
        #[inline]
        pub fn new(
            kind: ErrorKind,
            status: ErrorStatus,
            message: impl Into<Cow<'static, str>>,
        ) -> Self {
            ErrorBuilder {
                error: Error {
                    kind,
                    status,
                    message: message.into(),
                    operation: None,
                    context: SmallVec::new(),
                },
            }
        }

        /// Sets the operation name.
        #[inline]
        #[must_use]
        pub fn with_operation(mut self, operation: &'static str) -> Self {
            self.error.operation = Some(operation);
            self
        }

        /// Adds a context key-value pair.
        #[inline]
        #[must_use]
        pub fn with_context(
            mut self,
            key: impl Into<Cow<'static, str>>,
            value: impl Into<Cow<'static, str>>,
        ) -> Self {
            self.error.context.push((key.into(), value.into()));
            self
        }

        /// Adds a context key-value pair where value is converted via ToString.
        #[inline]
        #[must_use]
        pub fn with_context_value(
            mut self,
            key: impl Into<Cow<'static, str>>,
            value: impl ToString,
        ) -> Self {
            self.error
                .context
                .push((key.into(), Cow::Owned(value.to_string())));
            self
        }

        /// Adds multiple context key-value pairs.
        #[inline]
        #[must_use]
        pub fn with_context_iter<K, V>(mut self, iter: impl IntoIterator<Item = (K, V)>) -> Self
        where
            K: Into<Cow<'static, str>>,
            V: Into<Cow<'static, str>>,
        {
            for (key, value) in iter {
                self.error.context.push((key.into(), value.into()));
            }
            self
        }

        /// Builds the final [`Error`].
        #[inline]
        pub fn build(self) -> Error {
            self.error
        }
    }

    impl fmt::Display for ErrorBuilder {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Display::fmt(&self.error, f)
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permanent_error() {
        let error = Error::permanent(ErrorKind::NotFound, "not found");
        assert!(!error.is_retryable());
        assert!(error.is_permanent());
        assert_eq!(error.kind(), &ErrorKind::NotFound);
        assert_eq!(error.message(), "not found");
    }

    #[test]
    fn test_temporary_error() {
        let error = Error::temporary(ErrorKind::Timeout, "timeout");
        assert!(error.is_retryable());
        assert!(!error.is_permanent());
        assert_eq!(error.kind(), &ErrorKind::Timeout);
    }

    #[test]
    fn test_context() {
        let error = Error::permanent(ErrorKind::NotFound, "not found")
            .with_context("user_id", "123")
            .with_context("operation", "lookup");

        assert_eq!(error.context().len(), 2);
        assert_eq!(error.context()[0].0, "user_id");
        assert_eq!(error.context()[0].1, "123");
    }

    #[test]
    fn test_context_value() {
        let error = Error::permanent(ErrorKind::NotFound, "not found")
            .with_context_value("user_id", 123)
            .with_context_value("count", 42u64);

        assert_eq!(error.context().len(), 2);
        assert_eq!(error.context()[0].0, "user_id");
        assert_eq!(error.context()[0].1, "123");
        assert_eq!(error.context()[1].1, "42");
    }

    #[test]
    fn test_operation() {
        let error = Error::permanent(ErrorKind::NotFound, "not found").with_operation("fetch_user");
        assert_eq!(error.operation(), Some("fetch_user"));
    }

    #[test]
    fn test_display() {
        let error = Error::permanent(ErrorKind::NotFound, "not found")
            .with_operation("fetch")
            .with_context("id", "123");

        let display = error.to_string();
        assert!(display.contains("not found"));
        assert!(display.contains("fetch"));
        assert!(display.contains("id"));
        assert!(display.contains("123"));
    }

    #[test]
    fn test_builder() {
        let error = Error::builder(ErrorKind::NotFound, ErrorStatus::Permanent, "not found")
            .with_operation("fetch")
            .with_context("id", "123")
            .build();

        assert!(!error.is_retryable());
        assert_eq!(error.operation(), Some("fetch"));
        assert_eq!(error.context().len(), 1);
    }

    #[test]
    fn test_static_constructors() {
        let error = Error::not_found();
        assert_eq!(error.kind(), &ErrorKind::NotFound);
        assert!(error.is_permanent());
        assert_eq!(error.message(), "not found");

        let error = Error::timeout();
        assert_eq!(error.kind(), &ErrorKind::Timeout);
        assert!(error.is_retryable());
    }

    #[test]
    fn test_json_escaping() {
        let error = Error::permanent(ErrorKind::Validation, "invalid \"input\"\nwith newline")
            .with_context("field", "user\\name");

        let json = error.to_json();
        assert!(json.contains(r#"invalid \"input\"\nwith newline"#));
        assert!(json.contains(r#"user\\name"#));
    }

    #[test]
    fn test_memory_size() {
        // Verify size is reasonable (not a hard requirement, just informational)
        let size = std::mem::size_of::<Error>();
        println!("Error size: {size} bytes");
        // With SmallVec<[_; 1]> and Cow<str>, size will be larger than original
        // but we gain zero-copy for static strings and inline storage for 1 context item
        // The trade-off is acceptable for the performance gains
        assert!(size <= 160, "Error size {size} exceeds 160 bytes");
    }

    #[test]
    fn test_smallvec_inline() {
        // Verify SmallVec is being used efficiently
        let error =
            Error::permanent(ErrorKind::NotFound, "not found").with_context("key1", "value1");

        // With SmallVec<[_; 1]>, 1 item should be inline
        assert_eq!(error.context().len(), 1);
        // This should not have spilled to heap
        assert!(!error.context.spilled());

        // Adding a second item will spill to heap
        let error2 = error.with_context("key2", "value2");
        assert_eq!(error2.context().len(), 2);
        assert!(error2.context.spilled());
    }

    #[test]
    fn test_write_json() {
        let error = Error::permanent(ErrorKind::NotFound, "test").with_context("key", "value");

        let mut buf = String::new();
        error.write_json(&mut buf);

        assert!(buf.contains("\"kind\":\"not_found\""));
        assert!(buf.contains("\"key\":\"value\""));
    }
}

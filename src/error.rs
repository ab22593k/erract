use std::error::Error as ErrorTrait;
use std::fmt;

use crate::{ErrorKind, ErrorStatus};

pub use self::builder::ErrorBuilder;

/// Core error type for the erract library.
///
/// This struct represents a single error with:
/// - An actionable [`ErrorKind`](crate::ErrorKind) describing what the caller can do
/// - An explicit [`ErrorStatus`](crate::ErrorStatus) describing retry semantics
/// - A human-readable message
/// - Optional operation name for debugging
/// - Key-value context for troubleshooting
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
    message: String,
    operation: Option<&'static str>,
    pub(crate) context: Vec<(std::borrow::Cow<'static, str>, String)>,
}

impl Error {
    /// Creates a permanent error that cannot be retried.
    #[inline]
    pub fn permanent(kind: ErrorKind, message: impl Into<String>) -> Self {
        Error {
            kind,
            status: ErrorStatus::Permanent,
            message: message.into(),
            operation: None,
            context: Vec::new(),
        }
    }

    /// Creates a temporary error that is safe to retry.
    #[inline]
    pub fn temporary(kind: ErrorKind, message: impl Into<String>) -> Self {
        Error {
            kind,
            status: ErrorStatus::Temporary,
            message: message.into(),
            operation: None,
            context: Vec::new(),
        }
    }

    /// Creates a persistent error that was already retried.
    #[inline]
    pub fn persistent(kind: ErrorKind, message: impl Into<String>) -> Self {
        Error {
            kind,
            status: ErrorStatus::Persistent,
            message: message.into(),
            operation: None,
            context: Vec::new(),
        }
    }

    /// Creates an error with a specific status.
    #[inline]
    pub fn new(kind: ErrorKind, status: ErrorStatus, message: impl Into<String>) -> Self {
        Error {
            kind,
            status,
            message: message.into(),
            operation: None,
            context: Vec::new(),
        }
    }

    /// Returns a builder for configuring this error.
    #[inline]
    pub fn builder(
        kind: ErrorKind,
        status: ErrorStatus,
        message: impl Into<String>,
    ) -> ErrorBuilder {
        ErrorBuilder::new(kind, status, message)
    }

    /// Returns a reference to the error kind.
    #[inline]
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// Returns a reference to the error status.
    #[inline]
    pub fn status(&self) -> &ErrorStatus {
        &self.status
    }

    /// Returns a reference to the error message.
    #[inline]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the operation name if set.
    #[inline]
    pub fn operation(&self) -> Option<&'static str> {
        self.operation
    }

    /// Returns a reference to the context key-value pairs.
    #[inline]
    pub fn context(&self) -> &[(std::borrow::Cow<'static, str>, String)] {
        &self.context
    }

    /// Returns `true` if this error is safe to retry.
    #[inline]
    pub fn is_retryable(&self) -> bool {
        self.status.is_retryable()
    }

    /// Returns `true` if this error is permanent.
    #[inline]
    pub fn is_permanent(&self) -> bool {
        self.status.is_permanent()
    }

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
        key: impl Into<std::borrow::Cow<'static, str>>,
        value: impl ToString,
    ) -> Self {
        self.context.push((key.into(), value.to_string()));
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(op) = self.operation {
            write!(f, " (operation: {op})")?;
        }
        if !self.context.is_empty() {
            write!(f, " [")?;
            let mut first = true;
            for (key, value) in &self.context {
                if !first {
                    write!(f, ", ")?;
                }
                write!(f, "{key}: {value}")?;
                first = false;
            }
            write!(f, "]")?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

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
    #[inline]
    pub fn to_machine_string(&self) -> String {
        let mut output = format!(
            "kind={};status={};message={}",
            self.kind.to_machine_string(),
            self.status.to_machine_string(),
            self.message
        );
        if let Some(op) = self.operation {
            output.push_str(&format!(";operation={op}"));
        }
        if !self.context.is_empty() {
            let ctx: Vec<String> = self
                .context
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect();
            output.push_str(&format!(";context=[{}]", ctx.join(",")));
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
    #[inline]
    pub fn to_json(&self) -> String {
        let mut json = format!(
            r#"{{"kind":"{}","status":"{}","message":"{}""#,
            self.kind.to_machine_string(),
            self.status.to_machine_string(),
            self.message.escape_debug().collect::<String>()
        );
        if let Some(op) = self.operation {
            json.push_str(&format!(r#","operation":"{op}""#));
        }
        if !self.context.is_empty() {
            let ctx: Vec<String> = self
                .context
                .iter()
                .map(|(k, v)| {
                    format!(
                        r#""{}":"{}""#,
                        k.escape_debug().collect::<String>(),
                        v.escape_debug().collect::<String>()
                    )
                })
                .collect();
            json.push_str(&format!(r#","context":{{{}}}"#, ctx.join(",")));
        }
        json.push('}');
        json
    }
}

mod builder {
    use std::fmt;

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
        pub fn new(kind: ErrorKind, status: ErrorStatus, message: impl Into<String>) -> Self {
            ErrorBuilder {
                error: Error {
                    kind,
                    status,
                    message: message.into(),
                    operation: None,
                    context: Vec::new(),
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
            key: impl Into<std::borrow::Cow<'static, str>>,
            value: impl ToString,
        ) -> Self {
            self.error.context.push((key.into(), value.to_string()));
            self
        }

        /// Adds multiple context key-value pairs.
        #[inline]
        #[must_use]
        pub fn with_context_iter<K, V>(mut self, iter: impl IntoIterator<Item = (K, V)>) -> Self
        where
            K: Into<std::borrow::Cow<'static, str>>,
            V: ToString,
        {
            for (key, value) in iter {
                self.error.context.push((key.into(), value.to_string()));
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
            .with_context("user_id", 123)
            .with_context("operation", "lookup");

        assert_eq!(error.context().len(), 2);
        assert_eq!(error.context()[0].0, "user_id");
        assert_eq!(error.context()[0].1, "123");
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
}

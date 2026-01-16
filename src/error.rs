use std::borrow::Cow;
use std::error::Error as ErrorTrait;
use std::fmt::{self, Write};
use std::sync::Arc;

use crate::{ErrorKind, ErrorStatus};

pub use self::builder::ErrorBuilder;

/// Type alias for context storage.
pub type ContextVec = smallvec::SmallVec<[(Cow<'static, str>, Cow<'static, str>); 1]>;

/// Core error type for the erract library.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Error {
    kind: ErrorKind,
    status: ErrorStatus,
    message: Cow<'static, str>,
    operation: Option<&'static str>,
    pub(crate) context: crate::arena::ContextHandle,
    #[cfg_attr(feature = "serde", serde(skip, default))]
    source: Option<Arc<dyn std::error::Error + Send + Sync + 'static>>,
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.status == other.status
            && self.message == other.message
            && self.operation == other.operation
            && matches!(
                (&self.context, &other.context),
                (
                    crate::arena::ContextHandle::Empty,
                    crate::arena::ContextHandle::Empty
                )
            )
    }
}

impl Eq for Error {}

impl Error {
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

    /// Creates a permanent error with a static message (zero allocation).
    #[inline]
    pub fn permanent_static(kind: ErrorKind, message: &'static str) -> Self {
        Error {
            kind,
            status: ErrorStatus::Permanent,
            message: Cow::Borrowed(message),
            operation: None,
            context: crate::arena::ContextHandle::Empty,
            source: None,
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
            context: crate::arena::ContextHandle::Empty,
            source: None,
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
            context: crate::arena::ContextHandle::Empty,
            source: None,
        }
    }

    /// Creates a permanent error that cannot be retried.
    #[inline]
    pub fn permanent(kind: ErrorKind, message: impl Into<Cow<'static, str>>) -> Self {
        Error {
            kind,
            status: ErrorStatus::Permanent,
            message: message.into(),
            operation: None,
            context: crate::arena::ContextHandle::Empty,
            source: None,
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
            context: crate::arena::ContextHandle::Empty,
            source: None,
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
            context: crate::arena::ContextHandle::Empty,
            source: None,
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
            context: crate::arena::ContextHandle::Empty,
            source: None,
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

    /// Returns the context key-value pairs.
    pub fn context(&self) -> Vec<(Cow<'static, str>, Cow<'static, str>)> {
        match &self.context {
            crate::arena::ContextHandle::Heap(v) => v.to_vec(),
            crate::arena::ContextHandle::Arena {
                offset,
                len,
                thread_id,
            } => {
                if *thread_id == crate::arena::current_thread_id() {
                    crate::arena::with_arena(|arena| arena.get_pairs(*offset, *len))
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new(),
        }
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

    /// Returns an iterator over context key-value pairs.
    pub fn iter_context(&self) -> Vec<(String, String)> {
        self.context()
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    /// Sets the operation name for this error.
    #[inline]
    #[must_use]
    pub fn with_operation(mut self, operation: &'static str) -> Self {
        self.operation = Some(operation);
        self
    }

    /// Sets the source error.
    #[inline]
    #[must_use]
    pub fn with_source<E>(mut self, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        self.source = Some(Arc::new(source));
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
        let mut v = self.context();
        v.push((key.into(), value.into()));
        if v.len() > 1 {
            self.context = crate::arena::commit_to_arena(&v);
        } else {
            let mut cv = crate::arena::ContextVec::new();
            for p in v {
                cv.push(p);
            }
            self.context = crate::arena::ContextHandle::Heap(Box::new(cv));
        }
        self
    }

    /// Adds a key-value pair where value is converted via ToString.
    #[inline]
    #[must_use]
    pub fn with_context_value(
        self,
        key: impl Into<Cow<'static, str>>,
        value: impl ToString,
    ) -> Self {
        self.with_context(key, Cow::Owned(value.to_string()))
    }

    /// Converts this error into an Exn for context-aware propagation.
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
        f.write_str(&self.message)?;
        if let Some(op) = self.operation {
            f.write_str(" (operation: ")?;
            f.write_str(op)?;
            f.write_char(')')?;
        }
        let context = self.context();
        if !context.is_empty() {
            f.write_str(" [")?;
            let mut first = true;
            for (key, value) in &context {
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
        self.source
            .as_deref()
            .map(|e| e as &(dyn std::error::Error + 'static))
    }
}

impl Default for Error {
    fn default() -> Self {
        Error::unexpected()
    }
}

impl Error {
    /// Returns a machine-readable string representation of this error.
    pub fn to_machine_string(&self) -> String {
        let context = self.context();
        let capacity = 64 + self.message.len() + context.len() * 32;
        let mut output = String::with_capacity(capacity);

        output.push_str("kind=");
        output.push_str(&self.kind.to_machine_string());
        output.push_str(";status=");
        output.push_str(self.status.to_machine_string());
        output.push_str(";message=");
        output.push_str(&self.message);

        if let Some(op) = self.operation {
            output.push_str(";operation=");
            output.push_str(op);
        }

        if !context.is_empty() {
            output.push_str(";context=[");
            let mut first = true;
            for (k, v) in &context {
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
    pub fn to_json(&self) -> String {
        let context = self.context();
        let capacity = 128 + self.message.len() + context.len() * 48;
        let mut json = String::with_capacity(capacity);

        json.push_str(r#"{"kind":""#);
        json.push_str(&self.kind.to_machine_string());
        json.push_str(r#"","status":""#);
        json.push_str(self.status.to_machine_string());
        json.push_str(r#"","message":""#);
        write_escaped(&mut json, &self.message);
        json.push('"');

        if let Some(op) = self.operation {
            json.push_str(r#","operation":""#);
            json.push_str(op);
            json.push('"');
        }

        if !context.is_empty() {
            json.push_str(r#","context":{"#);
            let mut first = true;
            for (k, v) in &context {
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
    pub fn write_json(&self, buf: &mut String) {
        let context = self.context();
        buf.push_str(r#"{"kind":""#);
        buf.push_str(&self.kind.to_machine_string());
        buf.push_str(r#"","status":""#);
        buf.push_str(self.status.to_machine_string());
        buf.push_str(r#"","message":""#);
        write_escaped(buf, &self.message);
        buf.push('"');

        if let Some(op) = self.operation {
            buf.push_str(r#","operation":""#);
            buf.push_str(op);
            buf.push('"');
        }

        if !context.is_empty() {
            buf.push_str(r#","context":{"#);
            let mut first = true;
            for (k, v) in &context {
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
                let _ = write!(buf, r#"\u{:04x}"#, c as u32);
            }
            c => buf.push(c),
        }
    }
}

mod builder {
    use super::{Error, ErrorKind, ErrorStatus};
    use std::borrow::Cow;
    use std::fmt;
    use std::sync::Arc;

    /// Builder for configuring [`Error`] with additional context.
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
                    context: crate::arena::ContextHandle::Empty,
                    source: None,
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

        /// Sets the source error.
        #[inline]
        #[must_use]
        pub fn with_source<E>(mut self, source: E) -> Self
        where
            E: std::error::Error + Send + Sync + 'static,
        {
            self.error.source = Some(Arc::new(source));
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
            self.error = self.error.with_context(key, value);
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
            self.error = self.error.with_context_value(key, value);
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
                self.error = self.error.with_context(key, value);
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
        assert_eq!(error.kind(), &ErrorKind::NotFound);
        assert_eq!(error.message(), "not found");
    }

    #[test]
    fn test_context() {
        let error = Error::permanent(ErrorKind::NotFound, "not found")
            .with_context("user_id", "123")
            .with_context("operation", "lookup");

        assert_eq!(error.context().len(), 2);
    }

    #[test]
    fn test_memory_size() {
        let size = std::mem::size_of::<Error>();
        assert!(size <= 160, "Error size {size} exceeds 160 bytes");
    }
}

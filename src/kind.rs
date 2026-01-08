use std::borrow::Cow;
use std::fmt;

#[cfg(feature = "http")]
use super::http::HttpErrorKind;

#[cfg(feature = "db")]
use super::db::DatabaseErrorKind;

#[cfg(feature = "storage")]
use super::storage::StorageErrorKind;

/// Domain-specific error categorization by **action**, not origin.
///
/// Errors are categorized by what the caller should do when encountering them:
/// - Retry immediately
/// - Retry with backoff
/// - Fail immediately
/// - Return a specific error to the user
///
/// This is different from `thiserror` which categorizes by where the error came from.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Resource was not found.
    /// Don't retry - the resource doesn't exist.
    NotFound,
    /// Permission was denied.
    /// Won't succeed on retry - fix permissions first.
    PermissionDenied,
    /// Operation timed out.
    /// Safe to retry if not already persisted.
    Timeout,
    /// Input validation failed.
    /// Won't fix on retry - fix the input.
    Validation,
    /// An unexpected/unknown error occurred.
    /// May or may not be retryable depending on context.
    Unexpected,

    #[cfg(feature = "http")]
    /// HTTP-related error.
    Http(HttpErrorKind),

    #[cfg(feature = "db")]
    /// Database-related error.
    Database(DatabaseErrorKind),

    #[cfg(feature = "storage")]
    /// Storage-related error.
    Storage(StorageErrorKind),
}

impl ErrorKind {
    /// Returns `true` if this error kind represents a retryable condition.
    ///
    /// Note: This is a default implementation. In production, you may want
    /// to make more nuanced decisions based on specific error kinds.
    #[inline]
    pub fn is_retryable(&self) -> bool {
        match self {
            ErrorKind::NotFound => false,
            ErrorKind::PermissionDenied => false,
            ErrorKind::Timeout => true,
            ErrorKind::Validation => false,
            ErrorKind::Unexpected => false,
            #[cfg(feature = "http")]
            ErrorKind::Http(k) => k.is_retryable(),
            #[cfg(feature = "db")]
            ErrorKind::Database(k) => k.is_retryable(),
            #[cfg(feature = "storage")]
            ErrorKind::Storage(k) => k.is_retryable(),
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::NotFound => write!(f, "not found"),
            ErrorKind::PermissionDenied => write!(f, "permission denied"),
            ErrorKind::Timeout => write!(f, "timeout"),
            ErrorKind::Validation => write!(f, "validation error"),
            ErrorKind::Unexpected => write!(f, "unexpected error"),
            #[cfg(feature = "http")]
            ErrorKind::Http(k) => write!(f, "http error: {k}"),
            #[cfg(feature = "db")]
            ErrorKind::Database(k) => write!(f, "database error: {k}"),
            #[cfg(feature = "storage")]
            ErrorKind::Storage(k) => write!(f, "storage error: {k}"),
        }
    }
}

impl ErrorKind {
    /// Returns a machine-readable string representation of this error kind.
    ///
    /// The format uses underscores instead of spaces for easy parsing.
    ///
    /// # Examples
    ///
    /// ```
    /// use erract::ErrorKind;
    ///
    /// assert_eq!(ErrorKind::NotFound.to_machine_string(), "not_found");
    /// assert_eq!(ErrorKind::PermissionDenied.to_machine_string(), "permission_denied");
    /// assert_eq!(ErrorKind::Timeout.to_machine_string(), "timeout");
    /// ```
    #[inline]
    pub fn to_machine_string(&self) -> Cow<'static, str> {
        match self {
            ErrorKind::NotFound => Cow::Borrowed("not_found"),
            ErrorKind::PermissionDenied => Cow::Borrowed("permission_denied"),
            ErrorKind::Timeout => Cow::Borrowed("timeout"),
            ErrorKind::Validation => Cow::Borrowed("validation_error"),
            ErrorKind::Unexpected => Cow::Borrowed("unexpected_error"),
            #[cfg(feature = "http")]
            ErrorKind::Http(k) => Cow::Owned(format!("http_{}", k.to_machine_string())),
            #[cfg(feature = "db")]
            ErrorKind::Database(k) => Cow::Owned(format!("database_{}", k.to_machine_string())),
            #[cfg(feature = "storage")]
            ErrorKind::Storage(k) => Cow::Owned(format!("storage_{}", k.to_machine_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_is_not_retryable() {
        assert!(!ErrorKind::NotFound.is_retryable());
    }

    #[test]
    fn test_permission_denied_is_not_retryable() {
        assert!(!ErrorKind::PermissionDenied.is_retryable());
    }

    #[test]
    fn test_timeout_is_retryable() {
        assert!(ErrorKind::Timeout.is_retryable());
    }

    #[test]
    fn test_validation_is_not_retryable() {
        assert!(!ErrorKind::Validation.is_retryable());
    }

    #[test]
    fn test_unexpected_is_not_retryable() {
        assert!(!ErrorKind::Unexpected.is_retryable());
    }

    #[test]
    fn test_display() {
        assert_eq!(ErrorKind::NotFound.to_string(), "not found");
        assert_eq!(ErrorKind::PermissionDenied.to_string(), "permission denied");
        assert_eq!(ErrorKind::Timeout.to_string(), "timeout");
        assert_eq!(ErrorKind::Validation.to_string(), "validation error");
        assert_eq!(ErrorKind::Unexpected.to_string(), "unexpected error");
    }
}

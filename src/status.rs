use std::fmt;

/// Explicit retry semantics for errors.
///
/// Errors are categorized by whether they can be safely retried.
/// This eliminates guesswork when implementing retry logic.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorStatus {
    /// The error is permanent and retrying won't help.
    /// Examples: NotFound, PermissionDenied, Validation errors
    Permanent,
    /// The error is temporary and safe to retry.
    /// Examples: Network timeouts, rate limiting, temporary unavailability
    Temporary,
    /// The error was retried but is still failing.
    /// Use this when you've already attempted recovery.
    Persistent,
}

impl ErrorStatus {
    /// Returns `true` if the error is safe to retry.
    #[inline]
    pub fn is_retryable(&self) -> bool {
        matches!(self, ErrorStatus::Temporary)
    }

    /// Returns `true` if the error is permanent and retrying won't help.
    #[inline]
    pub fn is_permanent(&self) -> bool {
        matches!(self, ErrorStatus::Permanent)
    }

    /// Returns `true` if the error persisted after retries.
    #[inline]
    pub fn is_persistent(&self) -> bool {
        matches!(self, ErrorStatus::Persistent)
    }
}

impl fmt::Display for ErrorStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorStatus::Permanent => write!(f, "permanent"),
            ErrorStatus::Temporary => write!(f, "temporary"),
            ErrorStatus::Persistent => write!(f, "persistent"),
        }
    }
}

impl From<ErrorStatus> for bool {
    #[inline]
    fn from(status: ErrorStatus) -> Self {
        status.is_retryable()
    }
}

impl ErrorStatus {
    /// Returns a machine-readable string representation of this error status.
    ///
    /// # Examples
    ///
    /// ```
    /// use erract::ErrorStatus;
    ///
    /// assert_eq!(ErrorStatus::Permanent.to_machine_string(), "permanent");
    /// assert_eq!(ErrorStatus::Temporary.to_machine_string(), "temporary");
    /// assert_eq!(ErrorStatus::Persistent.to_machine_string(), "persistent");
    /// ```
    #[inline]
    pub fn to_machine_string(&self) -> String {
        match self {
            ErrorStatus::Permanent => "permanent".to_string(),
            ErrorStatus::Temporary => "temporary".to_string(),
            ErrorStatus::Persistent => "persistent".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permanent_is_not_retryable() {
        let status = ErrorStatus::Permanent;
        assert!(!status.is_retryable());
        assert!(status.is_permanent());
        assert!(!status.is_persistent());
    }

    #[test]
    fn test_temporary_is_retryable() {
        let status = ErrorStatus::Temporary;
        assert!(status.is_retryable());
        assert!(!status.is_permanent());
        assert!(!status.is_persistent());
    }

    #[test]
    fn test_persistent_is_not_retryable() {
        let status = ErrorStatus::Persistent;
        assert!(!status.is_retryable());
        assert!(!status.is_permanent());
        assert!(status.is_persistent());
    }

    #[test]
    fn test_display() {
        assert_eq!(ErrorStatus::Permanent.to_string(), "permanent");
        assert_eq!(ErrorStatus::Temporary.to_string(), "temporary");
        assert_eq!(ErrorStatus::Persistent.to_string(), "persistent");
    }
}

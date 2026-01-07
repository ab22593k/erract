use std::fmt;

/// Database-specific error kinds.
///
/// These errors categorize database-related failures by what the caller should do.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DatabaseErrorKind {
    /// Failed to establish database connection.
    /// May be temporary - safe to retry.
    ConnectionFailed,
    /// Lost database connection mid-operation.
    /// May be temporary - safe to retry.
    ConnectionLost,
    /// Query syntax error (SQL syntax).
    /// Permanent - fix the query.
    QuerySyntax,
    /// Query execution error (e.g., invalid parameters).
    /// May be permanent or retryable depending on cause.
    QueryExecution,
    /// Constraint violation (e.g., unique constraint, foreign key).
    /// Permanent - fix the data or schema.
    ConstraintViolation,
    /// Deadlock detected.
    /// Temporary - safe to retry with backoff.
    Deadlock,
    /// Transaction serialization failure.
    /// Temporary - safe to retry with backoff.
    SerializationFailure,
    /// Transaction timeout.
    /// Temporary - safe to retry with longer timeout.
    TransactionTimeout,
    /// Transaction already in progress (nested transaction error).
    /// Permanent - fix transaction management.
    NestedTransaction,
    /// No rows returned when rows were expected.
    /// May be permanent - fix the query or handle None case.
    NoRows,
    /// Too many rows returned.
    /// Permanent - fix the query to limit results.
    TooManyRows,
    /// Result set conversion error (type mismatch).
    /// Permanent - fix the type mapping.
    TypeMismatch,
    /// Database schema mismatch.
    /// Permanent - migrate the database.
    SchemaMismatch,
    /// Database locked (e.g., SQLite).
    /// Temporary - safe to retry with backoff.
    DatabaseLocked,
    /// Disk full or quota exceeded.
    /// Permanent - free up space or increase quota.
    DiskFull,
    /// Permission denied for the operation.
    /// Permanent - fix permissions.
    PermissionDenied,
    /// Database is in readonly mode.
    /// Permanent - check database configuration.
    ReadOnly,
}

impl DatabaseErrorKind {
    /// Returns `true` if this database error kind represents a retryable condition.
    #[inline]
    pub fn is_retryable(&self) -> bool {
        match self {
            DatabaseErrorKind::ConnectionFailed => true,
            DatabaseErrorKind::ConnectionLost => true,
            DatabaseErrorKind::QuerySyntax => false,
            DatabaseErrorKind::QueryExecution => false,
            DatabaseErrorKind::ConstraintViolation => false,
            DatabaseErrorKind::Deadlock => true,
            DatabaseErrorKind::SerializationFailure => true,
            DatabaseErrorKind::TransactionTimeout => true,
            DatabaseErrorKind::NestedTransaction => false,
            DatabaseErrorKind::NoRows => false,
            DatabaseErrorKind::TooManyRows => false,
            DatabaseErrorKind::TypeMismatch => false,
            DatabaseErrorKind::SchemaMismatch => false,
            DatabaseErrorKind::DatabaseLocked => true,
            DatabaseErrorKind::DiskFull => false,
            DatabaseErrorKind::PermissionDenied => false,
            DatabaseErrorKind::ReadOnly => false,
        }
    }

    /// Returns `true` if this is a connection-related error.
    #[inline]
    pub fn is_connection_error(&self) -> bool {
        matches!(
            self,
            DatabaseErrorKind::ConnectionFailed | DatabaseErrorKind::ConnectionLost
        )
    }

    /// Returns `true` if this is a query-related error.
    #[inline]
    pub fn is_query_error(&self) -> bool {
        matches!(
            self,
            DatabaseErrorKind::QuerySyntax
                | DatabaseErrorKind::QueryExecution
                | DatabaseErrorKind::TypeMismatch
        )
    }

    /// Returns `true` if this is a transaction-related error.
    #[inline]
    pub fn is_transaction_error(&self) -> bool {
        matches!(
            self,
            DatabaseErrorKind::Deadlock
                | DatabaseErrorKind::SerializationFailure
                | DatabaseErrorKind::TransactionTimeout
                | DatabaseErrorKind::NestedTransaction
        )
    }

    /// Returns `true` if this is a data-related error.
    #[inline]
    pub fn is_data_error(&self) -> bool {
        matches!(
            self,
            DatabaseErrorKind::ConstraintViolation
                | DatabaseErrorKind::NoRows
                | DatabaseErrorKind::TooManyRows
        )
    }

    /// Returns `true` if this is a configuration-related error.
    #[inline]
    pub fn is_configuration_error(&self) -> bool {
        matches!(
            self,
            DatabaseErrorKind::SchemaMismatch
                | DatabaseErrorKind::ReadOnly
                | DatabaseErrorKind::PermissionDenied
        )
    }

    /// Returns a category description for this error.
    #[inline]
    pub fn category(&self) -> &str {
        if self.is_connection_error() {
            "Connection"
        } else if self.is_query_error() {
            "Query"
        } else if self.is_transaction_error() {
            "Transaction"
        } else if self.is_data_error() {
            "Data"
        } else if self.is_configuration_error() {
            "Configuration"
        } else {
            "System"
        }
    }

    /// Returns a machine-readable string representation of this database error kind.
    #[inline]
    pub fn to_machine_string(&self) -> String {
        match self {
            DatabaseErrorKind::ConnectionFailed => "connection_failed".to_string(),
            DatabaseErrorKind::ConnectionLost => "connection_lost".to_string(),
            DatabaseErrorKind::QuerySyntax => "query_syntax".to_string(),
            DatabaseErrorKind::QueryExecution => "query_execution".to_string(),
            DatabaseErrorKind::ConstraintViolation => "constraint_violation".to_string(),
            DatabaseErrorKind::Deadlock => "deadlock".to_string(),
            DatabaseErrorKind::SerializationFailure => "serialization_failure".to_string(),
            DatabaseErrorKind::TransactionTimeout => "transaction_timeout".to_string(),
            DatabaseErrorKind::NestedTransaction => "nested_transaction".to_string(),
            DatabaseErrorKind::NoRows => "no_rows".to_string(),
            DatabaseErrorKind::TooManyRows => "too_many_rows".to_string(),
            DatabaseErrorKind::TypeMismatch => "type_mismatch".to_string(),
            DatabaseErrorKind::SchemaMismatch => "schema_mismatch".to_string(),
            DatabaseErrorKind::DatabaseLocked => "database_locked".to_string(),
            DatabaseErrorKind::DiskFull => "disk_full".to_string(),
            DatabaseErrorKind::PermissionDenied => "permission_denied".to_string(),
            DatabaseErrorKind::ReadOnly => "read_only".to_string(),
        }
    }
}

impl fmt::Display for DatabaseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseErrorKind::ConnectionFailed => write!(f, "connection failed"),
            DatabaseErrorKind::ConnectionLost => write!(f, "connection lost"),
            DatabaseErrorKind::QuerySyntax => write!(f, "query syntax error"),
            DatabaseErrorKind::QueryExecution => write!(f, "query execution error"),
            DatabaseErrorKind::ConstraintViolation => write!(f, "constraint violation"),
            DatabaseErrorKind::Deadlock => write!(f, "deadlock detected"),
            DatabaseErrorKind::SerializationFailure => write!(f, "serialization failure"),
            DatabaseErrorKind::TransactionTimeout => write!(f, "transaction timeout"),
            DatabaseErrorKind::NestedTransaction => write!(f, "nested transaction"),
            DatabaseErrorKind::NoRows => write!(f, "no rows returned"),
            DatabaseErrorKind::TooManyRows => write!(f, "too many rows returned"),
            DatabaseErrorKind::TypeMismatch => write!(f, "type mismatch"),
            DatabaseErrorKind::SchemaMismatch => write!(f, "schema mismatch"),
            DatabaseErrorKind::DatabaseLocked => write!(f, "database locked"),
            DatabaseErrorKind::DiskFull => write!(f, "disk full"),
            DatabaseErrorKind::PermissionDenied => write!(f, "permission denied"),
            DatabaseErrorKind::ReadOnly => write!(f, "database is read-only"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_failed_retryable() {
        assert!(DatabaseErrorKind::ConnectionFailed.is_retryable());
    }

    #[test]
    fn test_query_syntax_not_retryable() {
        assert!(!DatabaseErrorKind::QuerySyntax.is_retryable());
    }

    #[test]
    fn test_constraint_violation_not_retryable() {
        assert!(!DatabaseErrorKind::ConstraintViolation.is_retryable());
    }

    #[test]
    fn test_deadlock_retryable() {
        assert!(DatabaseErrorKind::Deadlock.is_retryable());
    }

    #[test]
    fn test_display() {
        assert_eq!(
            DatabaseErrorKind::ConnectionFailed.to_string(),
            "connection failed"
        );
        assert_eq!(
            DatabaseErrorKind::ConstraintViolation.to_string(),
            "constraint violation"
        );
        assert_eq!(DatabaseErrorKind::Deadlock.to_string(), "deadlock detected");
    }
}

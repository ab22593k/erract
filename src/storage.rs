use std::fmt;

/// Storage-specific error kinds.
///
/// These errors categorize storage-related failures by what the caller should do.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StorageErrorKind {
    /// File or resource not found.
    /// Permanent - the resource doesn't exist.
    NotFound,
    /// Directory not found.
    /// Permanent - the directory doesn't exist.
    DirectoryNotFound,
    /// Permission denied for the operation.
    /// Permanent - fix permissions.
    PermissionDenied,
    /// File already exists (when it shouldn't).
    /// Permanent - handle the conflict.
    AlreadyExists,
    /// Is a directory (expected a file).
    /// Permanent - fix the path.
    IsDirectory,
    /// Is not a directory (expected a directory).
    /// Permanent - fix the path.
    NotDirectory,
    /// Disk full or quota exceeded.
    /// Permanent - free up space or increase quota.
    DiskFull,
    /// Disk I/O error.
    /// May be temporary - safe to retry.
    IoError,
    /// File name too long.
    /// Permanent - shorten the name.
    FileNameTooLong,
    /// Path too long.
    /// Permanent - shorten the path.
    PathTooLong,
    /// Too many open files.
    /// May be temporary - retry after closing files.
    TooManyOpenFiles,
    /// Storage device is read-only.
    /// Permanent - check storage configuration.
    ReadOnly,
    /// Storage device is full.
    /// Permanent - free up space.
    StorageFull,
    /// Network storage connection error.
    /// May be temporary - safe to retry.
    NetworkError,
    /// Network storage timeout.
    /// May be temporary - safe to retry with longer timeout.
    NetworkTimeout,
    /// Invalid filename.
    /// Permanent - fix the filename.
    InvalidFilename,
    /// Invalid path.
    /// Permanent - fix the path.
    InvalidPath,
    /// Symlink loop detected.
    /// Permanent - fix the symlink configuration.
    SymlinkLoop,
    /// Too many symbolic links.
    /// Permanent - simplify the directory structure.
    TooManySymlinks,
}

impl StorageErrorKind {
    /// Returns `true` if this storage error kind represents a retryable condition.
    #[inline]
    pub fn is_retryable(&self) -> bool {
        match self {
            StorageErrorKind::NotFound => false,
            StorageErrorKind::DirectoryNotFound => false,
            StorageErrorKind::PermissionDenied => false,
            StorageErrorKind::AlreadyExists => false,
            StorageErrorKind::IsDirectory => false,
            StorageErrorKind::NotDirectory => false,
            StorageErrorKind::DiskFull => false,
            StorageErrorKind::IoError => true,
            StorageErrorKind::FileNameTooLong => false,
            StorageErrorKind::PathTooLong => false,
            StorageErrorKind::TooManyOpenFiles => true,
            StorageErrorKind::ReadOnly => false,
            StorageErrorKind::StorageFull => false,
            StorageErrorKind::NetworkError => true,
            StorageErrorKind::NetworkTimeout => true,
            StorageErrorKind::InvalidFilename => false,
            StorageErrorKind::InvalidPath => false,
            StorageErrorKind::SymlinkLoop => false,
            StorageErrorKind::TooManySymlinks => false,
        }
    }

    /// Returns `true` if this is a path-related error.
    #[inline]
    pub fn is_path_error(&self) -> bool {
        matches!(
            self,
            StorageErrorKind::NotFound
                | StorageErrorKind::DirectoryNotFound
                | StorageErrorKind::InvalidPath
                | StorageErrorKind::InvalidFilename
                | StorageErrorKind::IsDirectory
                | StorageErrorKind::NotDirectory
                | StorageErrorKind::SymlinkLoop
                | StorageErrorKind::TooManySymlinks
        )
    }

    /// Returns `true` if this is a permission-related error.
    #[inline]
    pub fn is_permission_error(&self) -> bool {
        matches!(
            self,
            StorageErrorKind::PermissionDenied | StorageErrorKind::ReadOnly
        )
    }

    /// Returns `true` if this is a capacity-related error.
    #[inline]
    pub fn is_capacity_error(&self) -> bool {
        matches!(
            self,
            StorageErrorKind::DiskFull
                | StorageErrorKind::StorageFull
                | StorageErrorKind::TooManyOpenFiles
        )
    }

    /// Returns `true` if this is a network storage error.
    #[inline]
    pub fn is_network_error(&self) -> bool {
        matches!(
            self,
            StorageErrorKind::NetworkError | StorageErrorKind::NetworkTimeout
        )
    }

    /// Returns `true` if this is an I/O error.
    #[inline]
    pub fn is_io_error(&self) -> bool {
        matches!(self, StorageErrorKind::IoError)
    }

    /// Returns a category description for this error.
    #[inline]
    pub fn category(&self) -> &str {
        if self.is_path_error() {
            "Path"
        } else if self.is_permission_error() {
            "Permission"
        } else if self.is_capacity_error() {
            "Capacity"
        } else if self.is_network_error() {
            "Network"
        } else if self.is_io_error() {
            "I/O"
        } else {
            "Other"
        }
    }

    /// Returns `true` if this error indicates the resource exists (when it shouldn't).
    #[inline]
    pub fn is_existence_error(&self) -> bool {
        matches!(
            self,
            StorageErrorKind::AlreadyExists
                | StorageErrorKind::NotFound
                | StorageErrorKind::DirectoryNotFound
        )
    }

    /// Returns a machine-readable string representation of this storage error kind.
    #[inline]
    pub fn to_machine_string(&self) -> String {
        match self {
            StorageErrorKind::NotFound => "not_found".to_string(),
            StorageErrorKind::DirectoryNotFound => "directory_not_found".to_string(),
            StorageErrorKind::PermissionDenied => "permission_denied".to_string(),
            StorageErrorKind::AlreadyExists => "already_exists".to_string(),
            StorageErrorKind::IsDirectory => "is_directory".to_string(),
            StorageErrorKind::NotDirectory => "not_directory".to_string(),
            StorageErrorKind::DiskFull => "disk_full".to_string(),
            StorageErrorKind::IoError => "io_error".to_string(),
            StorageErrorKind::FileNameTooLong => "file_name_too_long".to_string(),
            StorageErrorKind::PathTooLong => "path_too_long".to_string(),
            StorageErrorKind::TooManyOpenFiles => "too_many_open_files".to_string(),
            StorageErrorKind::ReadOnly => "read_only".to_string(),
            StorageErrorKind::StorageFull => "storage_full".to_string(),
            StorageErrorKind::NetworkError => "network_error".to_string(),
            StorageErrorKind::NetworkTimeout => "network_timeout".to_string(),
            StorageErrorKind::InvalidFilename => "invalid_filename".to_string(),
            StorageErrorKind::InvalidPath => "invalid_path".to_string(),
            StorageErrorKind::SymlinkLoop => "symlink_loop".to_string(),
            StorageErrorKind::TooManySymlinks => "too_many_symlinks".to_string(),
        }
    }
}

impl fmt::Display for StorageErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageErrorKind::NotFound => write!(f, "not found"),
            StorageErrorKind::DirectoryNotFound => write!(f, "directory not found"),
            StorageErrorKind::PermissionDenied => write!(f, "permission denied"),
            StorageErrorKind::AlreadyExists => write!(f, "already exists"),
            StorageErrorKind::IsDirectory => write!(f, "is a directory"),
            StorageErrorKind::NotDirectory => write!(f, "is not a directory"),
            StorageErrorKind::DiskFull => write!(f, "disk full"),
            StorageErrorKind::IoError => write!(f, "I/O error"),
            StorageErrorKind::FileNameTooLong => write!(f, "file name too long"),
            StorageErrorKind::PathTooLong => write!(f, "path too long"),
            StorageErrorKind::TooManyOpenFiles => write!(f, "too many open files"),
            StorageErrorKind::ReadOnly => write!(f, "read-only"),
            StorageErrorKind::StorageFull => write!(f, "storage full"),
            StorageErrorKind::NetworkError => write!(f, "network error"),
            StorageErrorKind::NetworkTimeout => write!(f, "network timeout"),
            StorageErrorKind::InvalidFilename => write!(f, "invalid filename"),
            StorageErrorKind::InvalidPath => write!(f, "invalid path"),
            StorageErrorKind::SymlinkLoop => write!(f, "symlink loop"),
            StorageErrorKind::TooManySymlinks => write!(f, "too many symbolic links"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_not_retryable() {
        assert!(!StorageErrorKind::NotFound.is_retryable());
    }

    #[test]
    fn test_io_error_retryable() {
        assert!(StorageErrorKind::IoError.is_retryable());
    }

    #[test]
    fn test_network_error_retryable() {
        assert!(StorageErrorKind::NetworkError.is_retryable());
    }

    #[test]
    fn test_permission_denied_not_retryable() {
        assert!(!StorageErrorKind::PermissionDenied.is_retryable());
    }

    #[test]
    fn test_display() {
        assert_eq!(StorageErrorKind::NotFound.to_string(), "not found");
        assert_eq!(StorageErrorKind::IoError.to_string(), "I/O error");
        assert_eq!(StorageErrorKind::DiskFull.to_string(), "disk full");
    }
}

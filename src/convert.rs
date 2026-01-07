use std::io;

use crate::{Error, ErrorKind, ErrorStatus};

impl From<io::Error> for Error {
    #[inline]
    fn from(err: io::Error) -> Self {
        let (kind, status) = match err.kind() {
            io::ErrorKind::NotFound => (ErrorKind::NotFound, ErrorStatus::Permanent),
            io::ErrorKind::PermissionDenied => {
                (ErrorKind::PermissionDenied, ErrorStatus::Permanent)
            }
            io::ErrorKind::ConnectionRefused => (ErrorKind::Unexpected, ErrorStatus::Temporary),
            io::ErrorKind::ConnectionReset => (ErrorKind::Unexpected, ErrorStatus::Temporary),
            io::ErrorKind::ConnectionAborted => (ErrorKind::Unexpected, ErrorStatus::Temporary),
            io::ErrorKind::NotConnected => (ErrorKind::Unexpected, ErrorStatus::Temporary),
            io::ErrorKind::AddrInUse => (ErrorKind::Unexpected, ErrorStatus::Temporary),
            io::ErrorKind::AddrNotAvailable => (ErrorKind::Unexpected, ErrorStatus::Permanent),
            io::ErrorKind::BrokenPipe => (ErrorKind::Unexpected, ErrorStatus::Temporary),
            io::ErrorKind::AlreadyExists => (ErrorKind::Unexpected, ErrorStatus::Permanent),
            io::ErrorKind::WouldBlock => (ErrorKind::Unexpected, ErrorStatus::Temporary),
            io::ErrorKind::InvalidInput => (ErrorKind::Validation, ErrorStatus::Permanent),
            io::ErrorKind::InvalidData => (ErrorKind::Validation, ErrorStatus::Permanent),
            io::ErrorKind::TimedOut => (ErrorKind::Timeout, ErrorStatus::Temporary),
            io::ErrorKind::WriteZero => (ErrorKind::Unexpected, ErrorStatus::Temporary),
            io::ErrorKind::Interrupted => (ErrorKind::Unexpected, ErrorStatus::Temporary),
            io::ErrorKind::Other => (ErrorKind::Unexpected, ErrorStatus::Temporary),
            io::ErrorKind::UnexpectedEof => (ErrorKind::Unexpected, ErrorStatus::Temporary),
            _ => (ErrorKind::Unexpected, ErrorStatus::Persistent),
        };
        Error::new(kind, status, err.to_string())
    }
}

impl From<std::str::Utf8Error> for Error {
    #[inline]
    fn from(err: std::str::Utf8Error) -> Self {
        Error::new(
            ErrorKind::Validation,
            ErrorStatus::Permanent,
            format!("UTF-8 conversion error: {err}"),
        )
    }
}

impl From<std::string::FromUtf8Error> for Error {
    #[inline]
    fn from(err: std::string::FromUtf8Error) -> Self {
        Error::new(
            ErrorKind::Validation,
            ErrorStatus::Permanent,
            format!("UTF-8 string conversion error: {err}"),
        )
    }
}

impl From<std::num::ParseIntError> for Error {
    #[inline]
    fn from(err: std::num::ParseIntError) -> Self {
        Error::new(
            ErrorKind::Validation,
            ErrorStatus::Permanent,
            format!("Integer parsing error: {err}"),
        )
    }
}

impl From<std::num::ParseFloatError> for Error {
    #[inline]
    fn from(err: std::num::ParseFloatError) -> Self {
        Error::new(
            ErrorKind::Validation,
            ErrorStatus::Permanent,
            format!("Float parsing error: {err}"),
        )
    }
}

impl From<std::fmt::Error> for Error {
    #[inline]
    fn from(err: std::fmt::Error) -> Self {
        Error::new(
            ErrorKind::Unexpected,
            ErrorStatus::Permanent,
            format!("Formatting error: {err}"),
        )
    }
}

impl From<std::array::TryFromSliceError> for Error {
    #[inline]
    fn from(err: std::array::TryFromSliceError) -> Self {
        Error::new(
            ErrorKind::Validation,
            ErrorStatus::Permanent,
            format!("Slice conversion error: {err}"),
        )
    }
}

impl From<std::net::AddrParseError> for Error {
    #[inline]
    fn from(err: std::net::AddrParseError) -> Self {
        Error::new(
            ErrorKind::Validation,
            ErrorStatus::Permanent,
            format!("Address parsing error: {err}"),
        )
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    #[inline]
    fn from(err: std::sync::PoisonError<T>) -> Self {
        Error::new(
            ErrorKind::Unexpected,
            ErrorStatus::Permanent,
            format!("Mutex poisoned: {err}"),
        )
    }
}

impl From<std::time::SystemTimeError> for Error {
    #[inline]
    fn from(err: std::time::SystemTimeError) -> Self {
        Error::new(
            ErrorKind::Unexpected,
            ErrorStatus::Temporary,
            format!("System time error: {err}"),
        )
    }
}

impl From<std::ffi::OsString> for Error {
    #[inline]
    fn from(err: std::ffi::OsString) -> Self {
        Error::new(
            ErrorKind::Validation,
            ErrorStatus::Permanent,
            format!("OS string conversion error: {err:?}"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_not_found() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        assert_eq!(err.kind(), &ErrorKind::NotFound);
        assert!(err.is_permanent());
    }

    #[test]
    fn test_io_error_timeout() {
        let io_err = io::Error::new(io::ErrorKind::TimedOut, "connection timeout");
        let err: Error = io_err.into();
        assert_eq!(err.kind(), &ErrorKind::Timeout);
        assert!(err.is_retryable());
    }

    #[test]
    fn test_io_error_permission_denied() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let err: Error = io_err.into();
        assert_eq!(err.kind(), &ErrorKind::PermissionDenied);
        assert!(err.is_permanent());
    }

    #[test]
    fn test_parse_int_error() {
        let parse_err: std::num::ParseIntError = "abc".parse::<u32>().unwrap_err();
        let err: Error = parse_err.into();
        assert_eq!(err.kind(), &ErrorKind::Validation);
        assert!(err.is_permanent());
    }

    #[test]
    fn test_addr_parse_error() {
        let parse_err: std::net::AddrParseError =
            "invalid".parse::<std::net::SocketAddr>().unwrap_err();
        let err: Error = parse_err.into();
        assert_eq!(err.kind(), &ErrorKind::Validation);
        assert!(err.is_permanent());
    }
}

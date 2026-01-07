use std::fmt;

/// HTTP-specific error kinds.
///
/// These errors categorize HTTP-related failures by what the caller should do.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HttpErrorKind {
    /// Client error (4xx status codes).
    /// Don't retry - the request is malformed or unauthorized.
    ClientError(u16),
    /// Server error (5xx status codes).
    /// May be temporary - safe to retry with backoff.
    ServerError(u16),
    /// Rate limited by the server.
    /// Slow down and retry with backoff.
    RateLimited,
    /// Network connectivity error.
    /// May be temporary - safe to retry.
    NetworkError,
    /// SSL/TLS handshake error.
    /// May be temporary - safe to retry.
    TlsError,
    /// Invalid URL or URL parsing error.
    /// Permanent - fix the URL.
    InvalidUrl,
    /// Redirect loop detected.
    /// Permanent - fix the redirect configuration.
    RedirectLoop,
    /// Too many redirects.
    /// Permanent - fix the redirect configuration.
    TooManyRedirects,
    /// Request timeout.
    /// Safe to retry with longer timeout.
    RequestTimeout,
    /// Content encoding error.
    /// May be temporary - safe to retry.
    EncodingError,
    /// Decoding error (e.g., invalid JSON).
    /// Permanent - fix the response handling.
    DecodingError,
}

impl HttpErrorKind {
    /// Returns `true` if this HTTP error kind represents a retryable condition.
    #[inline]
    pub fn is_retryable(&self) -> bool {
        match self {
            HttpErrorKind::ClientError(_) => false,
            HttpErrorKind::ServerError(_) => true,
            HttpErrorKind::RateLimited => true,
            HttpErrorKind::NetworkError => true,
            HttpErrorKind::TlsError => true,
            HttpErrorKind::InvalidUrl => false,
            HttpErrorKind::RedirectLoop => false,
            HttpErrorKind::TooManyRedirects => false,
            HttpErrorKind::RequestTimeout => true,
            HttpErrorKind::EncodingError => true,
            HttpErrorKind::DecodingError => false,
        }
    }

    /// Returns the HTTP status code if applicable.
    #[inline]
    pub fn status_code(&self) -> Option<u16> {
        match self {
            HttpErrorKind::ClientError(code) | HttpErrorKind::ServerError(code) => Some(*code),
            _ => None,
        }
    }

    /// Creates an `HttpErrorKind` from an HTTP status code.
    ///
    /// # Examples
    ///
    /// ```
    /// use erract::http::HttpErrorKind;
    ///
    /// let not_found = HttpErrorKind::from_status(404);
    /// assert_eq!(not_found.status_code(), Some(404));
    ///
    /// let server_error = HttpErrorKind::from_status(500);
    /// assert_eq!(server_error.status_code(), Some(500));
    /// ```
    #[inline]
    pub fn from_status(status: u16) -> Self {
        match status {
            429 => Self::RateLimited,
            400..=499 => Self::ClientError(status),
            500..=599 => Self::ServerError(status),
            _ if status >= 400 => Self::ClientError(status),
            _ => Self::ServerError(status),
        }
    }

    /// Returns `true` if this is a 4xx client error.
    #[inline]
    pub fn is_client_error(&self) -> bool {
        matches!(self, HttpErrorKind::ClientError(_))
    }

    /// Returns `true` if this is a 5xx server error.
    #[inline]
    pub fn is_server_error(&self) -> bool {
        matches!(self, HttpErrorKind::ServerError(_))
    }

    /// Returns `true` if this is a 4xx or 5xx error.
    #[inline]
    pub fn is_error(&self) -> bool {
        self.status_code().is_some_and(|s| s >= 400)
    }

    /// Returns `true` if this is a 2xx success.
    #[inline]
    pub fn is_success(&self) -> bool {
        false // Error kinds are always errors
    }

    /// Returns a human-readable description of the status code range.
    #[inline]
    pub fn status_range_description(&self) -> &str {
        match self {
            HttpErrorKind::ClientError(_) => "Client Error (4xx)",
            HttpErrorKind::ServerError(_) => "Server Error (5xx)",
            HttpErrorKind::RateLimited => "Rate Limited (429)",
            HttpErrorKind::NetworkError => "Network Error",
            HttpErrorKind::TlsError => "TLS/SSL Error",
            HttpErrorKind::InvalidUrl => "Invalid URL",
            HttpErrorKind::RedirectLoop => "Redirect Loop",
            HttpErrorKind::TooManyRedirects => "Too Many Redirects",
            HttpErrorKind::RequestTimeout => "Request Timeout",
            HttpErrorKind::EncodingError => "Encoding Error",
            HttpErrorKind::DecodingError => "Decoding Error",
        }
    }

    /// Returns a machine-readable string representation of this HTTP error kind.
    #[inline]
    pub fn to_machine_string(&self) -> String {
        match self {
            HttpErrorKind::ClientError(code) => format!("client_error_{code}"),
            HttpErrorKind::ServerError(code) => format!("server_error_{code}"),
            HttpErrorKind::RateLimited => "rate_limited".to_string(),
            HttpErrorKind::NetworkError => "network_error".to_string(),
            HttpErrorKind::TlsError => "tls_error".to_string(),
            HttpErrorKind::InvalidUrl => "invalid_url".to_string(),
            HttpErrorKind::RedirectLoop => "redirect_loop".to_string(),
            HttpErrorKind::TooManyRedirects => "too_many_redirects".to_string(),
            HttpErrorKind::RequestTimeout => "request_timeout".to_string(),
            HttpErrorKind::EncodingError => "encoding_error".to_string(),
            HttpErrorKind::DecodingError => "decoding_error".to_string(),
        }
    }
}

impl fmt::Display for HttpErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpErrorKind::ClientError(code) => write!(f, "client error: {code}"),
            HttpErrorKind::ServerError(code) => write!(f, "server error: {code}"),
            HttpErrorKind::RateLimited => write!(f, "rate limited"),
            HttpErrorKind::NetworkError => write!(f, "network error"),
            HttpErrorKind::TlsError => write!(f, "TLS error"),
            HttpErrorKind::InvalidUrl => write!(f, "invalid URL"),
            HttpErrorKind::RedirectLoop => write!(f, "redirect loop"),
            HttpErrorKind::TooManyRedirects => write!(f, "too many redirects"),
            HttpErrorKind::RequestTimeout => write!(f, "request timeout"),
            HttpErrorKind::EncodingError => write!(f, "encoding error"),
            HttpErrorKind::DecodingError => write!(f, "decoding error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_error_not_retryable() {
        let kind = HttpErrorKind::ClientError(400);
        assert!(!kind.is_retryable());
        assert_eq!(kind.status_code(), Some(400));
    }

    #[test]
    fn test_server_error_retryable() {
        let kind = HttpErrorKind::ServerError(500);
        assert!(kind.is_retryable());
        assert_eq!(kind.status_code(), Some(500));
    }

    #[test]
    fn test_rate_limited_retryable() {
        let kind = HttpErrorKind::RateLimited;
        assert!(kind.is_retryable());
        assert_eq!(kind.status_code(), None);
    }

    #[test]
    fn test_display() {
        assert_eq!(
            HttpErrorKind::ClientError(400).to_string(),
            "client error: 400"
        );
        assert_eq!(
            HttpErrorKind::ServerError(500).to_string(),
            "server error: 500"
        );
        assert_eq!(HttpErrorKind::RateLimited.to_string(), "rate limited");
        assert_eq!(HttpErrorKind::NetworkError.to_string(), "network error");
    }
}

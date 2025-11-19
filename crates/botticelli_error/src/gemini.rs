//! Gemini-specific error types and retry logic.

/// Gemini-specific error conditions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GeminiErrorKind {
    /// API key not found in environment
    MissingApiKey,
    /// Failed to create Gemini client
    ClientCreation(String),
    /// API request failed
    ApiRequest(String),
    /// HTTP error with status code and message
    HttpError {
        /// HTTP status code
        status_code: u16,
        /// Error message
        message: String,
    },
    /// Multimodal inputs not yet supported
    MultimodalNotSupported,
    /// URL media sources not yet supported
    UrlMediaNotSupported,
    /// Base64 decoding failed
    Base64Decode(String),
    /// WebSocket connection failed
    WebSocketConnection(String),
    /// WebSocket handshake failed (setup phase)
    WebSocketHandshake(String),
    /// Invalid message received from server
    InvalidServerMessage(String),
    /// Server sent goAway message
    ServerDisconnect(String),
    /// Stream was interrupted
    StreamInterrupted(String),
}

impl std::fmt::Display for GeminiErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeminiErrorKind::MissingApiKey => {
                write!(f, "GEMINI_API_KEY environment variable not set")
            }
            GeminiErrorKind::ClientCreation(msg) => {
                write!(f, "Failed to create Gemini client: {}", msg)
            }
            GeminiErrorKind::ApiRequest(msg) => write!(f, "Gemini API request failed: {}", msg),
            GeminiErrorKind::HttpError {
                status_code,
                message,
            } => write!(f, "HTTP {} error: {}", status_code, message),
            GeminiErrorKind::MultimodalNotSupported => write!(
                f,
                "Multimodal inputs not yet supported in simple Gemini wrapper"
            ),
            GeminiErrorKind::UrlMediaNotSupported => {
                write!(f, "URL media sources not yet supported for Gemini")
            }
            GeminiErrorKind::Base64Decode(msg) => write!(f, "Base64 decode error: {}", msg),
            GeminiErrorKind::WebSocketConnection(msg) => {
                write!(f, "WebSocket connection failed: {}", msg)
            }
            GeminiErrorKind::WebSocketHandshake(msg) => {
                write!(f, "WebSocket handshake failed: {}", msg)
            }
            GeminiErrorKind::InvalidServerMessage(msg) => {
                write!(f, "Invalid server message: {}", msg)
            }
            GeminiErrorKind::ServerDisconnect(msg) => {
                write!(f, "Server disconnected: {}", msg)
            }
            GeminiErrorKind::StreamInterrupted(msg) => {
                write!(f, "Stream interrupted: {}", msg)
            }
        }
    }
}

impl GeminiErrorKind {
    /// Check if this error type should be retried.
    pub fn is_retryable(&self) -> bool {
        match self {
            GeminiErrorKind::HttpError { status_code, .. } => {
                matches!(*status_code, 408 | 429 | 500 | 502 | 503 | 504)
            }
            GeminiErrorKind::WebSocketConnection(_) => true,
            GeminiErrorKind::WebSocketHandshake(_) => true,
            GeminiErrorKind::StreamInterrupted(_) => true,
            _ => false,
        }
    }

    /// Get retry strategy parameters for this error type.
    ///
    /// Returns `(initial_backoff_ms, max_retries, max_delay_secs)`.
    pub fn retry_strategy_params(&self) -> (u64, usize, u64) {
        match self {
            GeminiErrorKind::HttpError { status_code, .. } => match *status_code {
                429 => (5000, 3, 40),
                503 => (2000, 5, 60),
                500 | 502 | 504 => (1000, 3, 8),
                408 => (2000, 4, 30),
                _ => (2000, 5, 60),
            },
            GeminiErrorKind::WebSocketConnection(_) => (2000, 5, 60),
            GeminiErrorKind::WebSocketHandshake(_) => (2000, 5, 60),
            GeminiErrorKind::StreamInterrupted(_) => (1000, 3, 10),
            _ => (2000, 5, 60),
        }
    }
}

/// Gemini error with source location tracking.
///
/// # Examples
///
/// ```
/// use botticelli_error::{GeminiError, GeminiErrorKind};
///
/// let err = GeminiError::new(GeminiErrorKind::MissingApiKey);
/// assert!(format!("{}", err).contains("GEMINI_API_KEY"));
/// ```
#[derive(Debug, Clone)]
pub struct GeminiError {
    /// The kind of error that occurred
    pub kind: GeminiErrorKind,
    /// Line number where error was created
    pub line: u32,
    /// File where error was created
    pub file: &'static str,
}

impl GeminiError {
    /// Create a new GeminiError with automatic location tracking.
    #[track_caller]
    pub fn new(kind: GeminiErrorKind) -> Self {
        let location = std::panic::Location::caller();
        Self {
            kind,
            line: location.line(),
            file: location.file(),
        }
    }
}

impl std::fmt::Display for GeminiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Gemini Error: {} at line {} in {}",
            self.kind, self.line, self.file
        )
    }
}

impl std::error::Error for GeminiError {}

/// Trait for errors that support retry logic.
///
/// This trait allows error types to specify whether they should trigger a retry
/// and what retry strategy parameters to use.
///
/// # Examples
///
/// ```
/// use botticelli_error::{GeminiError, GeminiErrorKind, RetryableError};
///
/// let err = GeminiError::new(GeminiErrorKind::HttpError {
///     status_code: 503,
///     message: "Service unavailable".to_string(),
/// });
///
/// assert!(err.is_retryable());
/// let (backoff, retries, max_delay) = err.retry_strategy_params();
/// assert_eq!(backoff, 2000);  // 2 second initial backoff
/// assert_eq!(retries, 5);     // 5 retry attempts
/// ```
pub trait RetryableError {
    /// Returns true if this error should trigger a retry.
    ///
    /// Transient errors like 503 (service unavailable), 429 (rate limit),
    /// or network timeouts should return true. Permanent errors like 401
    /// (unauthorized) or 400 (bad request) should return false.
    fn is_retryable(&self) -> bool;

    /// Get retry strategy parameters for this error.
    ///
    /// Returns `(initial_backoff_ms, max_retries, max_delay_secs)`.
    /// Default implementation returns standard parameters.
    ///
    /// Override this to provide error-specific retry strategies:
    /// - Rate limit errors (429): Longer delays, fewer retries
    /// - Server overload (503): Standard delays, more patient
    /// - Server errors (500): Quick retries, fail fast
    fn retry_strategy_params(&self) -> (u64, usize, u64) {
        (2000, 5, 60) // Default: 2s initial, 5 retries, 60s cap
    }
}

impl RetryableError for GeminiError {
    fn is_retryable(&self) -> bool {
        self.kind.is_retryable()
    }

    fn retry_strategy_params(&self) -> (u64, usize, u64) {
        self.kind.retry_strategy_params()
    }
}

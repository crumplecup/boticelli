//! HTTP error types.

/// HTTP error wrapping reqwest errors with source location.
#[derive(Debug, Clone)]
pub struct HttpError {
    /// The underlying error message
    pub message: String,
    /// Line number where the error occurred
    pub line: u32,
    /// File where the error occurred
    pub file: &'static str,
}

impl HttpError {
    /// Create a new HttpError with the given message at the current location.
    ///
    /// # Examples
    ///
    /// ```
    /// use botticelli_error::HttpError;
    ///
    /// let err = HttpError::new("Connection refused");
    /// assert!(err.message.contains("Connection refused"));
    /// ```
    #[track_caller]
    pub fn new(message: impl Into<String>) -> Self {
        let location = std::panic::Location::caller();
        Self {
            message: message.into(),
            line: location.line(),
            file: location.file(),
        }
    }
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HTTP Error: {} at line {} in {}",
            self.message, self.line, self.file
        )
    }
}

impl std::error::Error for HttpError {}

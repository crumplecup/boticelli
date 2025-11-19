//! Backend error types.

/// Backend error with source location.
#[derive(Debug, Clone)]
pub struct BackendError {
    /// Error message
    pub message: String,
    /// Line number where the error occurred
    pub line: u32,
    /// File where the error occurred
    pub file: &'static str,
}

impl BackendError {
    /// Create a new BackendError with the given message at the current location.
    ///
    /// # Examples
    ///
    /// ```
    /// use botticelli_error::BackendError;
    ///
    /// let err = BackendError::new("Backend service unavailable");
    /// assert!(err.message.contains("unavailable"));
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

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Backend Error: {} at line {} in {}",
            self.message, self.line, self.file
        )
    }
}

impl std::error::Error for BackendError {}

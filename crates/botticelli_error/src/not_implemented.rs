//! Not implemented error types.

/// Not implemented error with source location.
#[derive(Debug, Clone)]
pub struct NotImplementedError {
    /// Description of what is not implemented
    pub message: String,
    /// Line number where the error occurred
    pub line: u32,
    /// File where the error occurred
    pub file: &'static str,
}

impl NotImplementedError {
    /// Create a new NotImplementedError with the given message at the current location.
    ///
    /// # Examples
    ///
    /// ```
    /// use botticelli_error::NotImplementedError;
    ///
    /// let err = NotImplementedError::new("Feature X not yet supported");
    /// assert!(err.message.contains("not yet supported"));
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

impl std::fmt::Display for NotImplementedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Not Implemented: {} at line {} in {}",
            self.message, self.line, self.file
        )
    }
}

impl std::error::Error for NotImplementedError {}

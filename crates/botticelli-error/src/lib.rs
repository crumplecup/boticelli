//! Error types for the Botticelli library.
//!
//! This crate provides the foundation error types used throughout the Botticelli ecosystem.

/// HTTP error wrapping reqwest errors with source location.
#[derive(Debug)]
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

/// JSON serialization/deserialization error with source location.
#[derive(Debug)]
pub struct JsonError {
    /// The underlying error message
    pub message: String,
    /// Line number where the error occurred
    pub line: u32,
    /// File where the error occurred
    pub file: &'static str,
}

impl JsonError {
    /// Create a new JsonError with the given message at the current location.
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

impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "JSON Error: {} at line {} in {}",
            self.message, self.line, self.file
        )
    }
}

impl std::error::Error for JsonError {}

/// Configuration error with source location.
#[derive(Debug)]
pub struct ConfigError {
    /// Error message
    pub message: String,
    /// Line number where the error occurred
    pub line: u32,
    /// File where the error occurred
    pub file: &'static str,
}

impl ConfigError {
    /// Create a new ConfigError with the given message at the current location.
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

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Configuration Error: {} at line {} in {}",
            self.message, self.line, self.file
        )
    }
}

impl std::error::Error for ConfigError {}

/// Not implemented error with source location.
#[derive(Debug)]
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

/// Backend error with source location.
#[derive(Debug)]
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

/// Crate-level error variants.
///
/// This is the foundation error enum. Additional variants will be added
/// by other botticelli crates during the workspace migration.
#[derive(Debug, derive_more::From)]
pub enum BotticelliErrorKind {
    /// HTTP error
    Http(HttpError),
    /// JSON serialization/deserialization error
    Json(JsonError),
    /// Generic backend error
    Backend(BackendError),
    /// Configuration error
    Config(ConfigError),
    /// Feature not yet implemented
    NotImplemented(NotImplementedError),
}

impl std::fmt::Display for BotticelliErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BotticelliErrorKind::Http(e) => write!(f, "{}", e),
            BotticelliErrorKind::Json(e) => write!(f, "{}", e),
            BotticelliErrorKind::Backend(e) => write!(f, "{}", e),
            BotticelliErrorKind::Config(e) => write!(f, "{}", e),
            BotticelliErrorKind::NotImplemented(e) => write!(f, "{}", e),
        }
    }
}

/// Botticelli error with kind discrimination.
#[derive(Debug)]
pub struct BotticelliError(Box<BotticelliErrorKind>);

impl BotticelliError {
    /// Create a new error from a kind.
    pub fn new(kind: BotticelliErrorKind) -> Self {
        Self(Box::new(kind))
    }

    /// Get the error kind.
    pub fn kind(&self) -> &BotticelliErrorKind {
        &self.0
    }
}

impl std::fmt::Display for BotticelliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Botticelli Error: {}", self.0)
    }
}

impl std::error::Error for BotticelliError {}

// Generic From implementation for any type that converts to BotticelliErrorKind
impl<T> From<T> for BotticelliError
where
    T: Into<BotticelliErrorKind>,
{
    fn from(err: T) -> Self {
        Self::new(err.into())
    }
}

/// Result type for Botticelli operations.
pub type BotticelliResult<T> = std::result::Result<T, BotticelliError>;

//! Storage-specific error types.

/// Kinds of storage errors.
#[derive(Debug, Clone, PartialEq)]
pub enum StorageErrorKind {
    /// Media not found at the specified location
    NotFound(String),
    /// Permission denied when accessing storage
    PermissionDenied(String),
    /// I/O error during storage operation
    Io(String),
    /// Invalid storage configuration
    InvalidConfig(String),
    /// Storage backend is unavailable
    Unavailable(String),
    /// Content hash mismatch (corruption detected)
    HashMismatch { expected: String, actual: String },
    /// Generic storage error with message
    Other(String),
}

impl std::fmt::Display for StorageErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageErrorKind::NotFound(path) => write!(f, "Media not found: {}", path),
            StorageErrorKind::PermissionDenied(msg) => {
                write!(f, "Permission denied: {}", msg)
            }
            StorageErrorKind::Io(msg) => write!(f, "I/O error: {}", msg),
            StorageErrorKind::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
            StorageErrorKind::Unavailable(msg) => write!(f, "Storage unavailable: {}", msg),
            StorageErrorKind::HashMismatch { expected, actual } => {
                write!(
                    f,
                    "Content hash mismatch: expected {}, got {}",
                    expected, actual
                )
            }
            StorageErrorKind::Other(msg) => write!(f, "{}", msg),
        }
    }
}

/// Storage error with location tracking.
#[derive(Debug, Clone)]
pub struct StorageError {
    /// The kind of error that occurred
    pub kind: StorageErrorKind,
    /// Line number where error was created
    pub line: u32,
    /// File where error was created
    pub file: &'static str,
}

impl StorageError {
    /// Create a new storage error with automatic location tracking.
    #[track_caller]
    pub fn new(kind: StorageErrorKind) -> Self {
        let location = std::panic::Location::caller();
        Self {
            kind,
            line: location.line(),
            file: location.file(),
        }
    }
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Storage Error: {} at line {} in {}",
            self.kind, self.line, self.file
        )
    }
}

impl std::error::Error for StorageError {}

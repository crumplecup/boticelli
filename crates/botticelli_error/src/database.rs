//! Database error types.

/// Database error conditions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DatabaseErrorKind {
    /// Connection failed
    Connection(String),
    /// Query execution failed
    Query(String),
    /// Serialization/deserialization error
    Serialization(String),
    /// Migration error
    Migration(String),
    /// Record not found
    NotFound,
    /// Table not found
    TableNotFound(String),
    /// Schema inference error
    SchemaInference(String),
}

impl std::fmt::Display for DatabaseErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseErrorKind::Connection(msg) => write!(f, "Database connection error: {}", msg),
            DatabaseErrorKind::Query(msg) => write!(f, "Database query error: {}", msg),
            DatabaseErrorKind::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            DatabaseErrorKind::Migration(msg) => write!(f, "Migration error: {}", msg),
            DatabaseErrorKind::NotFound => write!(f, "Record not found"),
            DatabaseErrorKind::TableNotFound(table) => {
                write!(f, "Table '{}' not found in database", table)
            }
            DatabaseErrorKind::SchemaInference(msg) => {
                write!(f, "Schema inference error: {}", msg)
            }
        }
    }
}

/// Database error with source location tracking.
///
/// # Examples
///
/// ```
/// use botticelli_error::{DatabaseError, DatabaseErrorKind};
///
/// let err = DatabaseError::new(DatabaseErrorKind::NotFound);
/// assert!(format!("{}", err).contains("not found"));
/// ```
#[derive(Debug, Clone)]
pub struct DatabaseError {
    /// The kind of error that occurred
    pub kind: DatabaseErrorKind,
    /// Line number where error was created
    pub line: u32,
    /// File where error was created
    pub file: &'static str,
}

impl DatabaseError {
    /// Create a new DatabaseError with automatic location tracking.
    #[track_caller]
    pub fn new(kind: DatabaseErrorKind) -> Self {
        let location = std::panic::Location::caller();
        Self {
            kind,
            line: location.line(),
            file: location.file(),
        }
    }
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Database Error: {} at line {} in {}",
            self.kind, self.line, self.file
        )
    }
}

impl std::error::Error for DatabaseError {}

// Diesel error conversions (only available with database feature)
#[cfg(feature = "database")]
impl From<diesel::result::Error> for DatabaseError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::NotFound => DatabaseError::new(DatabaseErrorKind::NotFound),
            _ => DatabaseError::new(DatabaseErrorKind::Query(err.to_string())),
        }
    }
}

#[cfg(feature = "database")]
impl From<diesel::ConnectionError> for DatabaseError {
    fn from(err: diesel::ConnectionError) -> Self {
        DatabaseError::new(DatabaseErrorKind::Connection(err.to_string()))
    }
}

#[cfg(feature = "database")]
impl From<serde_json::Error> for DatabaseError {
    fn from(err: serde_json::Error) -> Self {
        DatabaseError::new(DatabaseErrorKind::Serialization(err.to_string()))
    }
}

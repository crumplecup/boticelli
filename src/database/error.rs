//! Database-specific error types.

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
        }
    }
}

/// Database error with source location tracking.
#[derive(Debug, Clone)]
pub struct DatabaseError {
    pub kind: DatabaseErrorKind,
    pub line: u32,
    pub file: &'static str,
}

impl DatabaseError {
    /// Create a new DatabaseError with the given kind at the current location.
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

impl From<diesel::result::Error> for DatabaseError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::NotFound => DatabaseError::new(DatabaseErrorKind::NotFound),
            _ => DatabaseError::new(DatabaseErrorKind::Query(err.to_string())),
        }
    }
}

impl From<diesel::ConnectionError> for DatabaseError {
    fn from(err: diesel::ConnectionError) -> Self {
        DatabaseError::new(DatabaseErrorKind::Connection(err.to_string()))
    }
}

impl From<serde_json::Error> for DatabaseError {
    fn from(err: serde_json::Error) -> Self {
        DatabaseError::new(DatabaseErrorKind::Serialization(err.to_string()))
    }
}

/// Result type for database operations.
pub type DatabaseResult<T> = Result<T, DatabaseError>;

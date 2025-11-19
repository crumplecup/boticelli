//! Discord-specific error types.
//!
//! This module provides error handling for Discord integration, including
//! Serenity API errors, connection issues, and Discord-specific validation errors.

use std::fmt;

/// Discord error variants.
///
/// Represents different error conditions that can occur during Discord operations.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiscordErrorKind {
    /// Serenity API error (e.g., HTTP error, gateway error, rate limit).
    SerenityError(String),

    /// Database operation failed.
    DatabaseError(String),

    /// Guild (server) not found by ID.
    GuildNotFound(i64),

    /// Channel not found by ID.
    ChannelNotFound(i64),

    /// User not found by ID.
    UserNotFound(i64),

    /// Role not found by ID.
    RoleNotFound(i64),

    /// Bot lacks required permissions for an operation.
    InsufficientPermissions(String),

    /// Invalid Discord snowflake ID format.
    InvalidId(String),

    /// Connection to Discord gateway failed.
    ConnectionFailed(String),

    /// Bot token is invalid or expired.
    InvalidToken,

    /// Message failed to send.
    MessageSendFailed(String),

    /// Interaction (slash command, button) failed.
    InteractionFailed(String),

    /// Configuration error (missing env vars, invalid settings).
    ConfigurationError(String),
}

impl fmt::Display for DiscordErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SerenityError(msg) => write!(f, "Serenity API error: {msg}"),
            Self::DatabaseError(msg) => write!(f, "Database error: {msg}"),
            Self::GuildNotFound(id) => write!(f, "Guild not found: {id}"),
            Self::ChannelNotFound(id) => write!(f, "Channel not found: {id}"),
            Self::UserNotFound(id) => write!(f, "User not found: {id}"),
            Self::RoleNotFound(id) => write!(f, "Role not found: {id}"),
            Self::InsufficientPermissions(msg) => write!(f, "Insufficient permissions: {msg}"),
            Self::InvalidId(msg) => write!(f, "Invalid ID: {msg}"),
            Self::ConnectionFailed(msg) => write!(f, "Connection failed: {msg}"),
            Self::InvalidToken => write!(f, "Invalid or expired bot token"),
            Self::MessageSendFailed(msg) => write!(f, "Message send failed: {msg}"),
            Self::InteractionFailed(msg) => write!(f, "Interaction failed: {msg}"),
            Self::ConfigurationError(msg) => write!(f, "Configuration error: {msg}"),
        }
    }
}

/// Discord error with source location tracking.
///
/// Captures the error kind along with the file and line where the error occurred.
#[derive(Debug, Clone)]
pub struct DiscordError {
    pub kind: DiscordErrorKind,
    pub line: u32,
    pub file: &'static str,
}

impl DiscordError {
    /// Create a new DiscordError with automatic location tracking.
    ///
    /// # Example
    /// ```
    /// use botticelli_social::discord::{DiscordError, DiscordErrorKind};
    ///
    /// let err = DiscordError::new(DiscordErrorKind::InvalidToken);
    /// ```
    #[track_caller]
    pub fn new(kind: DiscordErrorKind) -> Self {
        let location = std::panic::Location::caller();
        Self {
            kind,
            line: location.line(),
            file: location.file(),
        }
    }
}

impl fmt::Display for DiscordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Discord Error: {} at line {} in {}", self.kind, self.line, self.file)
    }
}

impl std::error::Error for DiscordError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

/// Result type for Discord operations.
pub type DiscordResult<T> = Result<T, DiscordError>;

// Convenience From implementations for external error types
#[cfg(feature = "discord")]
impl From<serenity::Error> for DiscordError {
    #[track_caller]
    fn from(err: serenity::Error) -> Self {
        DiscordError::new(DiscordErrorKind::SerenityError(err.to_string()))
    }
}

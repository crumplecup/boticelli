//! Error types for MCP client operations.

use derive_more::{Display, Error};

/// Specific error conditions for MCP client operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display)]
pub enum McpClientErrorKind {
    /// Tool execution failed.
    #[display("Tool execution failed: {}", _0)]
    ToolExecutionFailed(String),

    /// Invalid tool call from LLM.
    #[display("Invalid tool call: {}", _0)]
    InvalidToolCall(String),

    /// Tool not found.
    #[display("Tool not found: {}", _0)]
    ToolNotFound(String),

    /// LLM backend error.
    #[display("LLM error: {}", _0)]
    LlmError(String),

    /// Serialization error.
    #[display("Serialization error: {}", _0)]
    SerializationError(String),

    /// Maximum iterations exceeded.
    #[display("Maximum iterations exceeded: {}", _0)]
    MaxIterationsExceeded(usize),
}

/// MCP client error with location tracking.
#[derive(Debug, Clone, Display, Error)]
#[display("MCP Client Error: {} at {}:{}", kind, file, line)]
pub struct McpClientError {
    /// The specific error kind.
    pub kind: McpClientErrorKind,
    /// Line number where error occurred.
    pub line: u32,
    /// File where error occurred.
    pub file: &'static str,
}

impl McpClientError {
    /// Creates a new error with automatic location tracking.
    #[track_caller]
    pub fn new(kind: McpClientErrorKind) -> Self {
        let loc = std::panic::Location::caller();
        Self {
            kind,
            line: loc.line(),
            file: loc.file(),
        }
    }
}

/// Result type for MCP client operations.
pub type McpClientResult<T> = Result<T, McpClientError>;

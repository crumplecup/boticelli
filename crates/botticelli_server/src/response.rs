use derive_getters::Getters;
use serde::{Deserialize, Serialize};

/// OpenAI-compatible chat completion response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Getters)]
pub struct ChatCompletionResponse {
    /// Unique identifier for the completion
    id: String,
    /// Object type (always "chat.completion")
    object: String,
    /// Unix timestamp of when the completion was created
    created: i64,
    /// Model used for completion
    model: String,
    /// Generated completions
    choices: Vec<Choice>,
    /// Token usage statistics
    usage: Usage,
}

/// A completion choice
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Getters)]
pub struct Choice {
    /// Index of this choice
    index: u32,
    /// The generated message
    message: ChoiceMessage,
    /// Reason why generation finished
    finish_reason: String,
}

/// Message in a choice
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Getters)]
pub struct ChoiceMessage {
    /// Role of the message (typically "assistant")
    role: String,
    /// Generated content
    content: String,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Getters)]
pub struct Usage {
    /// Tokens in the prompt
    prompt_tokens: u32,
    /// Tokens in the completion
    completion_tokens: u32,
    /// Total tokens used
    total_tokens: u32,
}

/// Streaming chat completion chunk
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Getters)]
pub struct ChatCompletionChunk {
    /// Unique identifier
    id: String,
    /// Object type (always "chat.completion.chunk")
    object: String,
    /// Unix timestamp
    created: i64,
    /// Model used
    model: String,
    /// Delta choices
    choices: Vec<ChunkChoice>,
}

/// A choice in a streaming chunk
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Getters)]
pub struct ChunkChoice {
    /// Index of this choice
    index: u32,
    /// Delta content
    delta: Delta,
    /// Finish reason (if complete)
    #[serde(skip_serializing_if = "Option::is_none")]
    finish_reason: Option<String>,
}

/// Delta content in a streaming chunk
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Getters)]
pub struct Delta {
    /// Role (only in first chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    /// Incremental content
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
}

//! Data transfer objects for OpenAI-compatible APIs.

use derive_builder::Builder;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

/// A message in the OpenAI chat format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Role: "system", "user", or "assistant"
    pub role: String,
    /// Message content
    pub content: String,
}

/// OpenAI chat completion request.
#[derive(Debug, Clone, Serialize, Builder, Getters)]
#[builder(setter(into))]
pub struct ChatRequest {
    /// Model identifier
    model: String,
    /// Conversation messages
    messages: Vec<ChatMessage>,
    /// Maximum tokens to generate
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    /// Sampling temperature
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    /// Enable streaming
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

impl ChatRequest {
    /// Creates a new builder for ChatRequest.
    pub fn builder() -> ChatRequestBuilder {
        ChatRequestBuilder::default()
    }
}

/// A choice in the OpenAI response.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatChoice {
    /// The message content
    pub message: ChatMessage,
    /// Reason for finishing
    #[serde(default)]
    pub finish_reason: Option<String>,
}

/// Token usage statistics.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatUsage {
    /// Tokens in the prompt
    #[serde(default)]
    pub prompt_tokens: Option<usize>,
    /// Tokens in the completion
    #[serde(default)]
    pub completion_tokens: Option<usize>,
    /// Total tokens
    #[serde(default)]
    pub total_tokens: Option<usize>,
}

/// OpenAI chat completion response.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    /// Response choices
    pub choices: Vec<ChatChoice>,
    /// Token usage
    #[serde(default)]
    pub usage: Option<ChatUsage>,
}

/// Errors from OpenAI-compatible APIs.
#[derive(Debug, Clone, derive_more::Display)]
pub enum OpenAICompatError {
    /// HTTP/network error
    #[display("HTTP error: {}", _0)]
    Http(String),

    /// API returned an error
    #[display("API error (status {}): {}", status, message)]
    Api {
        /// HTTP status code
        status: u16,
        /// Error message
        message: String,
    },

    /// Rate limit exceeded
    #[display("Rate limit exceeded")]
    RateLimit,

    /// Model not found
    #[display("Model not found: {}", _0)]
    ModelNotFound(String),

    /// Invalid request
    #[display("Invalid request: {}", _0)]
    InvalidRequest(String),

    /// Failed to parse response
    #[display("Response parsing failed: {}", _0)]
    ResponseParsing(String),

    /// Builder error
    #[display("Builder error: {}", _0)]
    Builder(String),
}

impl std::error::Error for OpenAICompatError {}

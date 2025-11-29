use derive_getters::Getters;
use serde::{Deserialize, Serialize};

/// OpenAI-compatible chat completion request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Getters, derive_builder::Builder)]
#[builder(setter(into))]
pub struct ChatCompletionRequest {
    /// Model identifier
    model: String,
    /// Conversation messages
    messages: Vec<Message>,
    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    max_tokens: Option<u32>,
    /// Temperature for sampling (0.0 - 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    temperature: Option<f32>,
    /// Top-p sampling parameter
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    top_p: Option<f32>,
    /// Enable streaming mode
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    stream: Option<bool>,
}

impl ChatCompletionRequest {
    /// Create a streaming version of this request
    pub fn with_streaming(self) -> Self {
        Self {
            stream: Some(true),
            ..self
        }
    }
}

/// A message in the conversation
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Getters, derive_builder::Builder,
)]
#[builder(setter(into))]
pub struct Message {
    /// Role of the message sender (system, user, assistant)
    role: String,
    /// Message content
    content: String,
}

impl Message {
    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        MessageBuilder::default()
            .role("system")
            .content(content)
            .build()
            .expect("Valid Message")
    }

    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        MessageBuilder::default()
            .role("user")
            .content(content)
            .build()
            .expect("Valid Message")
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        MessageBuilder::default()
            .role("assistant")
            .content(content)
            .build()
            .expect("Valid Message")
    }
}

//! Anthropic API request and response types.

use derive_getters::Getters;
use serde::{Deserialize, Serialize};

/// Anthropic API request.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, derive_builder::Builder)]
#[builder(setter(into), pattern = "owned")]
pub struct AnthropicRequest {
    /// Model identifier
    model: String,
    /// List of messages
    messages: Vec<AnthropicMessage>,
    /// Maximum tokens to generate
    max_tokens: u32,
    /// Optional system prompt
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    /// Optional temperature
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

impl AnthropicRequest {
    /// Creates a builder for `AnthropicRequest`.
    pub fn builder() -> AnthropicRequestBuilder {
        AnthropicRequestBuilder::default()
    }
}

/// Anthropic message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, derive_builder::Builder)]
#[builder(setter(into), pattern = "owned")]
pub struct AnthropicMessage {
    /// Role of the message sender
    role: String,
    /// Content blocks
    content: Vec<AnthropicContentBlock>,
}

impl AnthropicMessage {
    /// Creates a builder for `AnthropicMessage`.
    pub fn builder() -> AnthropicMessageBuilder {
        AnthropicMessageBuilder::default()
    }
}

/// Content block in an Anthropic message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicContentBlock {
    /// Text content
    Text {
        /// Text content
        text: String,
    },
    /// Image content
    Image {
        /// Image source
        source: AnthropicImageSource,
    },
}

/// Image source for Anthropic API.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, derive_builder::Builder)]
#[builder(setter(into), pattern = "owned")]
pub struct AnthropicImageSource {
    /// Source type (always "base64")
    #[builder(default = "\"base64\".to_string()")]
    r#type: String,
    /// Media type
    media_type: String,
    /// Base64-encoded image data
    data: String,
}

impl AnthropicImageSource {
    /// Creates a builder for `AnthropicImageSource`.
    pub fn builder() -> AnthropicImageSourceBuilder {
        AnthropicImageSourceBuilder::default()
    }
}

/// Anthropic API response.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, derive_builder::Builder)]
#[builder(setter(into), pattern = "owned")]
pub struct AnthropicResponse {
    /// Response ID
    id: String,
    /// Response type
    #[serde(rename = "type")]
    response_type: String,
    /// Role (should be "assistant")
    role: String,
    /// Content blocks
    content: Vec<AnthropicContent>,
    /// Model used
    model: String,
    /// Stop reason
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_reason: Option<String>,
    /// Usage information
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    usage: Option<AnthropicUsage>,
}

impl AnthropicResponse {
    /// Creates a builder for `AnthropicResponse`.
    pub fn builder() -> AnthropicResponseBuilder {
        AnthropicResponseBuilder::default()
    }
}

/// Content in an Anthropic response.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, derive_builder::Builder)]
#[builder(setter(into), pattern = "owned")]
pub struct AnthropicContent {
    /// Content type (always "text" for now)
    #[builder(default = "\"text\".to_string()")]
    #[serde(rename = "type")]
    content_type: String,
    /// Text content
    text: String,
}

impl AnthropicContent {
    /// Creates a builder for `AnthropicContent`.
    pub fn builder() -> AnthropicContentBuilder {
        AnthropicContentBuilder::default()
    }
}

/// Usage information from Anthropic API.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, derive_builder::Builder)]
#[builder(setter(into), pattern = "owned")]
pub struct AnthropicUsage {
    /// Input tokens
    input_tokens: u32,
    /// Output tokens
    output_tokens: u32,
}

impl AnthropicUsage {
    /// Creates a builder for `AnthropicUsage`.
    pub fn builder() -> AnthropicUsageBuilder {
        AnthropicUsageBuilder::default()
    }
}

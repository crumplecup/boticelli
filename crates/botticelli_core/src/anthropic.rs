use derive_getters::Getters;
use serde::{Deserialize, Serialize};

/// Anthropic message content block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
pub struct AnthropicContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    #[getter(skip)]
    text: String,
}

impl AnthropicContentBlock {
    /// Creates a new text content block.
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content_type: "text".to_string(),
            text: text.into(),
        }
    }

    /// Gets the text content.
    pub fn text_content(&self) -> &str {
        &self.text
    }
}

/// Anthropic API request message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters, derive_builder::Builder)]
#[builder(setter(into))]
pub struct AnthropicMessage {
    role: String,
    content: Vec<AnthropicContentBlock>,
}

impl AnthropicMessage {
    /// Creates a builder for AnthropicMessage.
    pub fn builder() -> AnthropicMessageBuilder {
        AnthropicMessageBuilder::default()
    }
}

/// Anthropic API request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters, derive_builder::Builder)]
#[builder(setter(into))]
pub struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

impl AnthropicRequest {
    /// Creates a builder for AnthropicRequest.
    pub fn builder() -> AnthropicRequestBuilder {
        AnthropicRequestBuilder::default()
    }
}

/// Anthropic API response content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
pub struct AnthropicResponseContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

/// Anthropic API response usage stats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
pub struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Anthropic API response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
pub struct AnthropicResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    content: Vec<AnthropicResponseContent>,
    model: String,
    stop_reason: Option<String>,
    usage: AnthropicUsage,
}

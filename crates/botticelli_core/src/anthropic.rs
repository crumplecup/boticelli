//! Anthropic API types and client.

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

/// Anthropic API configuration.
#[derive(Debug, Clone, Getters, derive_builder::Builder)]
#[builder(setter(into))]
pub struct AnthropicConfig {
    api_key: String,
    #[builder(default = "\"https://api.anthropic.com\".to_string()")]
    endpoint: String,
}

impl AnthropicConfig {
    /// Creates a builder for AnthropicConfig.
    pub fn builder() -> AnthropicConfigBuilder {
        AnthropicConfigBuilder::default()
    }
}

#[cfg(feature = "anthropic")]
use botticelli_error::{AnthropicErrorKind, ModelsError, ModelsErrorKind, ModelsResult};

/// Anthropic HTTP client.
#[cfg(feature = "anthropic")]
#[derive(Debug, Clone)]
pub struct AnthropicClient {
    client: reqwest::Client,
    config: AnthropicConfig,
}

#[cfg(feature = "anthropic")]
impl AnthropicClient {
    /// Creates a new Anthropic client.
    #[tracing::instrument(skip(config))]
    pub fn new(config: AnthropicConfig) -> ModelsResult<Self> {
        use std::time::Duration;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| {
                ModelsError::new(ModelsErrorKind::Anthropic(AnthropicErrorKind::Http(
                    e.to_string(),
                )))
            })?;

        Ok(Self { client, config })
    }

    /// Sends a generation request to Anthropic API.
    #[tracing::instrument(skip(self, request), fields(model = %request.model()))]
    pub async fn generate(&self, request: AnthropicRequest) -> ModelsResult<AnthropicResponse> {
        let url = format!("{}/v1/messages", self.config.endpoint());

        let response = self
            .client
            .post(&url)
            .header("x-api-key", self.config.api_key())
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                ModelsError::new(ModelsErrorKind::Anthropic(AnthropicErrorKind::Http(
                    e.to_string(),
                )))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ModelsError::new(ModelsErrorKind::Anthropic(
                AnthropicErrorKind::ApiError {
                    status: status.as_u16(),
                    message: body,
                },
            )));
        }

        response.json::<AnthropicResponse>().await.map_err(|e| {
            ModelsError::new(ModelsErrorKind::Anthropic(AnthropicErrorKind::Parse(
                e.to_string(),
            )))
        })
    }
}

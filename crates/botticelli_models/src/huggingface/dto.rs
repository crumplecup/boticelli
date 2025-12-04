//! HuggingFace Inference API data transfer objects.

use derive_builder::Builder;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

/// HuggingFace API request parameters.
#[derive(Debug, Clone, Getters, Builder)]
#[builder(setter(into))]
pub struct HuggingFaceRequest {
    /// Model identifier
    model: String,
    /// Input text
    inputs: String,
    /// Optional parameters
    #[builder(default)]
    parameters: Option<HuggingFaceParameters>,
}

impl HuggingFaceRequest {
    /// Creates a new builder for `HuggingFaceRequest`.
    pub fn builder() -> HuggingFaceRequestBuilder {
        HuggingFaceRequestBuilder::default()
    }
}

/// HuggingFace generation parameters.
#[derive(Debug, Clone, Getters, Builder, Serialize)]
#[builder(setter(into))]
pub struct HuggingFaceParameters {
    /// Maximum new tokens to generate
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    max_new_tokens: Option<u32>,
    /// Temperature for sampling
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    /// Top-p sampling
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
}

impl HuggingFaceParameters {
    /// Creates a new builder for `HuggingFaceParameters`.
    pub fn builder() -> HuggingFaceParametersBuilder {
        HuggingFaceParametersBuilder::default()
    }
}

/// HuggingFace response metadata.
#[derive(Debug, Clone, Getters, Deserialize)]
pub struct HuggingFaceMetadata {
    /// Tokens in the prompt
    #[serde(default)]
    prompt_tokens: Option<usize>,
    /// Tokens in the completion
    #[serde(default)]
    completion_tokens: Option<usize>,
}

/// HuggingFace API response.
#[derive(Debug, Clone, Getters, Builder, Deserialize)]
#[builder(setter(into))]
pub struct HuggingFaceResponse {
    /// Generated text
    generated_text: String,
    /// Response metadata
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<HuggingFaceMetadata>,
}

impl HuggingFaceResponse {
    /// Creates a new builder for `HuggingFaceResponse`.
    pub fn builder() -> HuggingFaceResponseBuilder {
        HuggingFaceResponseBuilder::default()
    }
}

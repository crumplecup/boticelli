//! Request and response types for LLM generation.

use crate::{Message, Output};
use serde::{Deserialize, Serialize};

/// Generic generation request (multimodal-safe).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct GenerateRequest {
    pub messages: Vec<Message>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub model: Option<String>,
}

/// The unified response object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub outputs: Vec<Output>,
}

//! Test utilities for Botticelli tests.
//!
//! This module provides mock implementations and test helpers.

use botticelli_core::{GenerateRequest, GenerateRequestBuilder, Message, Role};

pub mod mock_gemini;

#[allow(unused_imports)]
pub use mock_gemini::{MockBehavior, MockGeminiClient, MockResponse};

/// Helper to create a test GenerateRequest using the builder pattern.
pub fn create_test_request(
    prompt: &str,
    model: Option<String>,
    max_tokens: Option<usize>,
) -> GenerateRequest {
    let mut builder = GenerateRequestBuilder::default();
    builder
        .messages(vec![Message::new(Role::User, prompt.to_string())])
        .max_tokens(max_tokens)
        .temperature(None);
    
    if let Some(m) = model {
        builder.model(Some(m));
    }
    
    builder.build().expect("Failed to build test request")
}

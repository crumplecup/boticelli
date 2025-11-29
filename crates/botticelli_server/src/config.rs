//! Configuration for local inference server connection

use crate::{ServerError, ServerErrorKind};
use derive_getters::Getters;

/// Configuration for local inference server connection
#[derive(Debug, Clone, PartialEq, Eq, Hash, Getters, derive_builder::Builder)]
#[builder(setter(into))]
pub struct ServerConfig {
    /// Base URL of the server (e.g., "http://localhost:8080")
    base_url: String,
    /// Model identifier to use for inference
    model: String,
    /// Optional API key (mistral.rs doesn't require one by default)
    #[builder(default)]
    api_key: Option<String>,
}

impl ServerConfig {
    /// Create config from environment variables
    ///
    /// Reads:
    /// - `INFERENCE_SERVER_BASE_URL` (default: "http://localhost:8080")
    /// - `INFERENCE_SERVER_MODEL` (required)
    /// - `INFERENCE_SERVER_API_KEY` (optional)
    pub fn from_env() -> Result<Self, ServerError> {
        let base_url = std::env::var("INFERENCE_SERVER_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let model = std::env::var("INFERENCE_SERVER_MODEL").map_err(|_| {
            ServerError::new(ServerErrorKind::Configuration(
                "INFERENCE_SERVER_MODEL not set".into(),
            ))
        })?;
        let api_key = std::env::var("INFERENCE_SERVER_API_KEY").ok();

        Ok(ServerConfigBuilder::default()
            .base_url(base_url)
            .model(model)
            .api_key(api_key)
            .build()
            .expect("Valid ServerConfig"))
    }
}

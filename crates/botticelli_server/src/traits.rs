//! Trait interfaces for local inference server implementations
//!
//! This module defines the core traits that server implementations must provide:
//! - [`InferenceServer`] - Server lifecycle and health management
//! - [`ServerLauncher`] - Server creation and initialization
//! - [`ModelManager`] - Model downloading and management

use async_trait::async_trait;
use botticelli_error::ServerError;
use std::path::PathBuf;
use std::time::Duration;

/// Trait for managing the lifecycle of an inference server process
///
/// Implementations handle starting, monitoring, and stopping inference servers.
#[async_trait]
pub trait InferenceServer: Send + Sync {
    /// Get the port the server is listening on
    fn port(&self) -> u16;

    /// Get the base URL of the server (e.g., "http://localhost:8080")
    fn base_url(&self) -> String;

    /// Check if the server is healthy and responding to requests
    async fn health_check(&self) -> Result<(), ServerError>;

    /// Wait for the server to become ready, with timeout
    ///
    /// Polls the server health endpoint until it responds successfully
    /// or the timeout is reached.
    async fn wait_until_ready(&self, timeout: Duration) -> Result<(), ServerError>;

    /// Stop the server gracefully
    ///
    /// Consumes self to ensure the server can only be stopped once.
    fn stop(self) -> Result<(), ServerError>;
}

/// Trait for launching inference servers
///
/// Implementations create and start inference server processes with
/// implementation-specific configuration.
pub trait ServerLauncher: Send + Sync {
    /// The type of server this launcher creates
    type Server: InferenceServer;

    /// Configuration required to start the server
    type Config;

    /// Start a new server instance with the given configuration
    fn start(config: Self::Config) -> Result<Self::Server, ServerError>;
}

/// Trait for managing model downloads and local storage
///
/// Implementations handle downloading models from repositories (e.g., HuggingFace)
/// and managing the local model cache.
#[async_trait]
pub trait ModelManager: Send + Sync {
    /// The type representing a model specification (e.g., model name, quantization)
    type ModelSpec: Send + Sync;

    /// Check if a model is already downloaded
    fn is_downloaded(&self, spec: &Self::ModelSpec) -> bool;

    /// Download a model if not already present
    ///
    /// Returns the path to the downloaded model file.
    async fn download(&self, spec: &Self::ModelSpec) -> Result<PathBuf, ServerError>;

    /// Get the local filesystem path where a model would be stored
    fn model_path(&self, spec: &Self::ModelSpec) -> PathBuf;

    /// Ensure a model is available, downloading if necessary
    ///
    /// Convenience method that checks if downloaded and downloads if needed.
    async fn ensure_model(&self, spec: &Self::ModelSpec) -> Result<PathBuf, ServerError> {
        if self.is_downloaded(spec) {
            Ok(self.model_path(spec))
        } else {
            self.download(spec).await
        }
    }
}

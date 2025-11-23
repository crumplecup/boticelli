//! Social media platform trait and types.

use crate::ActorResult;
use async_trait::async_trait;
use std::collections::HashMap;

/// Platform-agnostic message to post.
#[derive(Debug, Clone)]
pub struct PlatformMessage {
    /// Text content
    pub text: String,
    /// Media URLs to attach
    pub media_urls: Vec<String>,
}

/// Metadata returned after posting.
pub type PlatformMetadata = HashMap<String, String>;

/// Platform capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlatformCapability {
    /// Supports text posts
    Text,
    /// Supports image attachments
    Images,
    /// Supports video attachments
    Videos,
    /// Supports link previews
    Links,
    /// Supports scheduled posts
    Scheduling,
}

/// Trait for social media platform implementations.
#[async_trait]
pub trait Platform: Send + Sync {
    /// Post a message to the platform.
    ///
    /// # Errors
    ///
    /// Returns error if posting fails.
    async fn post(&self, message: &PlatformMessage) -> ActorResult<PlatformMetadata>;

    /// Verify platform connection is working.
    ///
    /// # Errors
    ///
    /// Returns error if connection cannot be verified.
    async fn verify_connection(&self) -> ActorResult<()>;

    /// Get platform capabilities.
    fn capabilities(&self) -> Vec<PlatformCapability>;

    /// Get platform name.
    fn platform_name(&self) -> &str;
}

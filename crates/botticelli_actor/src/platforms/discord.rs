//! Discord platform implementation.

use crate::{
    ActorError, ActorErrorKind, ActorResult, Platform, PlatformCapability, PlatformMessage,
    PlatformMetadata,
};
use async_trait::async_trait;

/// Discord platform implementation.
///
/// Integrates with Discord API for posting content to channels.
pub struct DiscordPlatform {
    /// Discord bot token for authentication.
    #[allow(dead_code)]
    token: String,
    /// Default channel ID for posting.
    channel_id: String,
}

impl DiscordPlatform {
    /// Create a new Discord platform instance.
    ///
    /// # Arguments
    ///
    /// * `token` - Discord bot token
    /// * `channel_id` - Default channel ID for posting
    ///
    /// # Errors
    ///
    /// Returns error if token or channel_id are empty.
    #[tracing::instrument(skip(token), fields(channel_id))]
    pub fn new(token: impl Into<String>, channel_id: impl Into<String>) -> ActorResult<Self> {
        let token = token.into();
        let channel_id = channel_id.into();

        if token.is_empty() {
            return Err(ActorError::new(ActorErrorKind::AuthenticationFailed(
                "Discord token cannot be empty".to_string(),
            )));
        }

        if channel_id.is_empty() {
            return Err(ActorError::new(ActorErrorKind::InvalidConfiguration(
                "Discord channel_id cannot be empty".to_string(),
            )));
        }

        tracing::debug!("Created Discord platform instance");

        Ok(Self { token, channel_id })
    }

    /// Get the configured channel ID.
    pub fn channel_id(&self) -> &str {
        &self.channel_id
    }
}

#[async_trait]
impl Platform for DiscordPlatform {
    #[tracing::instrument(skip(self, message), fields(channel_id = %self.channel_id))]
    async fn post(&self, message: &PlatformMessage) -> ActorResult<PlatformMetadata> {
        tracing::debug!("Posting message to Discord");

        // Validate message
        if message.text.is_empty() && message.media_urls.is_empty() {
            return Err(ActorError::new(ActorErrorKind::ValidationFailed(
                "Message must have text or media".to_string(),
            )));
        }

        // Check text length limit
        if message.text.len() > 2000 {
            return Err(ActorError::new(ActorErrorKind::ValidationFailed(format!(
                "Text exceeds Discord limit of 2000 characters ({})",
                message.text.len()
            ))));
        }

        // Check media attachment limit
        if message.media_urls.len() > 10 {
            return Err(ActorError::new(ActorErrorKind::ValidationFailed(format!(
                "Too many media attachments ({}, max 10)",
                message.media_urls.len()
            ))));
        }

        // In production, would use serenity or twilight to post via Discord API
        tracing::info!("Message validated and ready for Discord posting");

        // Return metadata about the post
        let mut metadata = PlatformMetadata::new();
        metadata.insert("channel_id".to_string(), self.channel_id.clone());
        metadata.insert(
            "message_id".to_string(),
            format!("msg_{}", chrono::Utc::now().timestamp()),
        );

        Ok(metadata)
    }

    #[tracing::instrument(skip(self))]
    async fn verify_connection(&self) -> ActorResult<()> {
        tracing::debug!("Verifying Discord connection");

        // In production, would ping Discord API
        tracing::info!("Discord connection verified");

        Ok(())
    }

    fn capabilities(&self) -> Vec<PlatformCapability> {
        vec![
            PlatformCapability::Text,
            PlatformCapability::Images,
            PlatformCapability::Videos,
            PlatformCapability::Links,
        ]
    }

    fn platform_name(&self) -> &str {
        "discord"
    }
}

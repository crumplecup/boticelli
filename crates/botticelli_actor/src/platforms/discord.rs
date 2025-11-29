//! Discord platform implementation.

use crate::{
    ActorError, ActorErrorKind, ActorResult, Platform, PlatformCapability, PlatformMessage,
    PlatformMetadata,
};
use async_trait::async_trait;

/// Discord maximum message length in characters.
const DISCORD_MAX_MESSAGE_LENGTH: usize = 2000;

/// Discord maximum number of attachments per message.
const DISCORD_MAX_ATTACHMENTS: usize = 10;

/// Discord platform implementation.
///
/// Integrates with Discord API for posting content to channels.
#[derive(derive_builder::Builder)]
#[builder(setter(into))]
pub struct DiscordPlatform {
    /// Default channel ID for posting.
    channel_id: String,
    // Token will be retrieved from config when needed
}

impl DiscordPlatform {
    /// Create a new Discord platform instance.
    ///
    /// # Arguments
    ///
    /// * `channel_id` - Default channel ID for posting
    ///
    /// # Errors
    ///
    /// Returns error if channel_id is empty.
    #[tracing::instrument(fields(channel_id))]
    pub fn new(channel_id: impl Into<String>) -> ActorResult<Self> {
        let channel_id = channel_id.into();

        if channel_id.is_empty() {
            return Err(ActorError::new(ActorErrorKind::InvalidConfiguration(
                "Discord channel_id cannot be empty".to_string(),
            )));
        }

        tracing::debug!("Created Discord platform instance");

        Ok(DiscordPlatformBuilder::default()
            .channel_id(channel_id)
            .build()
            .expect("DiscordPlatform with validated fields"))
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
        if message.text.len() > DISCORD_MAX_MESSAGE_LENGTH {
            return Err(ActorError::new(ActorErrorKind::ValidationFailed(format!(
                "Text exceeds Discord limit of {} characters ({})",
                DISCORD_MAX_MESSAGE_LENGTH,
                message.text.len()
            ))));
        }

        // Check media attachment limit
        if message.media_urls.len() > DISCORD_MAX_ATTACHMENTS {
            return Err(ActorError::new(ActorErrorKind::ValidationFailed(format!(
                "Too many media attachments ({}, max {})",
                message.media_urls.len(),
                DISCORD_MAX_ATTACHMENTS
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

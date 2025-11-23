//! Platform implementations for social media services.

#[cfg(feature = "discord")]
pub mod discord;

#[cfg(feature = "discord")]
pub use discord::DiscordPlatform;

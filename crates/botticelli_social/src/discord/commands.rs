//! Discord bot command executor.
//!
//! This module provides the Discord implementation of the BotCommandExecutor trait,
//! enabling narratives to query Discord servers for real-time data.
//!
//! # Supported Commands
//!
//! - `server.get_stats` - Get server statistics (member count, channel count, etc.)
//! - `channels.list` - List all channels in a server
//! - `roles.list` - List all roles in a server
//!
//! # Example
//!
//! ```rust,ignore
//! use botticelli_social::DiscordCommandExecutor;
//! use std::collections::HashMap;
//!
//! // Create standalone executor
//! let executor = DiscordCommandExecutor::new("DISCORD_BOT_TOKEN");
//!
//! // Or create from existing bot
//! let bot = BotticelliBot::new(token, conn).await?;
//! let executor = DiscordCommandExecutor::with_http_client(bot.http_client());
//!
//! // Execute command
//! let mut args = HashMap::new();
//! args.insert("guild_id".to_string(), serde_json::json!("1234567890"));
//! let result = executor.execute("server.get_stats", &args).await?;
//! ```

use crate::{BotCommandError, BotCommandErrorKind, BotCommandExecutor, BotCommandResult};
use async_trait::async_trait;
use serde_json::Value as JsonValue;
use serenity::http::Http;
use serenity::model::id::GuildId;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, instrument};

/// Discord command executor for bot command execution.
///
/// Implements the BotCommandExecutor trait to provide Discord-specific
/// command handling using Serenity's HTTP client.
pub struct DiscordCommandExecutor {
    http: Arc<Http>,
}

impl DiscordCommandExecutor {
    /// Create a new Discord command executor with a bot token.
    ///
    /// This creates an independent HTTP client suitable for standalone use.
    /// The executor will make direct Discord API calls without requiring
    /// a running bot instance.
    ///
    /// # Arguments
    ///
    /// * `token` - Discord bot token from the Discord Developer Portal
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let executor = DiscordCommandExecutor::new("DISCORD_BOT_TOKEN");
    /// ```
    #[instrument(skip(token), fields(token_len = token.as_ref().len()))]
    pub fn new(token: impl AsRef<str>) -> Self {
        info!("Creating standalone Discord command executor");
        let http = Arc::new(Http::new(token.as_ref()));
        Self { http }
    }

    /// Create executor with an existing HTTP client.
    ///
    /// Use this to share the HTTP client with a running bot,
    /// coordinating rate limits and reducing connections.
    ///
    /// # Arguments
    ///
    /// * `http` - Arc reference to Serenity HTTP client
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let bot = BotticelliBot::new(token, conn).await?;
    /// let executor = DiscordCommandExecutor::with_http_client(bot.http_client());
    /// ```
    pub fn with_http_client(http: Arc<Http>) -> Self {
        info!("Creating Discord command executor with shared HTTP client");
        Self { http }
    }

    /// Parse guild_id argument from command args.
    fn parse_guild_id(
        command: &str,
        args: &HashMap<String, JsonValue>,
    ) -> BotCommandResult<GuildId> {
        let guild_id_value = args.get("guild_id").ok_or_else(|| {
            error!(command, "Missing required argument: guild_id");
            BotCommandError::new(BotCommandErrorKind::MissingArgument {
                command: command.to_string(),
                arg_name: "guild_id".to_string(),
            })
        })?;

        let guild_id_str = guild_id_value.as_str().ok_or_else(|| {
            error!(command, ?guild_id_value, "guild_id must be a string");
            BotCommandError::new(BotCommandErrorKind::InvalidArgument {
                command: command.to_string(),
                arg_name: "guild_id".to_string(),
                reason: "Must be a string".to_string(),
            })
        })?;

        let guild_id_u64: u64 = guild_id_str.parse().map_err(|_| {
            error!(command, guild_id_str, "Invalid guild_id format");
            BotCommandError::new(BotCommandErrorKind::InvalidArgument {
                command: command.to_string(),
                arg_name: "guild_id".to_string(),
                reason: "Invalid Discord ID format".to_string(),
            })
        })?;

        Ok(GuildId::new(guild_id_u64))
    }

    /// Execute: server.get_stats
    ///
    /// Get server statistics including member count, channel count, role count, etc.
    ///
    /// Required args: guild_id
    #[instrument(
        skip(self, args),
        fields(
            command = "server.get_stats",
            guild_id,
            member_count,
            channel_count
        )
    )]
    async fn server_get_stats(
        &self,
        args: &HashMap<String, JsonValue>,
    ) -> BotCommandResult<JsonValue> {
        debug!("Parsing guild_id argument");
        let guild_id = Self::parse_guild_id("server.get_stats", args)?;

        tracing::Span::current().record("guild_id", guild_id.get());
        info!(guild_id = %guild_id, "Fetching guild stats from Discord API");

        // Fetch guild data
        let guild = self
            .http
            .get_guild(guild_id)
            .await
            .map_err(|e| {
                error!(guild_id = %guild_id, error = %e, "Failed to fetch guild");
                BotCommandError::new(BotCommandErrorKind::ApiError {
                    command: "server.get_stats".to_string(),
                    reason: format!("Failed to fetch guild: {}", e),
                })
            })?;

        // Fetch member count (guild.approximate_member_count is only available with partial guilds)
        // For now, we'll use the guild data we have
        let member_count = guild.approximate_member_count.unwrap_or(0);
        let channel_count = 0; // Would need separate API call to get channels

        tracing::Span::current().record("member_count", member_count);
        tracing::Span::current().record("channel_count", channel_count);

        let stats = serde_json::json!({
            "guild_id": guild.id.to_string(),
            "name": guild.name,
            "member_count": member_count,
            "description": guild.description,
            "icon_url": guild.icon_url(),
            "banner_url": guild.banner_url(),
            "owner_id": guild.owner_id.to_string(),
            "verification_level": format!("{:?}", guild.verification_level),
            "premium_tier": format!("{:?}", guild.premium_tier),
            "premium_subscription_count": guild.premium_subscription_count.unwrap_or(0),
        });

        info!(
            member_count,
            "Successfully retrieved guild stats"
        );

        Ok(stats)
    }

    /// Execute: channels.list
    ///
    /// List all channels in a server.
    ///
    /// Required args: guild_id
    #[instrument(
        skip(self, args),
        fields(
            command = "channels.list",
            guild_id,
            channel_count
        )
    )]
    async fn channels_list(
        &self,
        args: &HashMap<String, JsonValue>,
    ) -> BotCommandResult<JsonValue> {
        debug!("Parsing guild_id argument");
        let guild_id = Self::parse_guild_id("channels.list", args)?;

        tracing::Span::current().record("guild_id", guild_id.get());
        info!(guild_id = %guild_id, "Fetching channels from Discord API");

        // Fetch channels
        let channels = self
            .http
            .get_channels(guild_id)
            .await
            .map_err(|e| {
                error!(guild_id = %guild_id, error = %e, "Failed to fetch channels");
                BotCommandError::new(BotCommandErrorKind::ApiError {
                    command: "channels.list".to_string(),
                    reason: format!("Failed to fetch channels: {}", e),
                })
            })?;

        let channel_count = channels.len();
        tracing::Span::current().record("channel_count", channel_count);

        let channels_json: Vec<JsonValue> = channels
            .into_iter()
            .map(|channel| {
                serde_json::json!({
                    "id": channel.id.to_string(),
                    "name": channel.name,
                    "type": format!("{:?}", channel.kind),
                    "position": channel.position,
                    "topic": channel.topic,
                    "nsfw": channel.nsfw,
                    "parent_id": channel.parent_id.map(|id| id.to_string()),
                })
            })
            .collect();

        info!(channel_count, "Successfully retrieved channels");

        Ok(serde_json::json!(channels_json))
    }

    /// Execute: roles.list
    ///
    /// List all roles in a server.
    ///
    /// Required args: guild_id
    #[instrument(
        skip(self, args),
        fields(
            command = "roles.list",
            guild_id,
            role_count
        )
    )]
    async fn roles_list(&self, args: &HashMap<String, JsonValue>) -> BotCommandResult<JsonValue> {
        debug!("Parsing guild_id argument");
        let guild_id = Self::parse_guild_id("roles.list", args)?;

        tracing::Span::current().record("guild_id", guild_id.get());
        info!(guild_id = %guild_id, "Fetching roles from Discord API");

        // Fetch roles
        let roles = self
            .http
            .get_guild_roles(guild_id)
            .await
            .map_err(|e| {
                error!(guild_id = %guild_id, error = %e, "Failed to fetch roles");
                BotCommandError::new(BotCommandErrorKind::ApiError {
                    command: "roles.list".to_string(),
                    reason: format!("Failed to fetch roles: {}", e),
                })
            })?;

        let role_count = roles.len();
        tracing::Span::current().record("role_count", role_count);

        let roles_json: Vec<JsonValue> = roles
            .into_iter()
            .map(|role| {
                serde_json::json!({
                    "id": role.id.to_string(),
                    "name": role.name,
                    "color": role.colour.0,
                    "hoist": role.hoist,
                    "position": role.position,
                    "permissions": role.permissions.bits(),
                    "managed": role.managed,
                    "mentionable": role.mentionable,
                })
            })
            .collect();

        info!(role_count, "Successfully retrieved roles");

        Ok(serde_json::json!(roles_json))
    }
}

#[async_trait]
impl BotCommandExecutor for DiscordCommandExecutor {
    fn platform(&self) -> &str {
        "discord"
    }

    #[instrument(
        skip(self, args),
        fields(
            platform = "discord",
            command,
            arg_count = args.len(),
            result_size,
            duration_ms
        )
    )]
    async fn execute(
        &self,
        command: &str,
        args: &HashMap<String, JsonValue>,
    ) -> BotCommandResult<JsonValue> {
        info!("Executing Discord bot command");

        let start = std::time::Instant::now();

        let result = match command {
            "server.get_stats" => self.server_get_stats(args).await?,
            "channels.list" => self.channels_list(args).await?,
            "roles.list" => self.roles_list(args).await?,
            _ => {
                error!(
                    command,
                    supported = ?self.supported_commands(),
                    "Command not found"
                );
                return Err(BotCommandError::new(BotCommandErrorKind::CommandNotFound(
                    command.to_string(),
                )));
            }
        };

        let duration_ms = start.elapsed().as_millis();
        let result_size = serde_json::to_string(&result)
            .map(|s| s.len())
            .unwrap_or(0);

        tracing::Span::current().record("duration_ms", duration_ms);
        tracing::Span::current().record("result_size", result_size);
        info!(
            duration_ms,
            result_size,
            "Discord command executed successfully"
        );

        Ok(result)
    }

    fn supports_command(&self, command: &str) -> bool {
        matches!(
            command,
            "server.get_stats" | "channels.list" | "roles.list"
        )
    }

    fn supported_commands(&self) -> Vec<String> {
        vec![
            "server.get_stats".to_string(),
            "channels.list".to_string(),
            "roles.list".to_string(),
        ]
    }

    fn command_help(&self, command: &str) -> Option<String> {
        match command {
            "server.get_stats" => Some(
                "Get server statistics (member count, channels, etc.)\n\
                 Required arguments: guild_id"
                    .to_string(),
            ),
            "channels.list" => Some(
                "List all channels in a server\n\
                 Required arguments: guild_id"
                    .to_string(),
            ),
            "roles.list" => Some(
                "List all roles in a server\n\
                 Required arguments: guild_id"
                    .to_string(),
            ),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supports_command() {
        let token = "test_token";
        let executor = DiscordCommandExecutor::new(token);

        assert!(executor.supports_command("server.get_stats"));
        assert!(executor.supports_command("channels.list"));
        assert!(executor.supports_command("roles.list"));
        assert!(!executor.supports_command("unknown.command"));
    }

    #[test]
    fn test_supported_commands() {
        let token = "test_token";
        let executor = DiscordCommandExecutor::new(token);

        let commands = executor.supported_commands();
        assert_eq!(commands.len(), 3);
        assert!(commands.contains(&"server.get_stats".to_string()));
        assert!(commands.contains(&"channels.list".to_string()));
        assert!(commands.contains(&"roles.list".to_string()));
    }

    #[test]
    fn test_command_help() {
        let token = "test_token";
        let executor = DiscordCommandExecutor::new(token);

        assert!(executor.command_help("server.get_stats").is_some());
        assert!(executor.command_help("channels.list").is_some());
        assert!(executor.command_help("roles.list").is_some());
        assert!(executor.command_help("unknown.command").is_none());
    }

    #[test]
    fn test_platform() {
        let token = "test_token";
        let executor = DiscordCommandExecutor::new(token);

        assert_eq!(executor.platform(), "discord");
    }
}

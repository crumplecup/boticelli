//! Integration tests for Discord bot commands using direct API calls.

use botticelli_social::{BotCommandExecutor, BotticelliBot};
use std::env;

/// Helper to load environment variables from .env
fn load_env() {
    dotenvy::dotenv().ok();
}

/// Helper to get test guild ID from environment
fn get_test_guild_id() -> String {
    env::var("TEST_GUILD_ID").expect("TEST_GUILD_ID not set in environment")
}

/// Helper to get Discord token from environment
fn get_discord_token() -> String {
    env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set in environment")
}

/// Helper to create a Discord bot
async fn create_discord_bot() -> BotticelliBot {
    let token = get_discord_token();
    BotticelliBot::new(token).await.expect("Failed to create Discord bot")
}

#[tokio::test]
#[cfg_attr(not(feature = "discord"), ignore)]
async fn test_channels_list() {
    load_env();
    let bot = create_discord_bot().await;
    let guild_id = get_test_guild_id();
    
    // Test channels.list command
    let result = bot.execute("channels.list", vec![("guild_id", &guild_id)]).await;
    assert!(result.is_ok(), "Failed to list channels: {:?}", result.err());
}

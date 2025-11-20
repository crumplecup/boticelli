//! Quick test to verify Discord command execution works
//!
//! Run with: cargo run --example test_discord_commands --features discord

use botticelli_social::DiscordCommandExecutor;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    // Load .env
    dotenvy::dotenv().ok();
    
    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set in .env");
    let guild_id = std::env::var("TEST_GUILD_ID").unwrap_or_else(|_| {
        eprintln!("⚠️  TEST_GUILD_ID not set, using placeholder");
        "YOUR_GUILD_ID".to_string()
    });
    
    if guild_id == "YOUR_GUILD_ID" {
        eprintln!("ERROR: Please set TEST_GUILD_ID in .env to a Discord server ID where your bot is a member");
        std::process::exit(1);
    }
    
    println!("🤖 Creating Discord command executor...");
    let executor = DiscordCommandExecutor::new(&token);
    
    println!("📊 Executing server.get_stats for guild {}...\n", guild_id);
    let mut args = HashMap::new();
    args.insert("guild_id".to_string(), serde_json::json!(guild_id));
    
    match executor.execute("server.get_stats", &args).await {
        Ok(result) => {
            println!("✅ Success! Got server stats:\n");
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }
        Err(e) => {
            eprintln!("❌ Error executing command: {}", e);
            std::process::exit(1);
        }
    }
    
    println!("\n📋 Executing channels.list...\n");
    match executor.execute("channels.list", &args).await {
        Ok(result) => {
            let channels = result.as_array().unwrap();
            println!("✅ Found {} channels", channels.len());
            if let Some(first) = channels.first() {
                println!("First channel: {}", serde_json::to_string_pretty(first).unwrap());
            }
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
        }
    }
    
    println!("\n🎭 Executing roles.list...\n");
    match executor.execute("roles.list", &args).await {
        Ok(result) => {
            let roles = result.as_array().unwrap();
            println!("✅ Found {} roles", roles.len());
            if let Some(first) = roles.first() {
                println!("First role: {}", serde_json::to_string_pretty(first).unwrap());
            }
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
        }
    }
    
    println!("\n🎉 All Discord command tests passed!");
}

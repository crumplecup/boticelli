//! Bot server command handler.

use botticelli_error::BotticelliResult;
use botticelli_server::BotServer;
use std::path::PathBuf;

use std::time::Duration;

/// Handle the `server` command
pub async fn handle_server_command(
    _config_path: Option<PathBuf>,
    _only_bots: Option<String>,
) -> BotticelliResult<()> {
    tracing::info!("Starting bot server");

    // Create and run the server
    let mut server = BotServer::new();
    
    // Default intervals
    let generation_interval = Duration::from_secs(6 * 60 * 60); // 6 hours
    let curation_interval = Duration::from_secs(12 * 60 * 60);   // 12 hours
    let posting_interval = Duration::from_secs(3 * 60 * 60);     // 3 hours base
    
    server.start(generation_interval, curation_interval, posting_interval).await?;
    
    tracing::info!("Bot server started. Press Ctrl+C to stop.");
    
    // Keep server running
    tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
    
    tracing::info!("Shutting down bot server...");
    server.stop().await?;

    Ok(())
}

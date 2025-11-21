//! Integration tests for Discord narratives.

use botticelli_interface::{ContentRepository, GenerationBackend, NarrativeRepository};
use botticelli_narrative::{InMemoryNarrativeRepository, NarrativeExecutor};
use std::path::PathBuf;

/// Helper to load .env file for tests
fn load_env() {
    dotenvy::dotenv().ok();
}

#[tokio::test]
#[cfg_attr(not(feature = "api"), ignore)]
async fn test_welcome_content_generation() -> Result<(), Box<dyn std::error::Error>> {
    load_env();
    tracing_subscriber::fmt::init();
    
    // Load narrative
    let narrative_path = PathBuf::from("crates/botticelli_narrative/narratives/discord/welcome_content_generation.toml");
    let narrative_repo = InMemoryNarrativeRepository::new();
    let narrative = narrative_repo.load_from_file(&narrative_path).await?;
    
    // Create executor (will need backend and content repository)
    // TODO: Set up proper backend and content repository
    
    Ok(())
}

#[tokio::test]
#[cfg_attr(not(feature = "api"), ignore)]
async fn test_publish_welcome() -> Result<(), Box<dyn std::error::Error>> {
    load_env();
    tracing_subscriber::fmt::init();
    
    // Load narrative
    let narrative_path = PathBuf::from("crates/botticelli_narrative/narratives/discord/publish_welcome.toml");
    let narrative_repo = InMemoryNarrativeRepository::new();
    let narrative = narrative_repo.load_from_file(&narrative_path).await?;
    
    // Create executor (will need backend, content repository, and bot manager)
    // TODO: Set up proper backend, content repository, and bot manager
    
    Ok(())
}

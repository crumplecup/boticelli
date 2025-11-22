//! Integration tests for Discord bot commands using test narratives.

use std::{env, path::PathBuf};

/// Helper to load environment variables from .env
fn load_env() {
    dotenvy::dotenv().ok();
}

/// Helper to get path to test narrative
fn get_test_narrative_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/narratives/discord")
        .join(format!("{}.toml", name))
}

/// Helper to run a test narrative
async fn run_test_narrative(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let narrative_path = get_test_narrative_path(name);
    
    // Use botticelli CLI to run the narrative
    let output = tokio::process::Command::new("cargo")
        .args(&[
            "run",
            "-p",
            "botticelli",
            "--bin",
            "botticelli",
            "--features",
            "gemini,discord,database",
            "--",
            "run",
            "--narrative",
            narrative_path.to_str().unwrap(),
            "--process-discord",
        ])
        .output()
        .await?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Narrative {} failed:\n{}", name, stderr);
        return Err(format!("Narrative execution failed: {}", stderr).into());
    }
    
    Ok(())
}

#[tokio::test]
#[cfg_attr(not(feature = "discord"), ignore)]
async fn test_channels_list() {
    load_env();
    run_test_narrative("channels_list_test")
        .await
        .expect("channels_list_test narrative failed");
}

#[tokio::test]
#[cfg_attr(not(feature = "discord"), ignore)]
async fn test_channels_get() {
    load_env();
    run_test_narrative("channels_get_test")
        .await
        .expect("channels_get_test narrative failed");
}

#[tokio::test]
#[cfg_attr(not(feature = "discord"), ignore)]
async fn test_messages_list() {
    load_env();
    run_test_narrative("messages_list_test")
        .await
        .expect("messages_list_test narrative failed");
}

#[tokio::test]
#[cfg_attr(not(feature = "discord"), ignore)]
async fn test_messages_send() {
    load_env();
    run_test_narrative("messages_send_test")
        .await
        .expect("messages_send_test narrative failed");
}

#[tokio::test]
#[cfg_attr(not(feature = "discord"), ignore)]
async fn test_members_list() {
    load_env();
    run_test_narrative("members_list_test")
        .await
        .expect("members_list_test narrative failed");
}

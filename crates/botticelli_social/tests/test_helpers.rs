//! Test helpers for Discord command testing with lifecycle management.

use std::path::PathBuf;
use std::process::Command;

/// Run a narrative test with setup and teardown lifecycle.
///
/// This ensures resources are cleaned up even if the test fails.
pub async fn run_test_with_lifecycle(
    setup_narrative: &str,
    test_narrative: &str,
    teardown_narrative: &str,
) -> Result<(), String> {
    // Run setup
    run_narrative_file(setup_narrative)
        .await
        .map_err(|e| format!("Setup failed: {}", e))?;

    // Run test (capture result but don't return early)
    let test_result = run_narrative_file(test_narrative).await;

    // Always run teardown, even if test failed
    let teardown_result = run_narrative_file(teardown_narrative).await;

    // Log teardown errors but don't fail if test passed
    if let Err(e) = teardown_result {
        eprintln!("Warning: Teardown failed: {}", e);
    }

    // Return test result
    test_result
}

/// Run a narrative file using the botticelli CLI.
pub async fn run_narrative_file(narrative_path: &str) -> Result<(), String> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set");
    
    let narrative_full_path = PathBuf::from(&manifest_dir)
        .join("tests/narratives/discord")
        .join(narrative_path);

    if !narrative_full_path.exists() {
        return Err(format!(
            "Narrative file not found: {}",
            narrative_full_path.display()
        ));
    }

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--bin",
            "botticelli",
            "--",
            "run",
            "--narrative",
            narrative_full_path.to_str().unwrap(),
            "--process-discord",
        ])
        .output()
        .map_err(|e| format!("Failed to execute narrative: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "Narrative execution failed:\n{}\n{}",
            stdout, stderr
        ));
    }

    Ok(())
}

/// Helper to run a test that needs a channel.
///
/// Creates a channel, runs the test, then deletes the channel.
pub async fn test_with_channel<F, Fut>(test_fn: F) -> Result<(), String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<(), String>>,
{
    run_test_with_lifecycle(
        "lifecycle/setup_channel.toml",
        &format!("lifecycle/noop.toml"), // Placeholder
        "lifecycle/teardown_channel.toml",
    )
    .await?;

    test_fn().await
}

/// Helper to run a test that needs a role.
pub async fn test_with_role<F, Fut>(test_fn: F) -> Result<(), String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<(), String>>,
{
    run_test_with_lifecycle(
        "lifecycle/setup_role.toml",
        &format!("lifecycle/noop.toml"),
        "lifecycle/teardown_role.toml",
    )
    .await?;

    test_fn().await
}

/// Helper to run a test that needs a channel and a message.
pub async fn test_with_channel_and_message<F, Fut>(test_fn: F) -> Result<(), String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<(), String>>,
{
    run_test_with_lifecycle(
        "lifecycle/setup_channel_and_message.toml",
        &format!("lifecycle/noop.toml"),
        "lifecycle/teardown_channel.toml", // Deleting channel deletes messages
    )
    .await?;

    test_fn().await
}

use botticelli_error::{BotticelliResult, ConfigError};
use botticelli_narrative::{NarrativeState, StateManager, StateScope};
use tempfile::TempDir;

#[test]
fn test_state_manager_save_and_load() -> BotticelliResult<()> {
    let temp_dir = TempDir::new().map_err(|e| ConfigError::new(e.to_string()))?;
    let manager = StateManager::new(temp_dir.path())?;
    let scope = StateScope::Global;

    // Create and save state
    {
        let mut state = NarrativeState::new();
        state.set("test_key", "test_value");
        manager.save(&scope, &state)?;
    }

    // Load state and verify
    {
        let state = manager.load(&scope)?;
        assert_eq!(state.get("test_key"), Some("test_value"));
    }

    Ok(())
}

#[test]
fn test_state_manager_persistence_across_runs() -> BotticelliResult<()> {
    let temp_dir = TempDir::new().map_err(|e| ConfigError::new(e.to_string()))?;
    let manager = StateManager::new(temp_dir.path())?;
    let scope = StateScope::Narrative("test".to_string());

    // First run: create and save
    {
        let mut state = NarrativeState::new();
        state.set("channel_id", "123456789");
        state.set("message_id", "987654321");
        manager.save(&scope, &state)?;
    }

    // Second run: load and verify
    {
        let state = manager.load(&scope)?;
        assert_eq!(state.get("channel_id"), Some("123456789"));
        assert_eq!(state.get("message_id"), Some("987654321"));
    }

    // Third run: load, modify, save
    {
        let mut state = manager.load(&scope)?;
        state.set("new_key", "new_value");
        manager.save(&scope, &state)?;
    }

    // Fourth run: verify all values
    {
        let state = manager.load(&scope)?;
        assert_eq!(state.get("channel_id"), Some("123456789"));
        assert_eq!(state.get("message_id"), Some("987654321"));
        assert_eq!(state.get("new_key"), Some("new_value"));
    }

    Ok(())
}

#[test]
fn test_cli_workflow_simulation() -> BotticelliResult<()> {
    let temp_dir = TempDir::new().map_err(|e| ConfigError::new(e.to_string()))?;
    let state_dir = temp_dir.path();

    // Simulate first CLI run with --save flag
    {
        let manager = StateManager::new(state_dir)?;
        let scope = StateScope::Narrative("test_narrative".to_string());

        let mut state = NarrativeState::new();
        state.set("TEST_CHANNEL_ID", "1234567890");
        state.set("TEST_MESSAGE_ID", "0987654321");

        manager.save(&scope, &state)?;

        // Verify it was saved
        let loaded = manager.load(&scope)?;
        assert_eq!(loaded.get("TEST_CHANNEL_ID"), Some("1234567890"));
    }

    // Simulate second CLI run loading state
    {
        let manager = StateManager::new(state_dir)?;
        let scope = StateScope::Narrative("test_narrative".to_string());

        let state = manager.load(&scope)?;
        assert_eq!(state.get("TEST_CHANNEL_ID"), Some("1234567890"));
        assert_eq!(state.get("TEST_MESSAGE_ID"), Some("0987654321"));
    }

    Ok(())
}

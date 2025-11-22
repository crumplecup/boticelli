# Discord Write Operations Testing Strategy

## Problem Statement

Write operations in Discord (creating channels, roles, messages, etc.) need:
1. **Proper cleanup** - Resources must be deleted after tests
2. **State tracking** - IDs must be cached for reuse and teardown
3. **Dependency ordering** - Some operations depend on others (e.g., send message requires channel)
4. **Error resilience** - Tests should clean up even on failure
5. **Isolation** - Tests shouldn't interfere with each other

## Current Approach Issues

- Manual setup/teardown in each test is repetitive
- State management isn't integrated with test lifecycle
- No automatic cleanup on test failure
- Resource leakage when tests fail

## Proposed Solution: Narrative-Based Test Lifecycle

### Architecture

```
Test Function
  ├─ Setup Narrative (creates resources, stores IDs)
  ├─ Test Narrative (exercises command with stored IDs)
  └─ Teardown Narrative (deletes resources by ID)
```

### Key Components

#### 1. State Management Integration

Use `NarrativeStateManager` to:
- Store created resource IDs during setup
- Retrieve IDs during test execution
- Track resources for cleanup

```toml
# Setup narrative stores ID in state management
[narrative]
name = "setup_test_channel"
skip_content_generation = true

[[acts]]
name = "create_channel"

[[acts.inputs]]
type = "bot_command"
platform = "discord"
command = "channels.create"
required = true

[acts.inputs.args]
guild_id = "${TEST_GUILD_ID}"
name = "test-channel"
# ID automatically captured and stored in state as TEST_CHANNEL_ID

# Test narrative retrieves ID from state
[narrative]
name = "test_channel_get"
skip_content_generation = true

[[acts]]
name = "get_channel"

[[acts.inputs]]
type = "bot_command"
platform = "discord"
command = "channels.get"
required = true

[acts.inputs.bot_command.args]
channel_id = "{state:test_channel_id}"  # Retrieved from state management
```

#### 2. Narrative Lifecycle Helpers

Create test helpers that manage the full lifecycle:

```rust
/// Run a test with automatic setup/teardown
async fn run_test_with_lifecycle(
    setup_narrative: &str,
    test_narrative: &str,
    teardown_narrative: &str,
) -> Result<(), String> {
    let state = NarrativeStateManager::new()?;
    
    // Setup phase
    run_narrative(setup_narrative, &state)
        .await
        .map_err(|e| format!("Setup failed: {}", e))?;
    
    // Test phase
    let test_result = run_narrative(test_narrative, &state).await;
    
    // Teardown phase (always runs, even on failure)
    let teardown_result = run_narrative(teardown_narrative, &state).await;
    
    // Return first error
    test_result.and(teardown_result)
        .map_err(|e| format!("Test failed: {}", e))
}
```

#### 3. Standard Narrative Patterns

**Setup Narrative Pattern:**
```toml
[narrative]
name = "setup_test_channel"
description = "Create a test channel"
skip_content_generation = true

[[acts]]
name = "create_channel"

[[acts.inputs]]
type = "bot_command"
platform = "discord"
command = "channels.create"
required = true

[acts.inputs.args]
guild_id = "${TEST_GUILD_ID}"
name = "botticelli-test-channel"
kind = "text"
```

**Setup Narrative (creates and caches channel ID):**
```toml
[narrative]
name = "setup_test_channel"
description = "Create a test channel and cache its ID"
skip_content_generation = true

[[acts]]
name = "create_channel"

[[acts.inputs]]
type = "bot_command"
platform = "discord"
command = "channels.create"
required = true

[acts.inputs.args]
guild_id = "$TEST_GUILD_ID"
name = "botticelli-test-channel"
channel_type = "0"
```

**Test Narrative (uses cached channel ID):**
```toml
[narrative]
name = "test_channels_get"
description = "Test getting a channel"
skip_content_generation = true

[[acts]]
name = "get_channel"

[[acts.inputs]]
type = "bot_command"
platform = "discord"
command = "channels.get"
required = true

[acts.inputs.args]
channel_id = "{{create_channel.channel_id}}"
```

**Teardown Narrative (deletes channel using cached ID):**
```toml
[narrative]
name = "teardown_test_channel"
description = "Delete test channel"
skip_content_generation = true

[[acts]]
name = "delete_channel"

[[acts.inputs]]
type = "bot_command"
platform = "discord"
command = "channels.delete"
required = true

[acts.inputs.args]
channel_id = "{{create_channel.channel_id}}"
```

Note: Channel ID references use the template syntax `{{act_name.field_name}}` to reference outputs from previous acts within the same narrative execution.

#### 4. Test Organization

```
crates/botticelli_social/tests/narratives/discord/
├── lifecycle/
│   ├── setup_channel.toml
│   ├── teardown_channel.toml
│   ├── setup_role.toml
│   ├── teardown_role.toml
│   └── ... (reusable setup/teardown)
├── channels/
│   ├── channels_create_test.toml
│   ├── channels_get_test.toml
│   ├── channels_update_test.toml
│   └── channels_delete_test.toml
├── messages/
│   ├── messages_send_test.toml
│   ├── messages_edit_test.toml
│   └── messages_delete_test.toml
└── ...
```

#### 5. Helper Functions

```rust
// Helper to run setup/test/teardown
async fn test_with_channel<F>(test_fn: F) -> Result<(), String>
where
    F: FnOnce(&str) -> BoxFuture<'static, Result<(), String>>,
{
    run_test_with_lifecycle(
        "lifecycle/setup_channel.toml",
        test_fn,
        "lifecycle/teardown_channel.toml",
    ).await
}

// Usage in tests
#[tokio::test]
async fn test_channels_get() {
    test_with_channel(|channel_id| async move {
        run_narrative("channels/channels_get_test.toml").await
    }).await.unwrap();
}
```

## Implementation Plan

### Phase 1: Core Infrastructure (Completed)
- ✅ NarrativeStateManager implementation
- ✅ State variable substitution in narratives
- ✅ Basic lifecycle pattern

### Phase 2: Lifecycle Helpers
1. Create `run_test_with_lifecycle` helper
2. Add `test_with_channel`, `test_with_role`, etc. helpers
3. Implement timestamp/unique naming for resources
4. Add error handling and cleanup guarantees

### Phase 3: Standard Lifecycles
1. Create lifecycle narratives for:
   - Channels (text, voice, category)
   - Roles
   - Messages
   - Threads
   - Webhooks
   - Invites
2. Document patterns in narrative library

### Phase 4: Test Migration
1. Migrate existing tests to use lifecycle helpers
2. Add tests for remaining commands
3. Verify cleanup works on failure
4. Add integration test that validates no resource leakage

### Phase 5: Documentation
1. Update DISCORD_COMMAND_TESTING_STRATEGY
2. Add examples to narrative documentation
3. Create testing guide for contributors

## Best Practices

### Resource Naming
- Use timestamps or UUIDs: `test-channel-${TIMESTAMP}`
- Tag resources: `[TEST] My Channel` for manual identification
- Store creation metadata in state

### Error Handling
- Always run teardown, even on test failure
- Log teardown failures but don't fail the test
- Track orphaned resources for manual cleanup

### State Management
- Use clear naming: `TEST_CHANNEL_ID`, `TEST_ROLE_ID`
- Scope state to test execution
- Clear state after teardown

### Test Isolation
- Each test creates its own resources
- Don't share resources between tests
- Use unique names to avoid conflicts

## Example: Complete Test Lifecycle

```rust
#[tokio::test]
async fn test_message_pin() {
    // Helper handles setup/teardown
    test_with_channel_and_message(async |channel_id, message_id| {
        // Test narrative uses stored IDs
        let result = run_narrative_test("messages/messages_pin_test.toml").await;
        
        // Verify result
        assert!(result.is_ok());
        
        // Check message is actually pinned
        let pins = get_pinned_messages(channel_id).await?;
        assert!(pins.iter().any(|p| p.id == message_id));
        
        Ok(())
    }).await.unwrap();
    // Teardown runs automatically, even if test panics
}
```

## Benefits

1. **DRY Principle** - Reusable setup/teardown narratives
2. **Safety** - Guaranteed cleanup prevents resource leakage
3. **Clarity** - Narrative files document test scenarios
4. **Maintainability** - Changes to setup logic in one place
5. **Reliability** - Tests clean up after themselves
6. **Debuggability** - State management logs all operations

## Next Steps

1. Implement `run_test_with_lifecycle` helper
2. Create standard lifecycle narratives
3. Migrate 3-5 tests as proof of concept
4. Validate cleanup works on failures
5. Roll out to all write operation tests

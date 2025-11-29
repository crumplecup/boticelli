# Discord Command Testing Strategy

## Status: Narrative-Based Testing Established ✅

We've successfully implemented a comprehensive narrative-based testing infrastructure:
- **30 passing tests** covering Discord bot commands
- Tests use TOML narratives in `crates/botticelli_social/tests/narratives/discord/`
- Established state management pattern for test resources
- Resource lifecycle management (setup → use → teardown)
- All tests integrated into `discord_command_test.rs`

**Current Coverage:** 30+ commands tested including channels, messages, members, roles, emojis, invites, webhooks

**Next Steps:** Continue expanding command coverage and add more complex workflow tests.

## Current State Analysis

### Problems Identified

1. **Narrative-based tests are complex** - Each test requires:
   - Creating TOML narrative files
   - Setting up database connections
   - Running full executor pipeline
   - Hard to isolate individual command failures

2. **Missing test infrastructure**:
   - No direct Discord API testing helpers
   - No way to test commands without full narrative execution
   - Difficult to verify command outputs

3. **Test coverage gaps**:
   - Many commands have no tests at all
   - Existing tests only cover happy paths
   - No error condition testing
   - No validation of command arguments

### Root Cause

The current architecture couples command testing to the narrative executor. We need a way to test commands in isolation.

## Recommended Testing Strategy

### Phase 1: Narrative-Based Command Testing (IMPLEMENTED ✅)

We use TOML narratives to test Discord commands in a realistic execution environment:

```toml
# tests/narratives/discord/channels_list_test.toml
[metadata]
name = "channels_list_test"
description = "Test listing channels in a guild"

[[acts]]
name = "list_channels"

[[acts.inputs]]
type = "bot_command"
platform = "discord"
command = "channels.list"
required = false

[acts.inputs.args]
guild_id = "{{TEST_GUILD_ID}}"
```

**Benefits:**
- Tests commands in realistic narrative context
- Validates TOML parsing and execution pipeline
- Reusable patterns for production narratives
- State management integration
- Easy to add multi-step workflows

### Phase 2: Command Argument Validation

Test that commands properly validate their arguments:

```rust
#[test]
fn test_channels_get_missing_channel_id() {
    let bot = DiscordBot::new(&token).unwrap();
    let args = BotCommandArgs::new()
        .with_arg("guild_id", "123");
    // Missing channel_id
    
    let result = bot.execute("channels.get", args);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("channel_id"));
}
```

### Phase 3: State Management Pattern (IMPLEMENTED ✅)

We use state management to track test resource IDs across narratives:

**Setup Narrative:**
```toml
# setup_test_channel.toml - Creates channel and stores ID in state
[[acts.inputs]]
type = "bot_command"
command = "channels.create"
[acts.inputs.args]
guild_id = "{{TEST_GUILD_ID}}"
name = "test-channel"
save_output_as = "TEST_CHANNEL_ID"
```

**Test Narrative:**
```toml
# channels_get_test.toml - Uses stored channel ID
[[acts.inputs]]
type = "bot_command"
command = "channels.get"
[acts.inputs.args]
channel_id = "{{TEST_CHANNEL_ID}}"  # Retrieved from state
```

**Teardown Narrative:**
```toml
# teardown_test_channel.toml - Cleans up using stored ID
[[acts.inputs]]
type = "bot_command"
command = "channels.delete"
[acts.inputs.args]
channel_id = "{{TEST_CHANNEL_ID}}"
```

**Test Runner Helper:**
```rust
fn run_test_narrative(name: &str) -> String {
    let narrative_path = format!("tests/narratives/discord/{}.toml", name);
    let output = Command::new("cargo")
        .args(["run", "--bin", "botticelli", "--", "run", "--narrative", &narrative_path])
        .output()
        .expect("Failed to run narrative");
    assert!(output.status.success(), "Narrative {} failed", name);
    String::from_utf8(output.stdout).unwrap()
}
```

### Phase 4: Test Fixtures and Cleanup

Create test utilities for:
- Creating temporary test channels
- Cleaning up after tests
- Shared test state (e.g., test guild setup)

```rust
struct TestGuild {
    guild_id: String,
    bot: DiscordBot,
    created_channels: Vec<String>,
}

impl TestGuild {
    fn new() -> Self { /* ... */ }
    
    fn create_test_channel(&mut self, name: &str) -> String {
        // Create and track channel
    }
}

impl Drop for TestGuild {
    fn drop(&mut self) {
        // Clean up all created channels
    }
}
```

## Implementation Plan

### Step 1: Create Narrative-Based Test Suite ✅ DONE

File: `crates/botticelli_social/tests/discord_command_test.rs`

Test coverage by command category:
- ✅ Guild operations (guilds.get)
- ✅ Channel operations (list, get, create, delete, update)
- ✅ Message operations (send, get, list, delete, pin, unpin)
- ✅ Member operations (list, get, kick, ban, unban)
- ✅ Role operations (list, get, create, delete, assign, remove)
- ✅ Emoji operations (list, get, create, delete)
- ✅ Invite operations (list, create, delete)
- ✅ Webhook operations (list, create, delete, execute)
- ✅ Thread operations (create, list, join, leave)
- ⚠️ Advanced operations (scheduled events, stage instances, etc.) - TODO

### Step 2: Add Argument Validation Tests

For each command, test:
- Missing required arguments
- Invalid argument formats
- Boundary conditions

### Step 3: Simplify Narrative Tests

Consolidate narrative tests into workflow tests:
- `test_channel_creation_workflow` - Create → verify → cleanup
- `test_message_publishing_workflow` - Send → pin → verify
- `test_content_generation_workflow` - Generate → select → publish

### Step 4: Add Test Utilities

Create `crates/botticelli_social/tests/test_utils/discord.rs`:
- `create_test_bot()` - Initialize bot with test token
- `create_temporary_channel()` - Create and auto-cleanup
- `send_test_message()` - Send and auto-cleanup
- `assert_discord_id()` - Validate ID format

## Success Criteria

1. **Coverage**: Every implemented command has at least one passing test
2. **Speed**: Direct command tests run in <5 seconds total
3. **Reliability**: Tests pass consistently (not flaky)
4. **Clarity**: Test failures clearly indicate what command/argument failed
5. **Cleanup**: Tests leave no artifacts in Discord server

## Current Testing Architecture

1. ✅ Created `discord_command_test.rs` with narrative-based testing
2. ✅ Established test narrative pattern in `tests/narratives/discord/`
3. ✅ Implemented state management for test resource IDs
4. ✅ Created setup/teardown narratives for resource lifecycle
5. ✅ Helper function `run_test_narrative()` for test execution
6. ✅ 30+ passing tests covering major command categories
7. ⚠️ Continue expanding coverage for remaining commands

## Open Questions

1. **Rate Limiting**: How do we handle Discord API rate limits in tests?
   - Solution: Use test guilds with generous rate limits, or add delays between tests

2. **Permissions**: What permissions does the test bot need?
   - Solution: Document required permissions in test README

3. **Cleanup Failures**: What if cleanup fails (e.g., network error)?
   - Solution: Log cleanup failures but don't fail tests; provide manual cleanup script

4. **Parallel Testing**: Can we run tests in parallel?
   - Solution: Start with serial execution, add parallelism later with proper isolation

# Bot Command Output State Capture Implementation

## Problem

Write operation tests need to capture IDs from bot command outputs (like `channel_id` from `channels.create`) and store them in state management for use in subsequent acts/narratives.

Currently:
- Bot commands execute and return output
- Output is logged but not captured
- State shows "Available keys: none"
- Subsequent acts can't reference the created resources

## Requirements

1. **Capture bot command outputs** - Extract JSON responses from Discord API
2. **Store in state management** - Save extracted values with meaningful keys
3. **Template substitution** - Allow `${STATE.channel_id}` syntax in narratives
4. **Persistence** - State must survive across narrative invocations (setup → test → teardown)

## Proposed Solution

### 1. Act Output Capture Syntax

```toml
[acts.create_channel]

[[acts.create_channel.input]]
type = "bot_command"
platform = "discord"
command = "channels.create"
required = true

[acts.create_channel.input.args]
guild_id = "${TEST_GUILD_ID}"
name = "botticelli-write-test"

# NEW: Capture outputs to state
[acts.create_channel.outputs]
channel_id = "$.id"  # JSONPath to extract channel ID
channel_name = "$.name"
```

### 2. Implementation Steps

#### Step 1: Extend ActConfig with outputs field
```rust
// In botticelli_narrative/src/core.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActConfig {
    // ... existing fields ...
    
    /// Output capture configuration - maps state keys to JSONPath expressions
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub outputs: HashMap<String, String>,
}
```

#### Step 2: Capture bot command responses
```rust
// In botticelli_narrative/src/executor.rs
// After bot command execution:
if let Some(bot_output) = act_output.bot_command_result {
    // Parse JSON response
    let json_value: serde_json::Value = serde_json::from_str(&bot_output)?;
    
    // Apply output mappings
    for (state_key, jsonpath) in &act_config.outputs {
        let value = extract_jsonpath(&json_value, jsonpath)?;
        state_manager.set(state_key, value)?;
    }
}
```

#### Step 3: State persistence
- State manager already saves to disk: `~/.config/botticelli/state/{narrative_name}.json`
- Ensure `--state-dir` flag is honored
- Verify state loads across invocations

#### Step 4: Template substitution
- Extend existing `${ENV.VAR}` syntax
- Add `${STATE.key}` for state lookups
- Example: `channel_id = "${STATE.channel_id}"`

### 3. Test Flow

```toml
# setup.toml - Creates channel, captures ID
[acts.create]
[[acts.create.input]]
type = "bot_command"
command = "channels.create"
[acts.create.input.args]
guild_id = "${TEST_GUILD_ID}"
[acts.create.outputs]
channel_id = "$.id"  # Saved to state

# test.toml - Uses captured ID
[acts.update]
[[acts.update.input]]
type = "bot_command"
command = "channels.update"
[acts.update.input.args]
channel_id = "${STATE.channel_id}"  # Retrieved from state
topic = "Updated topic"

# teardown.toml - Cleans up using captured ID
[acts.delete]
[[acts.delete.input]]
type = "bot_command"
command = "channels.delete"
[acts.delete.input.args]
channel_id = "${STATE.channel_id}"  # Retrieved from state
```

### 4. JSONPath Library

Use `jsonpath_lib` or `serde_json_path` for extraction:
```toml
[dependencies]
serde_json_path = "0.6"
```

### 5. Error Handling

- Invalid JSONPath → NarrativeError::OutputCaptureFailed
- Missing state key → NarrativeError::TemplateError (already exists)
- JSON parse failure → NarrativeError::OutputParseFailed

## Benefits

1. **Reusable test resources** - Create once, use many times
2. **Proper cleanup** - Delete resources using stored IDs
3. **Test isolation** - Each test run creates fresh resources
4. **Flexible extraction** - JSONPath supports nested fields
5. **Persistent state** - Works across narrative invocations

## Implementation Priority

1. ✅ State manager disk persistence (already done)
2. ⏱️ Output capture syntax parsing
3. ⏱️ JSONPath extraction from bot responses
4. ⏱️ STATE template substitution
5. ⏱️ Integration tests

## Next Steps

1. Add `outputs` field to ActConfig
2. Implement JSONPath extraction
3. Extend template engine for `${STATE.key}`
4. Update test narratives to use new syntax
5. Document in NARRATIVE_TOML_SPEC

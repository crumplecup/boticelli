# State Persistence Issue

## Problem

State is not persisting between narrative executions, even when using `--state-dir` flag.

### Observed Behavior

1. Setup narrative (`channel_create_setup.toml`) creates a channel via Discord bot command
2. Bot command executor captures the `channel_id` and saves it to state
3. Test narrative (`channel_update_test.toml`) runs immediately after
4. Test narrative tries to reference `${state:channel_id}`
5. **State key 'channel_id' not found. Available keys: none**

### Test Output

```
Stderr: Error: BotticelliError(Narrative(NarrativeError { 
  kind: TemplateError("State key 'channel_id' not found. Available keys: none"), 
  line: 900, 
  file: "crates/botticelli_narrative/src/executor.rs" 
}))
```

## Investigation

### State Manager Implementation

State manager exists in `botticelli_narrative/src/state.rs`:
- `load(&self, scope: &StateScope)` - loads state from disk
- `save(&self, scope: &StateScope, state: &NarrativeState)` - saves state to disk
- Uses JSON files in `state_dir` for persistence

### Executor Integration

In `executor.rs`:
- Line 170: State is loaded at start of `capture_bot_command_ids()`
- Line 209-210: IDs are saved to state (both long and short keys)
- Line 225: State is saved back to disk if IDs were captured
- Line 575: `capture_bot_command_ids()` is called after bot command execution

### CLI Integration

In `botticelli/src/cli/run.rs`:
- Lines 99-105: State manager is configured when `--state-dir` is provided
- State manager is passed to executor via `with_state_manager()`

## Hypothesis

The state IS being saved after the setup narrative completes, but when the test narrative loads, it's loading an empty state. Possible causes:

1. **Scope mismatch**: Setup saves to one scope, test loads from another
2. **Timing issue**: State file write isn't flushing before next narrative starts
3. **File path issue**: State files are being written/read from different locations
4. **State manager not configured**: Test narratives might not be getting state manager instance

## Next Steps

1. **Add debug logging** to confirm:
   - Setup narrative is actually saving state
   - Test narrative is loading from correct location
   - File system shows state file exists and contains data

2. **Verify scope consistency**: Both narratives should use `StateScope::Global`

3. **Check file permissions**: Ensure state directory is writable and files are readable

4. **Test with single CLI invocation**: Run both setup and test in one narrative file to eliminate inter-process state issues

5. **Manual state file inspection**: After running setup, check `/tmp/botticelli_test_state/global.json` exists and contains `channel_id`

## Related Code

- `crates/botticelli_narrative/src/state.rs` - State manager implementation
- `crates/botticelli_narrative/src/executor.rs:162-235` - ID capture logic
- `crates/botticelli_narrative/src/executor.rs:575` - ID capture invocation
- `crates/botticelli/src/cli/run.rs:99-105` - State manager configuration
- `crates/botticelli_social/tests/narratives/discord/write_tests/` - Test narratives

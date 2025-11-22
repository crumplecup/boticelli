# Discord Command Testing Strategy

## Current Status

We've implemented many Discord bot commands but comprehensive testing is still needed.

## Testing Challenges

1. **Database Dependency**: BotticelliBot requires a PostgreSQL connection, making unit tests more complex
2. **Authentication**: Tests need valid DISCORD_TOKEN and TEST_GUILD_ID
3. **Integration Testing**: Commands interact with real Discord API, requiring careful setup/teardown
4. **Rate Limits**: Need to be mindful of API rate limits during testing

## Recommended Approach

### Option 1: Narrative-Based Integration Tests (Preferred)

Use narrative files to test commands end-to-end:
- Store test narratives in `crates/botticelli_social/tests/narratives/discord/`
- Each command gets a minimal narrative that exercises it
- Run via `just narrate <test_narrative>`
- Manual verification in Discord test server

**Pros:**
- Tests real workflow that users will use
- Exercises full stack including database
- Easy to create and maintain
- Visual verification in Discord

**Cons:**
- Requires manual setup (database, Discord token)
- Not automated CI-friendly (yet)

### Option 2: Direct API Tests with Test Database

Create integration tests that:
- Set up test database connection
- Create BotticelliBot instance
- Execute commands directly
- Verify results

**Pros:**
- More traditional test structure
- Can be automated
- Faster feedback

**Cons:**
- Requires database fixtures
- More complex test infrastructure
- Still needs Discord API access

## Implementation Plan

### Phase 1: Manual Narrative Testing (Current)

1. Create minimal test narratives for each command category:
   - `test_channels.toml` - Channel operations
   - `test_messages.toml` - Message operations
   - `test_roles.toml` - Role operations
   - etc.

2. Document test procedures in this file
3. Manually verify each command works

### Phase 2: Automated Integration Tests (Future)

1. Create test database setup helper
2. Build `TestBot` wrapper that handles setup/teardown
3. Implement cargo tests that exercise commands
4. Add to CI pipeline with feature gates

## Current Test Coverage

### Implemented Commands

See DISCORD_API_COVERAGE_ANALYSIS.md for full list.

### Manual Test Status

- [ ] channels.list - Tested via narratives
- [ ] channels.create - Tested via setup_channels narrative
- [ ] messages.send - Tested via publish_welcome narrative
- [ ] messages.pin - Tested via publish_welcome narrative
- [ ] guilds.get - Not yet tested
- [ ] roles.list - Not yet tested
- (... more to be added)

## Running Tests Manually

```bash
# Set up environment
export DISCORD_TOKEN="your-token"
export TEST_GUILD_ID="your-guild-id"
export DATABASE_URL="postgresql://user:pass@localhost/botticelli"

# Test channel operations
just narrate crates/botticelli_social/tests/narratives/discord/test_channels

# Test message operations  
just narrate crates/botticelli_social/tests/narratives/discord/test_messages

# etc.
```

## Next Steps

1. Complete narrative-based manual testing for all implemented commands
2. Document results in this file
3. Design automated test infrastructure
4. Implement automated tests
5. Add to CI pipeline

## Notes

- Tests consume Discord API rate limits - be mindful
- Test guild should be dedicated test server, not production
- Some commands require specific permissions - document requirements
- Consider mocking Discord API for unit tests (future enhancement)

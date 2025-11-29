# Feature Gate Audit

## Summary

Analysis of `just check-features` output reveals several categories of issues across feature combinations.

## Issue Categories

### 1. Truly Dead Code (All Feature Combinations)

These appear in **every** feature combination and are genuinely unused:

**botticelli_narrative:**
- `NarrativeExecutor::execute_multi()` - Method never called anywhere
- `format_schema_for_prompt()` - Function never called anywhere

**Fix:** Either use them or remove them. No feature gates needed.

### 2. Missing Feature Declarations

**botticelli_actor** references non-existent features:
- `#[cfg(feature = "cron")]` - Feature doesn't exist in Cargo.toml
- `#[cfg(feature = "scheduled")]` - Feature doesn't exist in Cargo.toml

**Fix:** Either:
- Add these features to `crates/botticelli_actor/Cargo.toml`
- Remove the `#[cfg]` gates if features aren't needed
- Change to use existing features

### 3. Incomplete Implementations (Stub Fields)

**botticelli_actor:**
- `ActorState::registered_at` - Field defined but never read
- `RateLimitingSkill::state` - Field defined but never used
- `DiscordPlatform::token` (when `discord` feature enabled) - Field stored but never used

These are **implementation incomplete**, not feature gate issues.

**Fix:** Complete the implementation to use these fields, or remove them if not needed yet.

### 4. Database Model Fields (Legitimate - Future Use)

**botticelli_social** (when `discord` feature enabled):

- **GuildRow**: 28 unused fields (features, description, vanity_url_code, member_count, etc.)
- **GuildMemberRow**: 10 unused fields (nick, avatar, joined_at, premium_since, etc.)
- **UserRow**: 11 unused fields (bot, system, mfa_enabled, verified, etc.)

**Status:** These are database models matching Discord API schema. Fields exist for:
1. Schema completeness
2. Future feature development
3. Data persistence integrity

**Current Fix:** Already properly using `derive_getters` to provide accessors. The warnings are acceptable because:
- Diesel requires all table columns in the struct
- We'll use these fields as we build features
- Removing them would break database schema

**Future:** Add `#[allow(dead_code)]` with explanation, OR implement features that use these fields.

### 5. Missing Documentation

**botticelli_social:**
- 17 public struct fields missing docs in GuildRow, GuildMemberRow, UserRow

**Fix:** Add `///` doc comments to all public fields.

## Fixes Required

### Priority 1: Remove Genuinely Dead Code

```rust
// crates/botticelli_narrative/src/executor.rs:324
// Either use or delete execute_multi()

// crates/botticelli_narrative/src/storage_actor.rs:543  
// Either use or delete format_schema_for_prompt()
```

### Priority 2: Fix Feature Declarations

```toml
# crates/botticelli_actor/Cargo.toml
[features]
cron = []  # Add if needed
scheduled = []  # Add if needed
```

OR remove the `#[cfg(feature = "cron")]` and `#[cfg(feature = "scheduled")]` attributes.

### Priority 3: Complete Stub Implementations

**ActorState::registered_at:**
- Use it in health checks or metrics
- Or remove if not tracking registration time

**RateLimitingSkill::state:**
- Implement rate limiting logic that uses this HashMap
- Or remove if using different approach

**DiscordPlatform::token:**
- Use token for Discord API calls
- Or remove if handled elsewhere

### Priority 4: Document Public Fields

Add docs to all public fields in:
- `botticelli_social/src/discord/models/guild.rs`
- `botticelli_social/src/discord/models/member.rs`
- `botticelli_social/src/discord/models/user.rs`

### Priority 5: Database Models

Decision needed:
- Keep as-is with explanation that fields are for future use
- Add selective `#[allow(dead_code)]` with clear comments
- Implement features that use the fields

## Feature Combinations Tested

All combinations pass compilation, warnings only:

1. ✓ no-default-features
2. ✓ default-features  
3. ✓ all-features
4. ✓ gemini only
5. ✓ database only
6. ✓ discord only
7. ✓ tui only
8. ✓ gemini + database
9. ✓ gemini + discord
10. ✓ database + tui

**Clippy** `-D warnings` fails on truly dead code (Priority 1 above).

## Notes on #[allow(dead_code)]

Currently using in:
- None found during audit

**Philosophy:** `#[allow]` directives hide real problems. We should:
1. Fix the underlying issue (use the code, feature-gate it, or remove it)
2. Only use `#[allow]` for Diesel-generated code or explicit architectural decisions
3. Document WHY with clear comments when `#[allow]` is necessary

## Action Plan

1. **Immediate:** Remove/fix truly dead code (execute_multi, format_schema_for_prompt)
2. **Short-term:** Fix feature declarations and complete stub implementations
3. **Short-term:** Add documentation to public fields
4. **Long-term:** Implement features that use database model fields

## Status

- [ ] Priority 1: Dead code removal
- [ ] Priority 2: Feature declarations
- [ ] Priority 3: Complete stubs
- [ ] Priority 4: Documentation
- [ ] Priority 5: Database model decision

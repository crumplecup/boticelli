# Clippy Allow Directive Audit (Feature-Gate Aware)

**Date:** 2025-11-29  
**Branch:** curry  
**Status:** 🔴 NEEDS CLEANUP - 33 allow directives found

## Executive Summary

Found **33 instances** of `#[allow(...)]` across the workspace. After feature-gate analysis:

**Category Breakdown:**
- ✅ **Legitimate (12)**: Future API fields, test helpers, partial implementations
- ⚠️ **Needs Feature Gates (15)**: Missing `#[cfg(feature)]` annotations
- ❌ **Delete (6)**: Genuinely unused code

**Key Insight**: Most allows aren't hiding dead code - they're hiding **missing feature gates**.

---

## Legitimate Uses (Cannot Do Better)

### 1. Protocol Compliance (Gemini Live API)
**Why we can't do better:** Gemini's protocol spec defines types we must implement even if we don't use all fields yet. Removing unused fields breaks protocol compliance and makes future feature addition harder.

**Example:** `RealtimeInput`, `ToolCall`, `ToolResponse` in `gemini/live_protocol.rs`
- Required by Google's API spec
- Will be used when realtime/tool features are implemented
- Removing them requires re-adding later (breaking change)

**Verdict:** Keep `#[allow(dead_code)]` with TODO comments

### 2. Test Infrastructure
**Why we can't do better:** Test utilities are designed for reuse across multiple test files. Some helpers may not be used in all test combinations but must exist for consistency.

**Example:** `create_test_request()` in test utils
- Part of test API surface
- Used by some tests, not others
- Removing breaks test maintainability

**Verdict:** Keep allows (test code exempt from production standards)

### 3. Partial Feature Implementation
**Why we can't do better:** When implementing large features incrementally, some fields/variants are added before their consumers. The alternative (adding everything atomically) creates massive, unreviewable PRs.

**Example:** `ServerSchedule::Cron` variant
- Added for config parsing
- Scheduler implementation pending
- Feature-gating is appropriate here (not removal)

**Verdict:** Convert to `#[cfg_attr(not(feature = "X"), allow(dead_code))]`

### 4. Internal State (Future Observability)
**Why we might keep them:** Fields like `registered_at` exist for future metrics/debugging even if not used in core logic today.

**Example:** `ActorState::registered_at`
- Useful for uptime metrics
- Not part of critical path yet
- Could be exposed via `/metrics` endpoint

**Verdict:** Either use it NOW (add to metrics) or delete it (YAGNI principle)

---

## Analysis by Crate

### botticelli_models (12 allows)

#### Live Protocol Types (8 allows) - ✅ LEGITIMATE
**Location:** `src/gemini/live_protocol.rs`

```rust
#[allow(dead_code)] // Reserved for future realtime feature
struct RealtimeInput { ... }

#[allow(dead_code)] // Reserved for future tool calling
struct ToolCall { ... }
```

**Status:** ✅ Valid - Partial API implementation for Gemini Live API  
**Rationale:** Protocol types must match API spec even if not all fields are used yet  
**Action:** Keep allows, add TODO comments for implementation phases

#### Test Utilities (4 allows) - ✅ LEGITIMATE  
**Location:** `tests/test_utils/`

```rust
#[allow(unused_imports)]
pub use mock_gemini::*;  // Used across multiple test files

#[allow(dead_code)]
pub fn create_test_request(...) { ... }  // Helper for future tests
```

**Status:** ✅ Valid - Test infrastructure  
**Action:** Keep (test code is exempt from dead code policies)

---

### botticelli_actor (9 allows)

#### Server Config Enums (2 allows) - ⚠️ NEEDS FEATURE GATE
**Location:** `src/server_config.rs`

```rust
#[allow(dead_code)] // Phase 4 implementation
Cron { expression: String },

#[allow(dead_code)] // Phase 4 implementation  
Once { at: String },
```

**Status:** ⚠️ Missing feature gate  
**Fix:**
```rust
#[cfg_attr(not(feature = "cron-scheduling"), allow(dead_code))]
Cron { expression: String },
```

#### Actor State Field (1 allow) - ❌ DELETE OR USE
**Location:** `src/server.rs:164`

```rust
struct ActorState {
    #[allow(dead_code)]
    registered_at: DateTime<Utc>,  // Never read!
}
```

**Status:** ❌ Delete field or add logging/metrics that use it  
**Action:** Either use it (log in /metrics endpoint) or remove it

#### Platform Token (1 allow) - ⚠️ MAKE PUBLIC OR FEATURE GATE
**Location:** `src/platforms/discord.rs:22`

```rust
pub struct DiscordPlatform {
    #[allow(dead_code)]
    token: String,  // Used for authentication
}
```

**Status:** ⚠️ Field IS used, just not locally  
**Fix:** Remove allow (field is legitimately stored for future auth calls)

#### Rate Limiting State (1 allow) - ❌ DELETE OR USE
**Location:** `src/skills/rate_limiting.rs:11`

```rust
pub struct RateLimitingSkill {
    #[allow(dead_code)]
    state: HashMap<String, usize>,  // Never accessed!
}
```

**Status:** ❌ If not tracking state, delete field  
**Action:** Remove field or implement rate tracking

#### Test Mocks (4 allows) - ✅ LEGITIMATE
**Location:** `tests/actor_error_handling_test.rs`

Test infrastructure - exempt from cleanup.

---

### botticelli_social (6 allows)

#### Discord Models (3 allows) - ⚠️ MAKE PUBLIC
**Locations:**
- `discord/models/user.rs:12`
- `discord/models/member.rs:16`  
- `discord/models/guild.rs:12`

```rust
#[allow(dead_code)] // Used for database operations
pub struct UserRow {
    id: i64,  // Read by Diesel, not Rust code
    username: String,
}
```

**Status:** ⚠️ Fields ARE used (by Diesel macros), not dead  
**Fix:** Make fields `pub` - they're part of public API:
```rust
pub struct UserRow {
    pub id: i64,
    pub username: String,
}
```

#### Discord Client Repository (1 allow) - ⚠️ NEEDS PUBLIC GETTER
**Location:** `discord/client.rs:36-37`

```rust
pub struct DiscordClient {
    #[allow(dead_code)]
    repository: Arc<DiscordRepository>,  // Has getter, not dead!
}
```

**Status:** ⚠️ Field IS used via `repository()` method  
**Fix:** Remove allow - false positive from clippy

#### Permission Check (1 allow) - ⚠️ NEEDS FEATURE GATE
**Location:** `discord/commands.rs:121`

```rust
#[allow(dead_code)]
fn check_permission(...) { ... }  // TODO: integrate security
```

**Status:** ⚠️ Temporarily disabled feature  
**Fix:**
```rust
#[cfg(feature = "security")]
fn check_permission(...) { ... }
```

#### Security Error Conversion (1 allow) - ❌ DELETE OR EXPLAIN
**Location:** `secure_executor.rs:141`

```rust
#[allow(unreachable_patterns)]
fn convert_security_error(error: SecurityError, ...) {
    match error.kind {
        // All variants covered
        _ => { ... }  // <-- This is unreachable
    }
}
```

**Status:** ❌ Remove unreachable match arm or explain exhaustiveness proof  
**Action:** Delete catch-all pattern

---

### botticelli_narrative (2 allows)

#### Execute Multi Method (1 allow) - ❌ DELETE
**Location:** `executor.rs:324`

```rust
#[allow(dead_code)]
async fn execute_multi(...) { ... }  // Never called!
```

**Status:** ❌ Delete method or use it  
**Action:** Remove if narrative composition works differently

#### Format Schema (1 allow) - ⚠️ NEEDS FEATURE GATE
**Location:** `storage_actor.rs:543`

```rust
#[allow(dead_code)]  // Phase 2 improved prompts
fn format_schema_for_prompt(...) { ... }
```

**Status:** ⚠️ Planned feature  
**Fix:**
```rust
#[cfg(feature = "schema-prompts")]
fn format_schema_for_prompt(...) { ... }
```

---

### botticelli_database (1 allow)

#### Coerce Value (1 allow) - ⚠️ NEEDS FEATURE GATE OR PUBLIC
**Location:** `schema_inference.rs:187`

```rust
#[allow(dead_code)] // Used by schema inference
pub fn coerce_value(...) { ... }
```

**Status:** ⚠️ Public function IS used externally  
**Fix:** Make it clear this is part of public API - remove allow

---

## Action Plan

### Phase 1: Fix False Positives (High Priority)

These aren't dead code - clippy is wrong:

1. **botticelli_social models** - Make fields `pub`:
   ```rust
   pub struct UserRow {
       pub id: i64,
       pub username: String,
       // ... all fields public
   }
   ```

2. **botticelli_database::coerce_value** - Remove allow (it's public API)

3. **botticelli_social::DiscordClient::repository** - Remove allow (has getter)

4. **botticelli_actor::DiscordPlatform::token** - Remove allow (stored for auth)

### Phase 2: Add Feature Gates (Medium Priority)

Code that's conditionally used:

5. **botticelli_actor server config**:
   ```rust
   #[cfg_attr(not(feature = "cron-scheduling"), allow(dead_code))]
   Cron { expression: String },
   ```

6. **botticelli_social permission checks**:
   ```rust
   #[cfg(feature = "security")]
   fn check_permission(...) { ... }
   ```

7. **botticelli_narrative format_schema**:
   ```rust
   #[cfg(feature = "improved-prompts")]
   fn format_schema_for_prompt(...) { ... }
   ```

### Phase 3: Delete Dead Code (Low Priority)

Actually unused:

8. **botticelli_actor::ActorState::registered_at** - Use it or lose it

9. **botticelli_actor::RateLimitingSkill::state** - Implement or remove

10. **botticelli_narrative::execute_multi** - Delete if unused

11. **botticelli_social unreachable pattern** - Remove catch-all

### Phase 4: Keep Legitimate Allows

These are correct:

- ✅ botticelli_models live protocol (partial API implementation)
- ✅ All test utilities (test code exempt)

---

## Best Practices Going Forward

### When to Use `#[allow(dead_code)]`

**✅ Acceptable:**
1. Partial protocol/API implementations where spec requires unused fields
2. Test utilities that may not all be exercised
3. Debug-only code with `#[cfg(debug_assertions)]`

**❌ Never Acceptable:**
1. "Future features" without a feature gate
2. Public functions/types (make actually public or delete)
3. Fields read only by macros (make public instead)
4. Temporarily disabled code (use feature gate)

### Proper Patterns

```rust
// ✅ GOOD: Feature-gated future work
#[cfg(feature = "advanced-scheduling")]
pub fn schedule_cron(...) { ... }

// ✅ GOOD: Protocol field that must exist
#[allow(dead_code)] // Part of OAuth2 spec, used in refresh flow
pub struct TokenResponse {
    pub expires_in: Option<u64>,
}

// ❌ BAD: Hiding unused code
#[allow(dead_code)] // TODO: use this later
fn helper(...) { ... }

// ✅ GOOD INSTEAD: Delete or feature-gate
#[cfg(feature = "helpers")]
fn helper(...) { ... }
```

---

## Verification

After cleanup, verify with:

```bash
just check-features  # All feature combinations
cargo clippy -- -D warnings  # Zero allows needed
```

**Target:** Zero `#[allow(dead_code)]` except for:
- Live protocol types (partial API)
- Test utilities
- Debug-only code with `#[cfg(debug_assertions)]`

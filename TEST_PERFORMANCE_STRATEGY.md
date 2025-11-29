# Test Performance Optimization Strategy

## Status: ✅ RESOLVED - No Binary Tests Found

After investigation, **there are no binary integration tests** in the codebase. All tests properly use `#[test]`/`#[tokio::test]` and are located in `tests/` directories per CLAUDE.md.

The `[[test]]` declarations in Cargo.toml are standard integration test declarations with feature requirements, not binary tests.

## Slow Test Categories (Legitimate)

### Database Tests (~10-30s)
- Perform actual database operations  
- Create/read/update schema
- Test data persistence

**Why slow**: Real PostgreSQL I/O  
**Not a problem**: Testing real functionality

### Discord Command Tests (~5-15s)
- Execute full narrative workflows
- Parse TOML configurations
- Interact with database

**Why slow**: Complex multi-subsystem integration  
**Not a problem**: Correctly feature-gated

### API Tests (>30s)
- Make real LLM API calls
- Network latency + model processing
- Token-based rate limiting

**Why slow**: External API calls  
**Solution**: Feature-gated behind `api` feature

## Recommendations

1. **Keep current structure** - follows CLAUDE.md correctly
2. **Use nextest** - already provides parallelization
3. **Feature gates** - already properly applied
4. **Monitor regressions** - run `just test-timings` periodically

## Archived Analysis (Obsolete)

### ~~Option 1: Shared Binary Compilation (Quick Win)~~
**Impact**: Eliminate ~4m compile time per test
**Effort**: Low

```rust
// Use once_cell to compile binary once for all tests
use once_cell::sync::Lazy;

static COMPILED_BINARY: Lazy<PathBuf> = Lazy::new(|| {
    // Compile once, use in all tests
    compile_botticelli_binary()
});
```

**Estimated Improvement**: 137s → 90s per test (save 47s)

### Option 2: Mock Discord API (Best Practice)
**Impact**: Eliminate network I/O entirely
**Effort**: Medium

- Create mock Discord server using `wiremock` or `mockito`
- Record real API responses, replay in tests
- Tests run in milliseconds instead of minutes

**Estimated Improvement**: 137s → 0.1s per test (save 99.9%)

### Option 3: Parallel Test Execution
**Impact**: Run all 8 tests concurrently
**Effort**: Low (nextest already supports this)

```bash
cargo nextest run --test-threads=8
```

**Caveat**: May hit Discord rate limits harder
**Estimated Improvement**: 19m total → 2.5m total (8x speedup)

### Option 4: Feature-Gate Integration Tests
**Impact**: Don't run by default
**Effort**: Low

```rust
#[test]
#[cfg_attr(not(feature = "api"), ignore)]
fn test_channels_list() { ... }
```

```bash
# Fast dev cycle
cargo test

# Full validation
cargo test --features api
```

**Estimated Improvement**: Default test suite: 143s → 3s

## Recommendation

**Root cause:** Testing binaries instead of library code.

**Solution:** 
1. Refactor `main.rs` to thin wrapper around `lib.rs`
2. Test library functions directly
3. Keep 1-2 smoke tests for binary/CLI validation

**Why binary tests are slow:**
- Spawn subprocess
- Re-compile on every test
- Process startup overhead
- Cannot share state

## Implementation Plan

### Phase 1: Feature Gating (Now)
```rust
// Mark slow Discord API tests
#[cfg_attr(not(feature = "api"), ignore)]
fn test_channels_list() { ... }
```

### Phase 2: Shared Binary
```rust
// tests/discord_command_test.rs
use once_cell::sync::Lazy;

static BINARY: Lazy<PathBuf> = Lazy::new(|| {
    std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("botticelli")
});
```

### Phase 3: Mock API (Future)
- Use `wiremock` crate
- Record real responses → `tests/fixtures/discord_responses.json`
- Fast, deterministic, no rate limits

## Metrics
- Current: 143s total (19m for Discord tests)
- After Phase 1: 3s default, 143s with `--features api`
- After Phase 2: 3s default, 90s with `--features api`
- After Phase 3: 3s default, 3s with mocks, 90s with real API

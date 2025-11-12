# Gemini Model Selection Issue

## Quick Summary

**Status**: Tests written, implementation plan finalized
**Priority**: High (blocks multi-model narratives)

**The Bug**: `GeminiClient` ignores the `model` field in `GenerateRequest`, causing all API calls to use the same default model instead of the requested model.

**The Fix**:
1. Make `RateLimiter<T: Tier>` generic to take ownership of any tier type
2. Create `TieredGemini<T>` that couples `Gemini` client with tier
3. Store single `HashMap<String, RateLimiter<TieredGemini<GeminiTier>>>` in `GeminiClient`
4. Each model gets its own rate-limited client, created lazily via `Gemini::with_model()`

**Architecture Benefits**:
- Type-safe: Cannot access client without going through rate limiter
- Efficient: One HashMap, no `Box<dyn Tier>` overhead
- Clean ownership: RateLimiter owns the TieredGemini

**Next Steps**: See Phase 2 in Implementation Plan below (make RateLimiter generic).

---

## Problem Statement

The `GeminiClient` implementation has a critical bug where the model specified in `GenerateRequest.model` or the narrative's per-act model configuration is completely ignored. All API calls use whatever default model the `gemini-rust` crate provides (likely `gemini-2.5-flash`), regardless of what model name is requested.

## Root Cause Analysis

### Architecture Mismatch

1. **Boticelli API Design**: Supports per-request model selection via `GenerateRequest.model`
2. **gemini-rust Crate**: Sets model at client creation time via `Gemini::with_model(api_key, model_name)`

### Current Implementation Issues

**File**: `src/models/gemini.rs`

**Line 206**: Default model set but never used
```rust
model_name: "gemini-2.0-flash".to_string(),
```

**Line 198**: Client created without model specification
```rust
let client = Gemini::new(api_key)
    .map_err(|e| GeminiError::new(GeminiErrorKind::ClientCreation(e.to_string())))?;
```

**Line 232-316**: `generate_internal()` method ignores `req.model`
- Uses `req.max_tokens` (lines 245, 301) ✓
- Uses `req.temperature` (lines 296-298) ✓
- **Never uses `req.model`** ✗

## Example of the Problem

**Narrative**: `narrations/text_models.toml`
- Act 1: Requests `gemini-2.0-flash-lite`
- Act 2: Requests `gemini-2.0-flash`
- Act 3: Requests `gemini-2.5-flash-lite`

**Actual Behavior**: All three acts use the same model (likely `gemini-2.5-flash` default)

## Impact

1. **Narrative system broken**: Cannot execute multi-model workflows
2. **Cost unpredictability**: May be using more expensive models than intended
3. **Feature testing impossible**: Cannot validate behavior across different Gemini models
4. **API contract violation**: `BoticelliDriver` trait promises to respect `GenerateRequest.model`

## Solution: TieredGemini with Generic RateLimiter

Use a `TieredGemini` type that couples the model client with its tier, and make `RateLimiter` generic over `T: Tier` to take ownership of the tier.

### Architecture Overview

**TieredGemini** - Couples client with tier:
```rust
struct TieredGemini<T: Tier> {
    client: Gemini,
    tier: T,
}

impl<T: Tier> Tier for TieredGemini<T> {
    // Delegate all Tier methods to self.tier
    fn rpm(&self) -> Option<u32> { self.tier.rpm() }
    fn tpm(&self) -> Option<u64> { self.tier.tpm() }
    // ... etc
}
```

**Generic RateLimiter** - Takes ownership of any `T: Tier`:
```rust
pub struct RateLimiter<T: Tier> {
    inner: T,
    rpm_limiter: Option<Arc<DirectRateLimiter>>,
    tpm_limiter: Option<Arc<DirectRateLimiter>>,
    rpd_limiter: Option<Arc<DirectRateLimiter>>,
    concurrent_semaphore: Arc<Semaphore>,
}

impl<T: Tier> RateLimiter<T> {
    pub fn new(tier: T) -> Self {
        // Extract rate limits from tier to build governor limiters
        // Store tier as self.inner
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }
}
```

**GeminiClient** - Single HashMap storing rate-limited clients:
```rust
struct GeminiClient {
    // Single HashMap: model name -> rate-limited client
    clients: Arc<Mutex<HashMap<String, RateLimiter<TieredGemini<GeminiTier>>>>>,
    api_key: String,
    default_model: String,
    default_tier: GeminiTier,
}

async fn generate_internal(&self, req: &GenerateRequest) -> GeminiResult<GenerateResponse> {
    let model_name = req.model.as_ref().unwrap_or(&self.default_model);

    // Get or create rate-limited client for this model
    let rate_limited_client = {
        let mut clients = self.clients.lock().unwrap();
        clients.entry(model_name.clone())
            .or_insert_with(|| {
                let gemini = Gemini::with_model(&self.api_key, model_name).unwrap();
                let tiered = TieredGemini {
                    client: gemini,
                    tier: self.default_tier.clone(), // Or model-specific tier
                };
                RateLimiter::new(tiered)
            })
            .clone() // TODO: Check if RateLimiter needs Arc or Clone
    };

    // Acquire rate limit
    let _guard = rate_limited_client.acquire(estimated_tokens).await;

    // Access the client through the rate limiter
    let response = rate_limited_client.inner().client.generate_content()...;
}
```

### Benefits

1. **Type safety**: Cannot access client without going through rate limiter
2. **Single source of truth**: Model + tier + rate limits are coupled
3. **Ownership model**: RateLimiter owns the TieredGemini, enforcing controlled access
4. **Cleaner code**: One HashMap instead of two
5. **Generic pattern**: Can use `RateLimiter<T>` for other providers too
6. **No Box<dyn Trait>**: Direct storage eliminates dynamic dispatch overhead

## Testing Requirements

### Unit Tests Needed

1. **Model name validation**: Verify correct model is used for each request
2. **Model override**: Test that `GenerateRequest.model` overrides default
3. **Model caching**: Verify clients are reused correctly for same model
4. **Default model fallback**: When `req.model` is None
5. **Per-model rate limiting**: Each model has independent rate limiter with correct limits

### Integration Tests Needed

1. **Narrative multi-model execution**: Run `narrations/text_models.toml`
2. **Model metadata**: Verify `model_name()` returns correct value
3. **Rate limit independence**: Verify different models don't share rate limits

## Implementation Plan

### Phase 1: Tests ✓

- [x] Create `tests/gemini_model_test.rs`
- [x] Test: Model name is respected in API calls
- [x] Test: Default model is used when `req.model` is None
- [x] Test: Model override works in narrative execution

### Phase 2: Make RateLimiter Generic

**File**: `src/rate_limit/limiter.rs`

- [ ] Change `RateLimiter` from struct to `RateLimiter<T: Tier>`
- [ ] Replace `_tier: Box<dyn Tier>` with `inner: T`
- [ ] Update `new(tier: Box<dyn Tier>)` to `new(tier: T)`
- [ ] Extract rate limits from `tier` using trait methods (same as before)
- [ ] Add `pub fn inner(&self) -> &T` method to access the inner value
- [ ] Update docstrings to reflect generic parameter
- [ ] Verify that `RateLimiter` is `Clone` if `T: Clone` (may need `#[derive(Clone)]` with bounds)

**Breaking Changes**:
- `RateLimiter::new()` signature changes from `Box<dyn Tier>` to generic `T`
- Code using `RateLimiter` needs to specify type parameter

### Phase 3: Create TieredGemini Type

**File**: `src/models/gemini.rs` (add before `GeminiClient`)

- [ ] Create `struct TieredGemini<T: Tier> { client: Gemini, tier: T }`
- [ ] Implement `Tier for TieredGemini<T>` by delegating all methods to `self.tier`
- [ ] Add `#[derive(Clone)]` if `Gemini` and `T` are both `Clone`
- [ ] Add documentation explaining the type couples client with tier

### Phase 4: Refactor GeminiClient Structure

**File**: `src/models/gemini.rs`

- [ ] Replace fields:
  - Remove: `client: Gemini`, `rate_limiter: Option<RateLimiter>`
  - Add: `clients: Arc<Mutex<HashMap<String, RateLimiter<TieredGemini<GeminiTier>>>>>`
  - Add: `api_key: String`
  - Add: `default_tier: GeminiTier`
  - Keep: `model_name: String` (stores default model)

- [ ] Update `new_internal()`:
  - Store `api_key` field
  - Convert `tier: Option<Box<dyn Tier>>` to `GeminiTier` (or default to Free)
  - Initialize empty `clients` HashMap
  - Don't create any clients yet (lazy creation in generate_internal)

- [ ] Update `Debug` impl for new structure

### Phase 5: Implement Per-Request Model Selection

**File**: `src/models/gemini.rs` - `generate_internal()` method

- [ ] Extract model name: `let model_name = req.model.as_ref().unwrap_or(&self.model_name);`
- [ ] Get or create rate-limited client:
  ```rust
  let rate_limited_client = {
      let mut clients = self.clients.lock().unwrap();
      clients.entry(model_name.clone())
          .or_insert_with(|| {
              let client = Gemini::with_model(&self.api_key, model_name)
                  .map_err(|e| GeminiError::new(GeminiErrorKind::ClientCreation(e.to_string())))?;
              let tiered = TieredGemini { client, tier: self.default_tier };
              RateLimiter::new(tiered)
          })
  };
  ```
- [ ] Acquire rate limit: `let _guard = rate_limited_client.acquire(estimated_tokens).await;`
- [ ] Access client: `rate_limited_client.inner().client.generate_content()...`
- [ ] Remove old rate limiting code
- [ ] Handle errors from client creation in or_insert_with

### Phase 6: Fix Backward Compatibility

**Files**: Various

- [ ] Check `src/main.rs` CLI code - may still use `Box<dyn Tier>`
- [ ] Consider keeping `new_with_tier(Option<Box<dyn Tier>>)` as wrapper that converts to GeminiTier
- [ ] Or accept breaking change and update callers
- [ ] Update tests that use RateLimiter directly

### Phase 7: Update Supporting Methods

**File**: `src/models/gemini.rs`

- [ ] `model_name()` - keep returning `&self.model_name` (default model)
- [ ] `Metadata` impl - verify still correct
- [ ] Update docstrings to explain client pooling

### Phase 8: Testing and Validation

- [ ] Run `cargo check` to verify compilation
- [ ] Run `cargo test` (unit tests without API key)
- [ ] Run `cargo test --features gemini` (with API key set)
- [ ] Validate `narrations/text_models.toml` uses correct models
- [ ] Run `cargo clippy` and address warnings
- [ ] Verify thread safety of HashMap access

### Phase 9: Documentation

- [ ] Update docstrings to explain model selection
- [ ] Add examples showing model override
- [ ] Document default model behavior
- [ ] Note the client pooling implementation in module docs
- [ ] Update USAGE_TIERS.md if needed for new RateLimiter API

### Phase 10: Model-Specific Rate Limits (Future Work)

**Note**: Deferred to follow-up PR. Initial implementation uses tier-level rate limits for all models.

- [ ] Extend `boticelli.toml` schema to support per-model rate limits
- [ ] Update `BoticelliConfig` to parse model-specific configuration
- [ ] Modify rate limiter creation to look up model-specific limits
- [ ] Fall back to tier-level limits if model-specific config not found
- [ ] Document the new configuration format

## Migration Challenges

### Challenge 1: RateLimiter Clone/Arc

**Issue**: HashMap needs to clone/share the `RateLimiter<TieredGemini>`. Options:

1. **Make RateLimiter Clone**: Add `#[derive(Clone)]` - all the governor limiters and semaphore are already `Arc`-wrapped, so cloning is cheap
2. **Wrap in Arc**: Store `Arc<RateLimiter<TieredGemini>>` in HashMap
3. **Use entry API differently**: Keep `&mut` reference without cloning

**Recommendation**: Option 1 - make `RateLimiter` cloneable. The internal state is already Arc-based.

### Challenge 2: Error Handling in or_insert_with

**Issue**: `or_insert_with` closure must return `T`, but `Gemini::with_model()` can fail.

**Solutions**:
1. **Panic on error**: `.unwrap()` in the closure (acceptable for initialization errors)
2. **Pre-validate outside HashMap**: Check model name before lock
3. **Return Result from generate_internal**: Propagate error, but awkward with entry API
4. **Two-phase creation**: Check outside lock, insert inside lock

**Recommendation**: Option 1 initially (panic on client creation failure), refine in Phase 6 if needed.

### Challenge 3: Gemini Client Cloneability

**Issue**: Need to verify if `gemini_rust::Gemini` implements `Clone`.

**Investigation needed**:
- Check gemini-rust source/docs for Clone impl
- If not Clone, wrap in Arc: `TieredGemini { client: Arc<Gemini>, tier: T }`
- Or redesign to not require cloning

### Challenge 4: Converting Box<dyn Tier> to GeminiTier

**Issue**: `GeminiClient::new_with_tier(Option<Box<dyn Tier>>)` takes dynamic trait object, but we need concrete `GeminiTier`.

**Solutions**:
1. **Downcast**: Use `Any` trait to downcast `Box<dyn Tier>` to `GeminiTier`
2. **Change API**: Make `new_with_tier(Option<GeminiTier>)` - breaking change
3. **Keep Box<dyn Tier>**: Store in GeminiClient, but then can't use generic RateLimiter
4. **Enum dispatch**: Use GeminiTier enum, match on tier name from string

**Recommendation**: Option 2 or 4 - accept API change to enable generic architecture.

## Decisions Made

1. **Architecture** ✓: Use `TieredGemini<T: Tier>` with generic `RateLimiter<T>`
   - Couples model client with tier
   - RateLimiter takes ownership of TieredGemini
   - Single HashMap instead of two separate ones

2. **Rate limiting** ✓: Each model gets its own rate limiter
   - Different models have different rate limits per tier
   - Flash vs Flash-Lite have different quotas
   - RateLimiter owns the tier information

3. **Client lifecycle** ✓: No manual cleanup
   - Clients persist for program lifetime
   - Acceptable for CLI usage patterns
   - Minimal memory overhead (only creates clients for models actually used)

4. **Crate selection** ✓: Stick with gemini-rust
   - Use client pool to work around API design mismatch
   - Minimal HashMap contention expected

5. **Type system** ✓: Generic `T: Tier` instead of `Box<dyn Tier>`
   - Eliminates dynamic dispatch overhead
   - Better type safety
   - Cleaner ownership model

## Model-Specific Rate Limits Challenge

### Current Configuration

The `boticelli.toml` configuration defines rate limits at the tier level:

```toml
[providers.gemini.tiers.free]
rpm = 10                    # Requests per minute
tpm = 250_000              # Tokens per minute
rpd = 250                  # Requests per day
```

However, different Gemini models have different rate limits even within the same tier. For example:
- `gemini-2.0-flash`: Higher limits, more expensive
- `gemini-2.0-flash-lite`: Lower limits, cheaper
- `gemini-2.5-flash`: Latest model, potentially different limits

### Configuration Options

**Option A: Nested model configuration** (Recommended)
```toml
[providers.gemini.tiers.free.models."gemini-2.0-flash"]
rpm = 10
tpm = 250_000

[providers.gemini.tiers.free.models."gemini-2.0-flash-lite"]
rpm = 15  # Lite model has higher RPM but similar TPM
tpm = 250_000
```

**Option B: Separate model-tier keys**
```toml
[providers.gemini.model_tiers."gemini-2.0-flash".free]
rpm = 10
tpm = 250_000
```

**Option C: Fallback to tier defaults**
- Start with tier-level defaults
- Allow per-model overrides only when needed
- Most models inherit tier limits

### Implementation Strategy

For Phase 5, use **Option A with fallback**:
1. Look for model-specific config: `providers.gemini.tiers.{tier}.models.{model}`
2. If not found, fall back to tier-level config: `providers.gemini.tiers.{tier}`
3. This allows gradual migration: new models get defaults, special cases get overrides

## Open Questions

1. **Model-specific rate limits**: How to implement in configuration?
   - **Decision needed**: Choose Option A, B, or C above
   - Requires extending BoticelliConfig parsing
   - See Phase 5 in Implementation Plan

2. **Default model**: What should the default be?
   - Current: `gemini-2.0-flash`
   - Latest stable: `gemini-2.5-flash`
   - Most cost-effective: `gemini-2.0-flash-lite`
   - Let user configure in `boticelli.toml`?

3. **Model validation**: Should we validate model names?
   - Check against known models list?
   - Let API return errors for invalid models? (simpler, more maintainable)

4. **Clone behavior**: Can `Gemini` clients be cloned safely?
   - Need to verify gemini-rust's `Gemini` type supports clone
   - If not, wrap in `Arc` instead of cloning

## References

- gemini-rust docs: <https://docs.rs/gemini-rust>
- gemini-rust source: Check Cargo.lock for repository URL
- Gemini API models: <https://ai.google.dev/gemini-api/docs/models/gemini>

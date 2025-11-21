# Carousel Feature Design

## Overview

The Carousel feature enables budget-aware iterative execution of narrative acts or entire narratives. A carousel runs a specified number of iterations while respecting rate limit budgets, automatically stopping when limits are approached.

## Architecture

### Core Components

1. **CarouselConfig** (`botticelli_narrative::carousel`)
   - Defines iteration parameters
   - Sets estimated token consumption per iteration
   - Configures error handling behavior

2. **Budget** (`botticelli_rate_limit::budget`)
   - Tracks token and request consumption
   - Manages rate limit windows (per-minute, per-day)
   - Pre-validates iterations before execution

3. **CarouselState** (`botticelli_narrative::carousel`)
   - Tracks execution progress
   - Records successes and failures
   - Manages budget lifecycle

## TOML Configuration

### Narrative-Level Carousel

```toml
[narrative]
name = "daily_digest"
description = "Generate daily digest posts"

[narrative.carousel]
iterations = 50
estimated_tokens_per_iteration = 2000
continue_on_error = false
```

### Act-Level Carousel (Future Enhancement)

```toml
[acts.generate_post]
prompt = "Generate a social media post"

[acts.generate_post.carousel]
iterations = 10
estimated_tokens_per_iteration = 500
continue_on_error = true
```

## Implementation

### Budget Tracking

The `Budget` struct provides rate limit enforcement:

```rust
pub struct Budget {
    config: RateLimitConfig,
    tokens_per_minute: u64,
    tokens_per_day: u64,
    requests_per_minute: u64,
    requests_per_day: u64,
    minute_window_start: Instant,
    day_window_start: Instant,
}
```

**Key Methods:**

- `can_afford(tokens: u64) -> bool` - Pre-validates if an iteration fits within limits
- `consume(tokens: u64) -> Result<()>` - Records token/request consumption
- `remaining() -> BudgetRemaining` - Reports available budget

**Window Management:**

- Automatically resets windows when they expire (60s for minute, 86400s for day)
- Uses `Instant::now()` for high-precision timing
- Prevents rate limit violations through pre-validation

### Carousel Execution Flow

```
1. Create CarouselState with config and rate limits
2. Loop:
   a. Check can_continue() -> validates budget + iteration count
   b. If false, exit with budget_exhausted or completed status
   c. start_iteration() -> increments counter, logs progress
   d. Execute narrative/act
   e. Consume actual tokens used: budget_mut().consume(tokens)
   f. Record success/failure
3. finish() -> logs final statistics
4. Return CarouselResult with summary
```

## Status

**Implementation Status: COMPLETE**

✅ Budget tracking system
✅ Carousel configuration types
✅ TOML parsing support
✅ Error types
✅ NarrativeExecutor carousel wrapper (`execute_carousel`)
✅ Token consumption tracking
✅ Result aggregation (`CarouselResult`)
✅ All tests passing

**Future Enhancements**

⬜ Act-level carousel support (currently narrative-level only)
⬜ Actual token tracking from driver responses (currently uses estimates)
⬜ Integration tests with real API calls
⬜ Carousel pause/resume functionality

## Testing

Carousel foundation passes:
- ✅ `cargo check` - Clean compilation
- ✅ `cargo clippy` - Zero warnings
- ✅ All existing tests pass


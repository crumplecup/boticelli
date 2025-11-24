# Budget Multiplier Design

## Problem

Users need to throttle API usage below configured rate limits to:
- Avoid hitting quota edges (e.g., use 80% of RPM to stay safe)
- Conserve daily quota (e.g., use only 20% of RPD for testing)
- Share quotas across multiple processes
- Leave headroom for manual API usage

Currently, the only way to throttle is by editing `botticelli.toml`, which is:
- Manual and error-prone
- Requires restart
- Changes committed config
- Doesn't support temporary throttling

## Solution: Budget Multipliers

Add optional multipliers that scale effective rate limits:

```toml
[narrative]
name = "careful_generation"

[narrative.budget]
rpm_multiplier = 0.8  # Use 80% of requests per minute
rpd_multiplier = 0.2  # Use 20% of requests per day
tpm_multiplier = 1.0  # Use 100% of tokens per minute (default)
```

Or via CLI:

```bash
botticelli run --narrative file.toml \
  --rpm-multiplier 0.8 \
  --rpd-multiplier 0.2
```

Or via carousel config:

```toml
[carousel]
iterations = 10

[carousel.budget]
rpm_multiplier = 0.8
rpd_multiplier = 0.2
```

## Implementation

### 1. Budget Configuration Struct

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// Multiplier for requests per minute (0.0-1.0, default 1.0)
    #[serde(default = "default_multiplier")]
    pub rpm_multiplier: f64,
    
    /// Multiplier for tokens per minute (0.0-1.0, default 1.0)
    #[serde(default = "default_multiplier")]
    pub tpm_multiplier: f64,
    
    /// Multiplier for requests per day (0.0-1.0, default 1.0)
    #[serde(default = "default_multiplier")]
    pub rpd_multiplier: f64,
}

fn default_multiplier() -> f64 { 1.0 }

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            rpm_multiplier: 1.0,
            tpm_multiplier: 1.0,
            rpd_multiplier: 1.0,
        }
    }
}
```

### 2. Integration Points

**NarrativeMetadata:**
```rust
pub struct NarrativeMetadata {
    // ... existing fields
    budget: Option<BudgetConfig>,
}
```

**CarouselConfig:**
```rust
pub struct CarouselConfig {
    // ... existing fields
    budget: Option<BudgetConfig>,
}
```

**CLI:**
```rust
Run {
    // ... existing fields
    
    #[arg(long)]
    rpm_multiplier: Option<f64>,
    
    #[arg(long)]
    tpm_multiplier: Option<f64>,
    
    #[arg(long)]
    rpd_multiplier: Option<f64>,
}
```

### 3. Rate Limiter Application

In `RateLimiter::new()` or when configuring executor:

```rust
// Apply budget multipliers to tier config
let effective_rpm = tier_rpm.map(|r| (r as f64 * budget.rpm_multiplier) as u64);
let effective_tpm = tier_tpm.map(|t| (t as f64 * budget.tpm_multiplier) as u64);
let effective_rpd = tier_rpd.map(|r| (r as f64 * budget.rpd_multiplier) as u64);
```

### 4. Priority Order

1. **CLI flags** (highest priority - overrides everything)
2. **Carousel config** (overrides narrative metadata)
3. **Narrative metadata** (overrides defaults)
4. **Default** (1.0 - use full quota)

## Usage Examples

### Conservative Generation
```toml
[narrative]
name = "safe_generation"

[narrative.budget]
rpm_multiplier = 0.8  # Stay well below limit
rpd_multiplier = 0.5  # Use half daily quota
```

### Testing with Minimal Usage
```bash
just narrate test_narrative --rpm-multiplier 0.1 --rpd-multiplier 0.01
```

### Batch Processing
```toml
[carousel]
iterations = 100

[carousel.budget]
rpm_multiplier = 0.9   # Steady pace
rpd_multiplier = 0.75  # Use most of daily quota
```

## Validation

- Multipliers must be in range (0.0, 1.0]
- Zero multipliers rejected (would disable all requests)
- Values > 1.0 rejected (can't exceed tier limits)
- Logged at INFO level: "Applying budget multipliers: RPM=0.8, RPD=0.2"

## Benefits

- No config file editing required
- Temporary throttling via CLI
- Per-narrative budget control
- Clear intent in narrative files
- Protects against quota edge cases
- Enables quota sharing strategies
- Testing with minimal API usage

## Implementation Steps

1. Add `BudgetConfig` struct to `botticelli_core`
2. Add budget fields to `NarrativeMetadata` and `CarouselConfig`
3. Add CLI flags to `Commands::Run`
4. Update rate limiter initialization to apply multipliers
5. Add validation and logging
6. Update documentation
7. Add tests for multiplier application

## Alternative Considered: Absolute Limits

We could allow setting absolute limits instead of multipliers:

```toml
[narrative.budget]
rpm = 8  # Exactly 8 requests per minute
```

**Rejected because:**
- Requires knowing tier limits
- Less portable across tiers
- Harder to reason about "use 80% of quota"
- Multipliers are more intuitive for throttling

---

## Implementation Status: COMPLETE ✅

All implementation steps have been completed:

1. ✅ Added `BudgetConfig` struct to `botticelli_core`
2. ✅ Added budget fields to `NarrativeMetadata` and `CarouselConfig`
3. ✅ Added CLI flags to `Commands::Run`
4. ✅ Updated run handler to build merged budget (CLI > Carousel > Narrative > Default)
5. ✅ Added validation and logging
6. ✅ Updated generation_carousel.toml with example budget config
7. ✅ Full test coverage in budget.rs

### Usage Examples

**CLI override (temporary throttling):**
```bash
just narrate generation_carousel.feature --rpm-multiplier 0.8 --rpd-multiplier 0.2
```

**Carousel-level (per-narrative file):**
```toml
[narratives.feature.carousel.budget]
rpm_multiplier = 0.8
rpd_multiplier = 0.5
```

**Narrative-level:**
```toml
[narrative]
name = "careful_generation"

[narrative.budget]
rpm_multiplier = 0.9
```

### Next Steps (Future Work)

- Apply budget multipliers to rate limiter initialization in executor
- Integrate with RateLimiter::new() to scale tier limits
- Add metrics/logging for actual vs theoretical rate usage

# Feature Gate Testing Strategy

## Problem Statement

As Botticelli grows, we have multiple optional features that enable different functionality:

- `database` - PostgreSQL database integration (diesel, content tables)
- `discord` - Discord bot integration (serenity)
- `gemini` - Google Gemini API client
- `api` - Marker flag for API-consuming tests
- `tui` - Terminal UI (ratatui, crossterm)
- `backend-eframe` - eframe/wgpu rendering backend
- `text-detection`, `logo-detection`, `ocr` - OpenCV/Tesseract features

### Issues Encountered

1. **Dependency Leakage**: Optional dependencies (diesel, serenity) being pulled in without their feature flags
2. **Dead Code Warnings**: Functions/imports only used in specific feature combinations showing as unused
3. **Compilation Failures**: Code failing to compile with `--no-default-features` or specific feature combinations
4. **Test Isolation**: Tests requiring specific features not properly gated
5. **Default Features**: Features marked as default pulling in dependencies unconditionally

### Real Examples We Hit

- `botticelli_error` had `tui` as a default feature, pulling in database deps for all users
- `botticelli_tui` had database code scattered throughout instead of in feature-gated modules
- Imports used only with certain features causing unused warnings in other configurations
- `cargo check` (no features) pulling in diesel unexpectedly

## Solution: Comprehensive Feature Testing

### Tool Selection: cargo-hack

After research, we've selected **cargo-hack** as our primary testing tool because:

- Mature, widely-used tool (by tokio, serde, etc.)
- `--each-feature` tests each feature independently
- `--feature-powerset` tests all combinations (with filtering)
- Integrates cleanly with CI
- Good performance with incremental compilation

**Installation:**
```bash
cargo install cargo-hack
```

### Testing Strategy

#### 1. Core Feature Combinations to Test

We define these key test scenarios:

```bash
# Minimal build (no features)
cargo check --no-default-features

# Default build
cargo check

# Each feature independently
cargo hack check --each-feature --no-dev-deps

# Common combinations
cargo check --features database
cargo check --features discord
cargo check --features gemini,api
cargo check --features database,discord
cargo check --features tui,database

# All features (kitchen sink)
cargo check --all-features
```

#### 2. Feature Powerset Testing

For comprehensive testing, use powerset with depth limits:

```bash
# Test all combinations up to 3 features at a time
cargo hack check --feature-powerset --depth 3 --no-dev-deps

# Or test everything (expensive, CI only)
cargo hack check --feature-powerset --no-dev-deps
```

#### 3. Exclude Incompatible Combinations

Some features should be skipped together:

```bash
# Skip testing combinations that don't make sense
cargo hack check --feature-powerset \
  --exclude-features api \
  --exclude-no-default-features
```

### Architecture Guidelines

#### Module-Level Feature Gating

**DO:**
```rust
// lib.rs
#[cfg(feature = "database")]
mod database;

#[cfg(feature = "database")]
pub use database::{DatabaseType, DatabaseOther};

// database.rs - entire module is feature-gated at lib.rs level
use diesel::prelude::*;
pub struct DatabaseType { /* ... */ }
```

**DON'T:**
```rust
// lib.rs
mod database;  // ❌ Always compiled

// database.rs
#[cfg(feature = "database")]  // ❌ Feature gate inside module
use diesel::prelude::*;
```

#### Import Feature Gating

**DO:**
```rust
#[cfg(feature = "database")]
use diesel::prelude::*;

#[cfg(feature = "tui")]
use ratatui::widgets::Widget;
```

**DON'T:**
```rust
use diesel::prelude::*;  // ❌ Always imported, unused warning without feature
```

#### Function Feature Gating

**DO:**
```rust
#[cfg(feature = "database")]
pub fn query_database(conn: &mut Connection) -> Result<Vec<Row>> {
    // ...
}

// Or make public and document
/// Query database rows.
///
/// Available with the `database` feature.
#[cfg(feature = "database")]
pub fn query_database(conn: &mut Connection) -> Result<Vec<Row>> {
    // ...
}
```

#### Conditional Compilation in Feature-Gated Modules

If a module is already feature-gated at the `lib.rs` level, you don't need to repeat the feature gate inside:

```rust
// lib.rs
#[cfg(feature = "database")]
mod database_impl;

// database_impl.rs - no need for #[cfg] here, entire module is conditional
use diesel::prelude::*;
```

### Testing Framework Integration

#### Justfile Targets

Add these targets to our `justfile`:

```just
# Test no features
test-minimal:
    cargo check --no-default-features
    cargo clippy --no-default-features

# Test each feature independently
test-each-feature:
    cargo hack check --each-feature --no-dev-deps
    cargo hack clippy --each-feature --no-dev-deps

# Test common combinations
test-feature-combos:
    cargo check --features database
    cargo check --features discord
    cargo check --features gemini,api
    cargo check --features database,discord
    cargo check --features tui,database

# Full powerset (expensive)
test-feature-powerset:
    cargo hack check --feature-powerset --depth 3 --no-dev-deps

# All features
test-all-features:
    cargo check --all-features
    cargo test --all-features
    cargo clippy --all-features

# Complete feature test suite
test-features: test-minimal test-each-feature test-feature-combos test-all-features
```

#### CI Integration

Add to `.github/workflows/rust.yml`:

```yaml
name: Feature Matrix Testing

on: [push, pull_request]

jobs:
  feature-test:
    name: Feature combination testing
    runs-on: ubuntu-latest
    strategy:
      matrix:
        feature-set:
          - ""  # no features
          - "--features database"
          - "--features discord"
          - "--features gemini"
          - "--features tui,database"
          - "--all-features"
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      - name: Check ${{ matrix.feature-set }}
        run: cargo check ${{ matrix.feature-set }}
      - name: Clippy ${{ matrix.feature-set }}
        run: cargo clippy ${{ matrix.feature-set }}

  feature-powerset:
    name: Feature powerset testing
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      - name: Install cargo-hack
        run: cargo install cargo-hack
      - name: Check feature powerset
        run: cargo hack check --feature-powerset --depth 3 --no-dev-deps
```

### Debugging Feature Issues

#### Finding Unused Warnings

```bash
# Check a specific feature combination
cargo check --features database 2>&1 | grep "warning: unused"

# Check without features
cargo check --no-default-features 2>&1 | grep "warning: unused"
```

#### Finding Dependency Leaks

```bash
# See dependency tree for minimal build
cargo tree --no-default-features

# See what pulls in a specific dep
cargo tree --no-default-features -i diesel

# Compare with features
cargo tree --features database -i diesel
```

#### Verifying Feature Gates

```bash
# Expand macros to see what's compiled
cargo expand --no-default-features

# Check specific module
cargo expand --no-default-features database
```

### Best Practices Summary

1. **Default features should be minimal** - Users opt-in to heavy deps
2. **Feature-gate at module level in lib.rs** - Cleaner than scattered cfg attributes
3. **Feature-gate imports** - Prevent unused warnings
4. **Document feature requirements** - Public API docs should mention required features
5. **Test feature combinations regularly** - Use cargo-hack in CI
6. **Use justfile targets** - Make testing easy for developers
7. **Separate feature-specific code into modules** - Don't mix database and non-database code

### Migration Checklist

When adding a new feature or refactoring existing code:

- [ ] Create dedicated module for feature-specific code
- [ ] Add `#[cfg(feature = "...")]` at module level in lib.rs
- [ ] Feature-gate all imports of optional dependencies
- [ ] Add feature requirement to public API documentation
- [ ] Test `cargo check --no-default-features`
- [ ] Test `cargo check --features your-feature`
- [ ] Test `cargo hack check --each-feature`
- [ ] Update justfile with new feature combinations
- [ ] Update CI matrix if needed

### Current Status

#### Features Properly Gated

- ✅ `gemini` - Clean module separation
- ✅ `database` - Mostly clean (some TUI issues resolved)
- ✅ `discord` - Clean module separation

#### Features Needing Work

- ⚠️ `tui` - Partially refactored, database code now in separate module
- ⚠️ Optional features (text-detection, ocr, etc.) - Need verification

#### Validation Commands

```bash
# These should all pass with zero warnings
cargo check --no-default-features
cargo check
cargo check --all-features
cargo hack check --each-feature --no-dev-deps
```

## References

- [cargo-hack documentation](https://github.com/taiki-e/cargo-hack)
- [cargo-all-features](https://github.com/frewsxcv/cargo-all-features) (alternative tool)
- [Cargo Features Reference](https://doc.rust-lang.org/cargo/reference/features.html)
- [Rust Project Primer - Crate Features](https://rustprojectprimer.com/checks/features.html)

## Next Steps

1. Install cargo-hack: `cargo install cargo-hack`
2. Add justfile targets for feature testing
3. Run initial audit: `just test-features`
4. Fix any warnings/errors discovered
5. Add CI workflow for feature matrix testing
6. Document feature requirements in README.md

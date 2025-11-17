# Phase 3.5 Architecture: Narrative-Database Interface Pattern

## Problem Statement
The narrative and database modules have circular dependency:
- Database needs: `Narrative`, `Act`, `NarrativeExecution`, `ActExecution`
- Narrative executor needs: `NarrativeRepository` trait for persistence
- Traditional approach creates circular dependency

## Solution: Interface Layer Pattern

### Architecture Overview
Move narrative TYPES and TRAITS to `botticelli-interface`, keeping implementations separate:

```
botticelli-interface
  ├── Narrative types: Narrative, NarrativeMetadata, NarrativeToc
  ├── Execution types: NarrativeExecution, ActExecution, ExecutionStatus
  ├── Repository types: ExecutionFilter, ExecutionSummary, VideoMetadata
  └── NarrativeRepository trait
       ↑                    ↑
       |                    |
botticelli-database    botticelli-narrative
(implements)           (uses trait + executor)
```

### Types to Move to botticelli-interface

**From `src/narrative/core.rs`:**
- `NarrativeMetadata` - Metadata structure
- `NarrativeToc` - Table of contents
- `Narrative` - Main narrative structure (needs simplification - remove ActConfig dependency)

**From `src/narrative/executor.rs`:**
- `ActExecution` - Single act execution result
- `NarrativeExecution` - Complete execution result

**From `src/narrative/repository.rs`:**
- `NarrativeRepository` trait - Main persistence interface
- `ExecutionFilter` - Query filter structure
- `ExecutionSummary` - Lightweight execution info
- `ExecutionStatus` enum - Execution state
- `VideoMetadata` - Video metadata structure

**Total:** ~500 LOC of pure data structures and trait definitions

### Dependency Resolution

**Current problematic dependencies:**
1. `Narrative` depends on `ActConfig` (from narrative/provider.rs)
2. `ActConfig` depends on `Input` (from core)
3. Types use `BotticelliResult` (from error)

**Solutions:**
1. **ActConfig** → Keep as-is or simplify `Narrative` to hold `HashMap<String, String>` in interface
2. **Input** → Already in `botticelli-core`, no issue
3. **BotticelliResult** → Already re-exported in interface, no issue

### Implementation Steps

1. **Update botticelli-interface Cargo.toml:**
   ```toml
   [dependencies]
   botticelli-error = { workspace = true }
   botticelli-core = { workspace = true }
   serde = { workspace = true }
   serde_json = { workspace = true }
   async-trait = { workspace = true }
   ```

2. **Create narrative module in botticelli-interface:**
   - `src/narrative/mod.rs` - Module root
   - `src/narrative/types.rs` - Core narrative structures
   - `src/narrative/execution.rs` - Execution types
   - `src/narrative/repository.rs` - Repository trait and types

3. **Update botticelli-interface lib.rs:**
   ```rust
   pub mod narrative;
   pub use narrative::*;
   ```

4. **Update botticelli-database:**
   ```rust
   use botticelli_interface::{
       NarrativeRepository, NarrativeExecution, ExecutionFilter,
       ExecutionSummary, ExecutionStatus
   };
   use botticelli_error::DatabaseError;
   
   pub struct PostgresNarrativeRepository {
       // implementation
   }
   
   impl NarrativeRepository for PostgresNarrativeRepository {
       // implement trait methods
   }
   ```

5. **Create botticelli-narrative:**
   - Depends on: interface, core, error, optionally database
   - Implements: `NarrativeExecutor`, processors, TOML parsing
   - Uses: `NarrativeRepository` trait for optional persistence

### Benefits

✅ **No circular dependencies** - Clean unidirectional flow  
✅ **Clear separation** - Types/traits vs implementations  
✅ **Flexible** - Database is optional for narrative execution  
✅ **Testable** - Can mock repository for unit tests  
✅ **Consistent** - Follows same pattern as `StreamChunk`, `FinishReason`  

### Current Status

- ✅ `DatabaseError` and `NarrativeError` in foundation
- ✅ Database source files copied to `crates/botticelli-database/`
- 🔨 Narrative types need to move to interface
- 🔨 Database imports need updating
- 🔨 Narrative executor crate needs creation

### Estimated Remaining Work

- **Interface types addition:** ~2 hours (careful extraction and import fixing)
- **Database crate completion:** ~3 hours (fix imports, test compilation)
- **Narrative crate creation:** ~4 hours (executor, processors, tests)
- **Integration testing:** ~2 hours

**Total:** ~11 hours of focused work

## Lessons Learned

1. **Traits + Types Together**: Interface crates should contain both the trait and all types it operates on
2. **Error Foundation**: All error types centralized in foundation prevents circular deps
3. **Pure Data Structures**: Types in interface should be pure data (no complex logic)
4. **Clear Boundaries**: Implementation details stay in implementation crates

This pattern scales well for future features and maintains clean architecture.

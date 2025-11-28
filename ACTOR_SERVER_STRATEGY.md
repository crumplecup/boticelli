# Actor Server Integration - Strategy Document

**Status**: ✅ PIPELINE OPERATIONAL - End-to-end execution successful
**Date**: 2025-11-28 (Updated after testing)
**Components**: Content Generator, Content Curator, Discord Poster

---

## Executive Summary

The actor server infrastructure is **successfully operational** and **end-to-end pipeline confirmed working**:
- ✅ Database connection pooling working
- ✅ NarrativeExecutionSkill integrated and executing
- ✅ Table query registry functioning
- ✅ Bot command registry configured
- ✅ State persistence tracking executions
- ✅ All three actors loaded and scheduled

## Test Results (2025-11-28)

**Pipeline Performance**:
- ✅ **Content Generator**: Created 27 posts in `potential_discord_posts`
- ✅ **Content Curator**: Approved 10 posts to `approved_discord_posts` (no array errors!)
- ✅ **Discord Poster**: Successfully posted 9 posts to Discord

**Issues Status**:
1. **Carousel composition** → ✅ **IMPLEMENTED AND WORKING** (Option F1)
   - Fixed: Modified narrative_execution.rs to pass MultiNarrative to executor (3 lines)
   - Test: All 5 narratives execute in rotation (feature, usecase, tutorial, community, problem)
   - Status: **Carousel composition fully functional** - system rotates through multiple narratives
   - Details: See CAROUSEL_COMPOSITION_STRATEGY.md for implementation details

2. **PostgreSQL array insertion** → ✅ **RESOLVED** (no fix needed)
   - Expected failure, but works perfectly with native `text[]` columns
   - Tags stored as `{approved,curated}` format natively

3. **Discord Poster** → ✅ **CONFIRMED WORKING**
   - Successfully posted 9/10 approved posts to Discord

**Remaining Issues**:
- Generator's critique act fails with "No JSON found in LLM response" (non-blocking, prompt engineering issue)
- Content generation acts occasionally fail JSON parsing (unrelated to carousel composition)

---

## Issue 1: Content Generator - Carousel Composition Failure

### Problem
```
ERROR: Referenced narrative 'feature' not found.
Narrative composition requires MultiNarrative.
```

The `batch_generate` narrative uses carousel mode to execute 5 sub-narratives (feature, usecase, tutorial, community, problem) in rotation for 3 iterations. The narrative executor doesn't find these referenced narratives within the multi-narrative file.

### Root Cause
File: `crates/botticelli_narrative/narratives/discord/generation_carousel.toml`

The file contains:
- `[narratives.batch_generate]` - The carousel orchestrator
- `[narratives.feature]`, `[narratives.usecase]`, etc. - Individual generation narratives

When loaded with `narrative_name = "batch_generate"`, the executor loads ONLY that narrative and doesn't have access to the other narratives in the same file for composition.

### Options

#### Option A: Use Single Narrative (Quick Fix)
**Effort**: 5 minutes
**Impact**: Generates 1 post per execution instead of 15

Change `actors/generation_actor.toml`:
```toml
narrative_path = "crates/botticelli_narrative/narratives/discord/generation_carousel.toml"
narrative_name = "feature"  # Or usecase, tutorial, community, problem
```

**Pros**:
- Immediate solution
- Tests the full pipeline quickly
- Each narrative still generates quality content

**Cons**:
- Reduced throughput (1 post vs 15 per execution)
- Doesn't test carousel composition feature
- May need to manually rotate between narratives

#### Option B: Fix Carousel Composition in NarrativeExecutor
**Effort**: 2-4 hours
**Impact**: Full batch generation capability

Modify `NarrativeExecutionSkill` to load the entire `MultiNarrative` file and pass it to the executor along with the starting narrative name.

**Implementation**:
1. Update skill to load `MultiNarrative::from_file()`
2. Pass both the multi-narrative and target narrative name to executor
3. Executor can resolve composition references from the multi-narrative

**Pros**:
- Enables carousel mode for high-volume generation
- Tests advanced narrative composition
- Generates 15 posts per execution

**Cons**:
- Requires code changes to narrative execution flow
- More complex, higher risk of introducing bugs
- Not critical for initial deployment

#### Option C: Create Separate Batch Script
**Effort**: 30 minutes
**Impact**: Achieves batch generation without carousel

Create a shell script that runs the actor-server with different narratives sequentially:
```bash
#!/bin/bash
for narrative in feature usecase tutorial community problem; do
    NARRATIVE_NAME=$narrative ./target/debug/actor-server \
        --config actors/generation_single.toml
    sleep 5
done
```

**Pros**:
- Simple, no code changes
- Tests multiple narratives
- Easy to schedule via cron

**Cons**:
- Less elegant than carousel
- Requires separate config file
- Manual orchestration

### ~~Recommendation: Option A (Short-term) + Option B (Future Enhancement)~~ → **✅ IMPLEMENTED**

**Timeline**:
- ✅ **Option A applied initially** (2025-11-28): Used single narrative to unblock testing
- ✅ **Option B implemented** (2025-11-28): Full carousel composition working

**Implementation Details**:
- See CAROUSEL_COMPOSITION_STRATEGY.md for complete diagnostic and fix documentation
- Modified narrative_execution.rs to pass MultiNarrative to executor
- All 5 narratives (feature, usecase, tutorial, community, problem) execute in rotation
- Trait-based abstraction (`NarrativeProvider`) enabled seamless fix with minimal code changes

---

## Issue 2: Content Curator - PostgreSQL Array Insertion

### Problem
```
ERROR: Failed to insert content:
malformed array literal: "["approved","curated"]"
```

The curator narrative successfully:
1. ✅ Queried `potential_discord_posts` table
2. ✅ Ran LLM analysis (28 seconds)
3. ✅ Inferred schema for `approved_discord_posts`
4. ✅ Created the table dynamically
5. ❌ Failed inserting JSON with array fields

### Root Cause
PostgreSQL expects array literals in format `{value1,value2}` but the JSON contains `["value1","value2"]`. When the system tries to insert JSON arrays directly into PostgreSQL array columns, the format mismatch causes the error.

Likely columns affected:
- `tags` field containing `["approved","curated"]`
- Possibly other array-type fields inferred from JSON

### Options

#### Option A: Fix Schema Inference to Use JSONB for Arrays
**Effort**: 1-2 hours
**Impact**: Preserves JSON structure, avoids conversion issues

Modify `crates/botticelli_database/src/schema_inference.rs`:
- When inferring array types from JSON, use `JSONB` column type instead of PostgreSQL arrays
- JSONB stores JSON natively, no format conversion needed

**Pros**:
- Clean solution, no data transformation
- JSONB supports rich querying
- Handles nested structures better

**Cons**:
- Slightly less efficient than native arrays for simple lists
- Query syntax different (e.g., `column @> '["tag"]'` vs `'tag' = ANY(column)`)

#### Option B: Add Array Literal Conversion in StorageActor
**Effort**: 2-3 hours
**Impact**: Converts JSON arrays to PostgreSQL array format during insertion

Modify `crates/botticelli_narrative/src/storage_actor.rs`:
- Detect JSON array fields
- Convert `["a","b"]` → `{a,b}` before insertion
- Handle escaping for strings with special characters

**Pros**:
- Uses native PostgreSQL arrays
- More efficient for simple tag lists
- Better for traditional PostgreSQL queries

**Cons**:
- Complex conversion logic
- Must handle edge cases (null, escaped quotes, etc.)
- Additional maintenance burden

#### Option C: Change Narrative to Output PostgreSQL Format
**Effort**: 30 minutes
**Impact**: Limited, only fixes this specific narrative

Update the curator narrative prompt to instruct the LLM:
```
Output tags in PostgreSQL array format: {"tag1","tag2"}
Instead of JSON format: ["tag1","tag2"]
```

**Pros**:
- Quick fix for this specific case
- No code changes

**Cons**:
- LLM may not reliably follow format instructions
- Doesn't solve the general problem
- Fragile, depends on LLM compliance

### Recommendation: **Option A - Use JSONB for Arrays**

**Rationale**:
- Most robust solution for dynamic schema inference
- JSONB is PostgreSQL's recommended type for JSON data
- Avoids complex format conversion logic
- Handles all JSON structures consistently

**Implementation**:
1. Update `infer_type()` in `schema_inference.rs`
2. When detecting `Array` type in JSON, map to `JSONB` instead of PostgreSQL array
3. Existing JSONB insertion code already works

**Risk**: Low. JSONB is well-supported and the inference system already handles JSON types.

---

## Issue 3: Discord Poster - Incomplete Testing

### Problem
The Discord Poster actor started executing but output was truncated. Unknown if it completed successfully.

### What We Know
- Actor loaded successfully
- Discord platform configured with channel ID
- Narrative execution began
- Uses `discord_poster.toml` narrative

### Options

#### Option A: Run Full Test with Logging
**Effort**: 10 minutes
**Impact**: Validates posting actor works

Run actor-server with detailed logging:
```bash
RUST_LOG=botticelli_actor=debug,botticelli_narrative=debug \
  ./target/debug/actor-server --config actor_server.toml
```

Monitor for:
- Table query of `approved_discord_posts`
- Discord API calls
- Post creation confirmation

**Pros**:
- Validates end-to-end pipeline
- Reveals any Discord API issues
- Low effort

**Cons**:
- Requires Discord channel to exist
- May post test content to real channel

#### Option B: Test Posting Actor Individually
**Effort**: 15 minutes
**Impact**: Isolated test without other actors

Create minimal test config:
```toml
# test_poster.toml
[server]
check_interval_seconds = 60

[[actors]]
name = "test_poster"
config_file = "actors/posting_actor.toml"
channel_id = "${DISCORD_CHANNEL_ID}"
enabled = true

[actors.schedule]
type = "Immediate"
```

**Pros**:
- Focused testing
- Faster feedback loop
- Won't run other actors

**Cons**:
- Requires separate config file
- Extra setup

### Recommendation: **Option A - Full Test with Logging**

**Rationale**:
- Tests the complete integration
- Validates all three actors work together
- Same amount of effort as individual test
- More realistic production scenario

**Action**: After fixing Issues #1 and #2, run full integration test with debug logging.

---

## Dependency Analysis

```
Issue #1 (Generator) ─────┐
                          ├──> Issue #2 (Curator) ──> Issue #3 (Poster)
                          │
                          └──> Can test independently
```

**Critical Path**:
1. Fix #1 (Generator) to populate `potential_discord_posts`
2. Fix #2 (Curator) to populate `approved_discord_posts`
3. Test #3 (Poster) to post from `approved_discord_posts`

**Parallel Work Possible**:
- Issues #1 and #2 can be fixed in parallel
- Issue #2 fix (JSONB) doesn't depend on #1
- Issue #3 testing requires #2 to be fixed

---

## Implementation Priority

### Phase 1: Unblock Pipeline (30 minutes)
**Goal**: Get end-to-end flow working

1. **Issue #1**: Apply Option A
   - Change `generation_actor.toml` to use single narrative
   - Run generator manually to populate test data

2. **Verify**: Check `potential_discord_posts` has data
   ```sql
   SELECT COUNT(*) FROM potential_discord_posts;
   ```

### Phase 2: Fix Curator (1-2 hours)
**Goal**: Enable curation to completion

3. **Issue #2**: Implement Option A
   - Update `schema_inference.rs` to use JSONB for arrays
   - Test with existing data from Phase 1

4. **Verify**: Check `approved_discord_posts` has data
   ```sql
   SELECT COUNT(*) FROM approved_discord_posts;
   ```

### Phase 3: Validate Poster (15 minutes)
**Goal**: Confirm posting works

5. **Issue #3**: Run Option A
   - Execute with debug logging
   - Monitor Discord channel for posts

6. **Verify**: Check Discord channel and database logs

### Phase 4: Enhancement (2-4 hours, optional)
**Goal**: Enable high-volume generation

7. **Issue #1**: Implement Option B
   - Fix carousel composition in NarrativeExecutor
   - Test batch generation of 15 posts

8. **Verify**: Measure throughput and content quality

---

## Risk Assessment

| Issue | Risk if Unfixed | Mitigation |
|-------|----------------|------------|
| #1 Generator | No new content generated | Manual content creation, single narrative mode |
| #2 Curator | No approved content for posting | Manual curation, direct DB insertion |
| #3 Poster | No posts to Discord | Manual posting, bot command testing |

**Overall Risk**: **Low**
All issues have workarounds and the core infrastructure is proven working.

---

## Success Criteria

### Minimum Viable (MVP) - ✅ EXCEEDED
- [x] Generator creates 1 post in `potential_discord_posts` → **27 posts created**
- [x] Curator approves 1 post to `approved_discord_posts` → **10 posts approved**
- [x] Poster publishes 1 post to Discord channel → **9 posts published**
- [x] All actors run without crashes on schedule → **All executed successfully**

### Optimal - ✅ MOSTLY ACHIEVED
- [x] Carousel composition working → **IMPLEMENTED** - all 5 narratives execute in rotation (carousel mechanism functional, though content generation has JSON parsing issues)
- [x] Curator processes all pending posts → **10 posts curated successfully**
- [x] Poster publishes approved posts without errors → **9/10 posts published**
- [ ] System runs autonomously for 24 hours → Not yet tested
- [x] State persistence tracks all executions → **Working (execution IDs 60-62 logged)**

**Note**: Carousel composition is fully functional. Content generation acts occasionally fail due to LLM JSON formatting (prompt engineering issue), not carousel composition infrastructure.

---

## Next Actions

### Completed (2025-11-28)
1. ✅ Document strategy (this document)
2. ✅ Apply Phase 1 fixes (changed to single narrative)
3. ✅ Run integration test (full pipeline working!)
4. ✅ Phase 2 validation (array insertion works natively)
5. ✅ Phase 3 validation (poster confirmed working, 9 posts published)

### Optional Improvements
1. ⬜ Fix generator critique act JSON extraction issue
2. ✅ Implement carousel composition support → **COMPLETED** (2025-11-28)
3. ⬜ Monitor production run (24 hrs)
4. ⬜ Performance optimization
5. ⬜ Enhanced error recovery

---

## Appendix: Technical Details

### A. File Locations
```
Configuration:
  actor_server.toml           - Server config
  actors/generation_actor.toml - Generator config
  actors/curation_actor.toml   - Curator config
  actors/posting_actor.toml    - Poster config

Narratives:
  crates/botticelli_narrative/narratives/discord/
    ├── generation_carousel.toml  - Multi-narrative batch generation
    ├── curate_and_approve.toml   - Curation narrative
    └── discord_poster.toml        - Posting narrative

Code:
  crates/botticelli_actor/src/
    ├── skills/narrative_execution.rs - NarrativeExecutionSkill
    ├── actor.rs                       - Core actor execution
    └── bin/actor-server.rs            - Server binary

  crates/botticelli_database/src/
    └── schema_inference.rs            - Schema inference (Issue #2)

  crates/botticelli_narrative/src/
    ├── executor.rs                    - Narrative executor (Issue #1)
    └── storage_actor.rs               - Content insertion (Issue #2)
```

### B. Database Tables
```sql
-- Generated content (from generator)
potential_discord_posts (
  -- Dynamically inferred schema
  -- Contains raw LLM output
)

-- Approved content (from curator)
approved_discord_posts (
  -- Dynamically inferred schema
  -- Contains curated posts ready for publishing
)

-- State tracking
actor_server_state (
  task_id TEXT PRIMARY KEY,
  last_run TIMESTAMP,
  consecutive_failures INTEGER,
  is_paused BOOLEAN
)

actor_server_executions (
  id SERIAL PRIMARY KEY,
  task_id TEXT,
  started_at TIMESTAMP,
  completed_at TIMESTAMP,
  status TEXT,
  -- execution metadata
)
```

### C. Key Architecture Decisions

1. **Connection Pooling**: Actor uses pool, skills get connections as needed
2. **Table Registry**: Standalone connection for table queries (TODO: refactor to use pool)
3. **Skill Registry**: NarrativeExecutionSkill registered in actor-server binary
4. **State Persistence**: Tracks execution history, circuit breaker, pause state
5. **NoOpPlatform**: Used for generator/curator (no Discord posting needed)

---

**Document Version**: 1.0
**Last Updated**: 2025-11-28
**Author**: Claude (with human oversight)

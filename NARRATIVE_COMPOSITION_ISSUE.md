# Narrative Composition Issue

## Problem Statement

We attempted to implement a Discord post generation carousel using multi-narrative TOML files with a composition pattern where a top-level "batch_generate" narrative references multiple sub-narratives (feature, usecase, tutorial, community, tip). However, the current implementation has incomplete support for this pattern, leading to validation failures and architectural ambiguity.

## Current State

### What Works

- Multi-narrative TOML files load successfully
- Individual narratives can be selected and executed by name
- Basic act definitions with text/structured inputs work
- File content loading via recursive search works

### What Doesn't Work

- Narratives referencing other narratives as acts (composition pattern)
- Shared acts defined at top-level that multiple narratives can use
- DRY principle violated: critique/refine logic duplicated across 5 narratives
- Unclear execution model for nested narratives

## Root Causes

### 1. Ambiguous Shared Acts Pattern

Current TOML attempts to define shared acts at top-level:

```toml
[acts.critique]
model = "gemini-2.5-flash-lite"
# ... config ...

[narratives.feature.toc]
order = ["generate", "critique", "refine"]  # References shared act
```

Questions:

- Are top-level `[acts.*]` shared across all narratives?
  - Yes, acts need to be shared.
- Do narratives inherit them automatically?
  - Yes.
- Can narratives override shared acts?
  - No, bad idea.
- How do we distinguish shared vs narrative-specific acts?
  - See previous answer.

### 2. Incomplete Narrative Reference Implementation

We added `narrative_ref` field to `ActConfig` but didn't implement:

- Recursive narrative execution in executor
  - Needed
- Context passing between parent/child narratives
  - Needed
- Output propagation from child to parent
  - Needed
- Variable substitution across narrative boundaries
  - Needed
- Cycle detection
  - Needed, should fail to compile with custom warning

### 3. Conflicting Design Goals

**Goal A**: DRY shared acts (critique/refine logic once)
**Goal B**: Narrative composition (batch_generate calls feature/usecase/etc)

These are different patterns that may require different solutions:

- Shared acts = code reuse within TOML
- Narrative composition = workflow orchestration

## Attempted Quick Fixes (Abandoned)

1. Added `narrative_ref` field to ActConfig - incomplete, no execution
2. Modified validation to allow empty acts - masks real problems
3. Debug logging for narrative references - doesn't solve execution

## Design Questions to Resolve

### 1. Shared Acts Pattern

**Option A: Explicit Inheritance**

```toml
[acts.critique]  # Shared definition
# ...

[narratives.feature.acts.critique]
inherits = "critique"  # Explicit reference
temperature = 0.5      # Optional override
```

-> Use this Option B
**Option B: Automatic Inheritance**

```toml
[acts.critique]  # Available to all narratives

[narratives.feature.toc]
order = ["generate", "critique"]  # Auto-finds shared act
```

**Option C: No Shared Acts (Current)**

```toml
[narratives.feature.acts.critique]  # Duplicate for each narrative
# Full definition...

[narratives.usecase.acts.critique]  # Duplicate again
# Same definition...
```

### 2. Narrative Composition Pattern

**Option A: Act-Based Reference**

```toml
[narratives.batch.acts.run_feature]
narrative = "feature"  # Execute as act
```

**Option B: ToC-Based Reference**

```toml
[narratives.batch.toc]
order = ["narrative:feature", "narrative:usecase"]  # Special prefix
```

--> I prefer Option C
**Option C: Separate Orchestration**

```toml
[orchestration.batch_generate]
narratives = ["feature", "usecase", "tutorial"]
mode = "carousel"
iterations = 10
```

### 3. Context and Variable Propagation

When narrative B is called from narrative A:

- Does B see A's outputs?
  - No, narratives are self contained
- Can B reference A's acts via `{{parent.act_name}}`?
  - Acts are shared, not per narrative
- Does B's output become an act in A for templating?
  - Table capture and table loading are used here, so no need to capture output beyond that.
- How do we handle naming conflicts?
  - Compile error with helpful warning.

### 4. Carousel Integration

Current carousel implementation:

- Runs ToC order in a loop
- Each iteration executes all acts sequentially
- No awareness of narrative composition

Questions:

- Should carousel iterate over acts or narratives?
  - Either.
- Can carousel compose narratives (current goal)?
  - Yes, needed.
- Do we need separate carousel modes?

## Decided Strategy

### Decisions Made

1. **Shared Acts**: Option B - Automatic Inheritance
   - Top-level `[acts.*]` are available to all narratives
   - Narratives automatically find shared acts in ToC
   - No overriding allowed (compile error if attempted)

2. **Narrative Composition**: Option C - Separate Orchestration
   - New `[orchestration.*]` section for running multiple narratives
   - Mode = "carousel" for iterating over narrative list
   - Keeps narratives self-contained

3. **Context/Variable Propagation**:
   - Narratives are self-contained (no parent/child context)
   - Acts are shared globally, not per-narrative
   - Output captured via table storage (existing pattern)
   - Naming conflicts = compile error with helpful warning

4. **Carousel Integration**:
   - Can iterate over acts (current) or narratives (new)
   - Orchestration carousel composes narratives
   - Narrative carousel composes acts (existing)

### Implementation Plan

### Phase 1: Implement Automatic Act Inheritance

1. Update TOML parser to recognize top-level `[acts.*]`
2. Make shared acts available to all narratives
3. Add validation: error if narrative tries to override shared act
4. Add validation: error if ToC references non-existent act

### Phase 2: Implement Orchestration Pattern

1. Add `[orchestration.*]` section to TOML schema
2. Implement orchestration executor
3. Support `mode = "carousel"` with `iterations`
4. Add cycle detection for narrative references

### Phase 3: Refactor generation_carousel.toml

1. Move critique/refine to top-level `[acts.*]`
2. Create 5 narratives with only `generate` act each
3. Add `[orchestration.batch_generate]` to run all 5
4. Test full execution with budget multipliers

## Files to Modify

### Parser Changes (`crates/botticelli_narrative/src/`)
- `toml_types.rs` - Add `orchestration` field to `NarrativeToml`
- `core.rs` - Parse top-level acts, add orchestration support
- Validation logic for shared act conflicts

### Executor Changes (`crates/botticelli_narrative/src/`)
- Orchestration executor for running multiple narratives
- Cycle detection for narrative references
- Act resolution: check narrative-local, then shared

### Configuration Changes
- `generation_carousel.toml` - Refactor to use shared acts + orchestration
- `DISCORD_POSTING_STRATEGY.md` - Update with new patterns

## Success Criteria

- [ ] Top-level `[acts.*]` parsed and shared across narratives
- [ ] Narratives can reference shared acts in ToC
- [ ] Error if narrative defines act with same name as shared act
- [ ] `[orchestration.*]` section parsed
- [ ] Orchestration carousel runs multiple narratives
- [ ] Cycle detection prevents infinite loops
- [ ] 5 narratives use shared critique/refine acts
- [ ] Orchestration runs all 5 in carousel mode
- [ ] Posts generated to potential_posts table
- [ ] Budget multipliers throttle within limits

## Implementation Notes

**Shared Act Resolution Order:**
1. Check narrative-local acts first
2. Fall back to top-level shared acts
3. Error if not found in either

**Validation Rules:**
- Compile error: narrative act name conflicts with shared act
- Compile error: ToC references non-existent act
- Compile error: orchestration references non-existent narrative
- Compile error: orchestration cycle detected

---

**Status**: Problem defined, awaiting decision on strategy
**Created**: 2025-11-24
**Author**: Claude + Erik

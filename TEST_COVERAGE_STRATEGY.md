# Test Coverage Strategy

## Executive Summary

Current coverage: **56.28%** overall
- Focus needed: Error paths, edge cases, actor lifecycle, bot orchestration
- Strengths: Core types, basic happy paths, Discord commands
- Weaknesses: Error handling, async actor systems, narrative execution paths

## Coverage by Crate (Priority Order)

### Critical Gaps

**botticelli_actor (34.97%)** - HIGHEST PRIORITY
- Missing: Actor lifecycle tests (startup, shutdown, failure recovery)
- Missing: Concurrent actor communication under load
- Missing: Error propagation across actor boundaries
- Missing: Storage actor integration tests
- Risk: Production outages from untested failure modes

**botticelli_narrative (48.04%)** - HIGH PRIORITY  
- Missing: Carousel failure scenarios (partial completion, mid-stream errors)
- Missing: Multi-narrative composition edge cases
- Missing: Table extraction with malformed JSON
- Missing: File resolution in nested directory structures
- Risk: Content generation pipeline failures

**botticelli_server (28.22%)** - HIGH PRIORITY
- Missing: Bot orchestration integration tests
- Missing: Concurrent bot execution
- Missing: Configuration loading and validation
- Missing: Graceful shutdown with active bots
- Risk: Server stability issues in production

### Moderate Gaps

**botticelli_models (58.64%)** - MEDIUM PRIORITY
- Missing: Streaming error recovery
- Missing: Rate limit boundary conditions
- Missing: Token budget exhaustion scenarios
- Strength: Good happy path coverage

**botticelli_social (73.24%)** - MEDIUM PRIORITY
- Missing: Discord API error handling
- Missing: Concurrent table operations
- Strength: Command tests are solid

**botticelli_database (66.12%)** - MEDIUM PRIORITY
- Missing: Transaction rollback scenarios
- Missing: Connection pool exhaustion
- Missing: Schema migration edge cases

### Low Priority

**botticelli_core (74.68%)** - Good coverage, mostly builder patterns
**botticelli_interface (80.65%)** - Trait definitions, less testable
**botticelli_error (85.30%)** - Strong coverage

## Strategic Recommendations

### Phase 1: Critical Reliability (Prevents Outages)

1. **Actor Lifecycle Tests**
   - Actor registration and deregistration
   - Graceful shutdown with pending work
   - Restart after crash
   - Message queue overflow handling

2. **Bot Orchestration Tests**
   - Start multiple bots concurrently
   - Stop bots gracefully
   - Bot failure isolation (one bot fails, others continue)
   - Configuration reload without restart

3. **Narrative Carousel Resilience**
   - Partial carousel completion on error
   - Resume from checkpoint after failure
   - Malformed JSON handling in extraction
   - Table write failures during generation

### Phase 2: Edge Case Coverage (Prevents Bugs)

1. **Error Path Testing**
   - Every Result-returning function needs failure test
   - Rate limit exhaustion recovery
   - Database connection loss recovery
   - Invalid configuration handling

2. **Boundary Condition Testing**
   - Empty inputs
   - Maximum size inputs
   - Concurrent access patterns
   - Resource exhaustion scenarios

### Phase 3: Integration & E2E (Prevents Regressions)

1. **Full Pipeline Tests**
   - Generation → Curation → Posting end-to-end
   - Multi-bot coordination
   - Long-running server stability

2. **Performance Tests**
   - Carousel with 100+ iterations
   - Concurrent narrative execution
   - Database query performance under load

## Tests to Remove

**Unnecessary Duplication:**
- Multiple tests of same builder pattern (keep one comprehensive test)
- Redundant Discord command tests (consolidate similar operations)

**Low Value:**
- Tests of trivial getters/setters (derive_getters already correct)
- Tests of third-party library behavior (trust dependencies)

## Tests That Are "Just Right"

**botticelli_social::discord_command_test** - Gold standard:
- Uses macro for DRY test generation
- Tests real Discord operations
- Feature-gated appropriately
- Clear success/failure expectations

**botticelli_core builder tests** - Adequate:
- Cover construction patterns
- Test validation logic
- Don't over-test derived code

## Implementation Priority

1. **Week 1**: Actor lifecycle + bot orchestration (prevents production outages)
2. **Week 2**: Carousel resilience + error paths (prevents data loss)
3. **Week 3**: Edge cases + boundary conditions (prevents bugs)
4. **Week 4**: Integration tests + performance validation (prevents regressions)

## Metrics to Track

- **Coverage %**: Target 75% overall (not 100% - diminishing returns)
- **Error path coverage**: Every Result should have failure test
- **Integration test count**: At least 5 full pipeline tests
- **Performance regression detection**: Track carousel timing

## Philosophy

**Test what breaks, not what works.**
- Focus on failure modes, not happy paths
- Integration over unit tests for async systems
- Real scenarios over synthetic edge cases
- Maintenance cost matters - avoid brittle tests

**Current state:** Good foundations, weak on resilience
**Goal state:** Confident deployments, fast failure detection

## Implementation Progress

### Phase 1: Critical Gaps (Completed)
- ✅ Storage actor error handling tests (malformed JSON, missing tables, empty data)
- ✅ Narrative error recovery tests (invalid acts, empty TOC, circular references)
- ✅ Schema inference edge case tests (nested objects, arrays, nulls, mixed types)

### Phase 2: Actor System (Next)
- ⏳ Actor lifecycle tests
- ⏳ Concurrent communication tests
- ⏳ Error propagation tests

### Phase 3: Bot Server Integration (Upcoming)
- ⬜ Bot orchestration integration tests
- ⬜ Graceful shutdown tests
- ⬜ Configuration validation tests

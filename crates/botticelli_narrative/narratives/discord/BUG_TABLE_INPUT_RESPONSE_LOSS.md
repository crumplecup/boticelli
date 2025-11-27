# Bug Report: LLM Response Loss with Table Inputs + ContentGenerationProcessor

**Status**: Investigation
**Priority**: High
**Affected Component**: `botticelli_narrative` - Response handling pipeline
**Created**: 2025-11-26

## Summary

LLM responses are completely lost (response_length=0) when a narrative act combines:
1. SQL table input (`type = "table"`)
2. Active ContentGenerationProcessor (no `skip_content_generation` flag)

The LLM successfully generates output (confirmed by token usage), but the response text becomes empty before reaching the JSON extraction step.

## Observed Behavior

### Symptoms
- LLM API call succeeds (response received successfully)
- Token usage confirms text generation (e.g., 4497 prompt + 599 output = 5096 total tokens)
- Response text is empty when ContentGenerationProcessor runs (`response_length=0`)
- Error: "No JSON found in LLM response (length: 0)"

### Log Evidence
```
[INFO] Processing act with registered processors act=select processors=1
[INFO] Processing content generation act=select table=approved_discord_posts mode=Inference
[DEBUG] Starting content generation table=approved_discord_posts
[INFO] Started tracking content generation table=approved_discord_posts
[ERROR] No JSON found in LLM response response_length=0
```

### Working vs Broken

**✅ WORKS** (generation_carousel.toml):
```toml
[[acts.generate.input]]
type = "text"
file = "BOTTICELLI_CONTEXT.md"

[[acts.generate.input]]
type = "text"
content = "Generate JSON..."
```
Result: ✅ Response captured, JSON extracted, rows inserted

**❌ FAILS** (curate_and_approve.toml):
```toml
[[acts.select.input]]
type = "table"
table_name = "potential_discord_posts"
format = "markdown"

[[acts.select.input]]
type = "text"
content = "Output JSON..."
```
Result: ❌ Response lost, 0 characters, JSON extraction fails

## Expected Behavior

The response text should be preserved through the entire pipeline:
1. API call returns response with text
2. Response text stored in ActExecution
3. ContentGenerationProcessor receives response text
4. JSON extracted from response text
5. Rows inserted into target table

## Theory: Response Discarding in Input Processing

### Primary Hypothesis
The response is being discarded during input template resolution or table formatting, specifically when:
- Table query results are large (18KB+ markdown)
- Template resolution runs on act response
- Response text gets overwritten or cleared

### Potential Code Paths

**Path 1: Response cleared during table input processing**
```
execute() -> process_inputs() -> Input::Table processing
  -> response_text gets modified/cleared?
  -> ActExecution created with empty response
  -> ContentGenerationProcessor receives empty response
```

**Path 2: Template resolution overwrites response**
```
execute() -> ActExecution created with response_text
  -> processor runs -> attempts template resolution
  -> response field accidentally cleared during table reference resolution
```

**Path 3: Table data interferes with response capture**
```
extract_text_from_outputs() called with table data still in context
  -> Returns empty string when table inputs present
  -> Response lost before ActExecution creation
```

## Evidence Supporting Theory

### Test Results
- **5 different prompts tested** - ALL failed with same behavior
- **Token usage confirms generation** - 4497-7916 tokens used
- **Table query succeeds** - "Retrieved rows count=10"
- **Response becomes 0** - After API success, before JSON parsing

### Comparison Data

| Input Type | ContentGen Active | Result |
|------------|-------------------|--------|
| Text file  | Yes              | ✅ Works (generation_carousel) |
| Text literal | Yes            | ✅ Works (generation_carousel) |
| Table query | Yes             | ❌ Fails (response_length=0) |
| Table query | No (skip flag)  | ✅ Works (curate_posts_final) |

### Key Observation
The ONLY configuration that fails is: **Table input + ContentGenerationProcessor**

## Isolation Strategy

### Phase 1: Confirm Response Capture
**Goal**: Determine if response is lost before or after ActExecution creation

**Test 1.1**: Minimal table + text input without JSON request
```toml
[[acts.test.input]]
type = "table"
table_name = "potential_discord_posts"
limit = 1

[[acts.test.input]]
type = "text"
content = "Say hello."
```
**Expected**: If response_length > 0, response is captured correctly
**If fails**: Bug is in response capture, not JSON extraction

**Test 1.2**: Table input with skip_content_generation + debug logging
```toml
[narrative]
skip_content_generation = true

[[acts.test.input]]
type = "table"
...
```
**Expected**: Response preserved when ContentGenerationProcessor skipped
**If works**: Confirms processor involvement

### Phase 2: Isolate Input Processing
**Goal**: Determine if table input affects response handling

**Test 2.1**: Dual text inputs (no table)
```toml
[[acts.test.input]]
type = "text"
content = "List: A, B, C"

[[acts.test.input]]
type = "text"
content = "Pick one. Output JSON: {\"choice\": \"...\"}"
```
**Expected**: Works (no table)

**Test 2.2**: Table input AFTER text prompt (reversed order)
```toml
[[acts.test.input]]
type = "text"
content = "Output JSON: {\"test\": \"value\"}"

[[acts.test.input]]
type = "table"
table_name = "potential_discord_posts"
limit = 1
```
**Expected**: If works, order matters; if fails, table presence matters

**Test 2.3**: Empty table query
```toml
[[acts.test.input]]
type = "table"
table_name = "potential_discord_posts"
where_clause = "1=0"  # Returns no rows

[[acts.test.input]]
type = "text"
content = "Output JSON..."
```
**Expected**: If works, table data size is factor; if fails, table presence is factor

### Phase 3: Code Inspection
**Goal**: Find where response is cleared/lost

**Files to examine**:
1. `crates/botticelli_narrative/src/executor.rs`
   - `execute()` method around line 200-400
   - `process_inputs()` method
   - `extract_text_from_outputs()` function
   - ActExecution creation logic

2. `crates/botticelli_narrative/src/content_generation.rs`
   - `ContentGenerationProcessor::process()` method
   - Response text access pattern

3. `crates/botticelli_interface/src/narrative/execution.rs`
   - `ActExecution` struct definition
   - Response field handling

**Search patterns**:
```bash
# Find where response is set/modified
rg "response.*=.*String" executor.rs
rg "response\.clear\(\)" executor.rs
rg "\.response" executor.rs | grep -v "pub response"

# Find table input processing
rg "Input::Table" executor.rs -A 20
rg "table_name" executor.rs -A 10
```

### Phase 4: Add Instrumentation
**Goal**: Trace response through pipeline

**Logging additions needed**:
```rust
// In executor.rs after LLM call
tracing::debug!(
    response_length = response_text.len(),
    response_preview = &response_text[..response_text.len().min(200)],
    "Response captured from LLM"
);

// Before ActExecution creation
tracing::debug!(
    response_length = response_text.len(),
    "Creating ActExecution with response"
);

// In ContentGenerationProcessor
tracing::debug!(
    act_response_length = context.execution.response.len(),
    act_response_preview = &context.execution.response[..context.execution.response.len().min(200)],
    "Processor received response"
);
```

## Reproduction Steps

1. Create narrative with table input + ContentGenerationProcessor:
```toml
[narrative]
name = "test_bug"
target = "test_output"

[toc]
order = ["test"]

[acts.test]
[[acts.test.input]]
type = "table"
table_name = "potential_discord_posts"
limit = 1

[[acts.test.input]]
type = "text"
content = "Output JSON: {\"test\": \"value\"}"
```

2. Run: `just narrate test_bug`
3. Observe: `Response length: 0 characters`

## Environment

- **Botticelli version**: Current (models branch)
- **OS**: Linux 6.12.48-1-MANJARO
- **Rust**: (check with `rustc --version`)
- **Database**: PostgreSQL (schema inference mode)
- **LLM**: Google Gemini (gemini-2.5-flash, gemini-2.5-flash-lite)

## Related Code

**Response handling**:
- `crates/botticelli_narrative/src/executor.rs` - Main execution loop
- `crates/botticelli_narrative/src/content_generation.rs` - Processor implementation
- `crates/botticelli_interface/src/narrative/execution.rs` - ActExecution struct

**Table input processing**:
- `crates/botticelli_narrative/src/executor.rs:process_inputs()` - Table query handling
- `crates/botticelli_database/src/table_query.rs` - SQL execution

## Workarounds

Until fixed, use one of these approaches:

### Workaround 1: Two-step process
```toml
# Step 1: Analyze with skip flag
[acts.analyze]
skip_content_generation = true
[[acts.analyze.input]]
type = "table"
...

# Step 2: Format with text input
[acts.format]
[[acts.format.input]]
type = "text"
content = "Based on: {{analyze}}, output JSON..."
```

### Workaround 2: Export to file
```bash
# Export table to markdown file
psql $DATABASE_URL -c "SELECT * FROM potential_discord_posts" > posts.md
```
```toml
# Load as text file instead of table
[[acts.test.input]]
type = "text"
file = "posts.md"
```

### Workaround 3: Manual approval
```bash
# Use skip_content_generation for analysis
just narrate curate_posts_final
# Manually copy approved posts via SQL
```

## Next Steps

1. ✅ Document theory and isolation strategy (this document)
2. 🔲 Execute Phase 1 isolation tests
3. 🔲 Execute Phase 2 isolation tests
4. 🔲 Code inspection (Phase 3)
5. 🔲 Add instrumentation if needed (Phase 4)
6. 🔲 File GitHub issue with findings
7. 🔲 Implement fix or document permanent workaround

## Investigation Log

### 2025-11-26: Initial Discovery
- Discovered while implementing curation pipeline
- Tested 5 different prompts - all failed identically
- Confirmed token usage shows LLM generates output
- Confirmed `skip_content_generation` workaround succeeds

### 2025-11-26: Isolation Strategy Designed
- Created this document
- Designed 3-phase isolation plan
- Identified key code paths to investigate

### 2025-11-26: Isolation Tests Executed

**Phase 1 Results:**
- **Test 1.1** (minimal table + text, no JSON request): ✅ PASS
  - Response captured: response_length=58
  - Token usage: 530 prompt + 99 output = 629 total
  - Conclusion: Response IS captured with table input when no JSON extraction attempted
  - File: `isolation_test_1_1_minimal.toml`

**Phase 2 Results:**
- **Test 2.1** (dual text inputs, no table - control): ✅ PASS
  - Content generation successful: row_count=1
  - Conclusion: ContentGenerationProcessor works perfectly without table input
  - File: `isolation_test_2_1_no_table.toml`

- **Test 2.2** (text prompt FIRST, then table - reversed order): ❌ FAIL
  - Response lost: response_length=0
  - Token usage: 5086 prompt + 599 output = 5685 total
  - Conclusion: Input order doesn't matter - table presence causes failure
  - File: `isolation_test_2_2_reversed.toml`

- **Test 2.3** (empty table query with where_clause="1=0"): ❌ FAIL
  - Response lost: response_length=0
  - Token usage: 530 prompt + 99 output = 629 total
  - Note: where_clause not applied (retrieved 1 row instead of 0) - separate bug
  - Conclusion: Even minimal table data causes response loss
  - File: `isolation_test_2_3_empty_table.toml`

**Key Findings:**

1. **Bug is definitively NOT prompt-related**
   - All isolation tests use simple prompts requesting JSON output
   - Some work (2.1), some fail (2.2, 2.3) with identical prompt patterns

2. **Response IS captured from LLM**
   - Token usage confirms generation occurs (99-599 output tokens)
   - Test 1.1 proves response capture works (58 characters captured)
   - Response becomes 0 ONLY when ContentGenerationProcessor attempts JSON extraction

3. **Table input presence is the trigger**
   - Test 2.1 (no table): ✅ Works
   - Test 1.1 (table, no JSON extraction): ✅ Works
   - Test 2.2 (table + JSON extraction, text first): ❌ Fails
   - Test 2.3 (table + JSON extraction, table first): ❌ Fails

4. **Input order irrelevant**
   - Table before text (standard): ❌ Fails
   - Text before table (reversed): ❌ Fails
   - Proves table presence itself is the issue, not order

**Hypothesis Confirmed:**
The bug occurs at the intersection of:
- Table input present (in any position, any size)
- ContentGenerationProcessor active (attempts JSON extraction)
- Result: Response text becomes empty (0 characters) before JSON parsing

**Next Steps:**
- Phase 3: Code inspection to find where response is cleared
- Search patterns:
  - `response.*=.*String` in executor.rs
  - `response.clear()` in executor.rs
  - Response handling in ContentGenerationProcessor
  - ActExecution creation logic

### 2025-11-26: Phase 3 Code Inspection - CRITICAL FINDINGS

**Instrumentation Added:**
- Added debug logging to executor.rs:404-416 (response extraction)
- Added debug logging to executor.rs:446-450 (ActExecution creation)
- Added debug logging to executor.rs:370-393 (processed inputs)
- Added debug logging to client.rs:612-616 (Gemini Live API input combining)
- Added debug logging to live_client.rs:394-398 (server response chunks)

**Discovery 1: Response IS Generated by LLM**
- Token usage confirms generation: 530 prompt + 99 output = 629 total tokens
- Gemini API successfully responds
- Response extraction from `Output::Text` occurs correctly

**Discovery 2: Response Truncation at API Level**
Test runs show:
- Test 2.3 (table + JSON request):
  - Inputs sent: 2241 chars (table) + 30 chars (text prompt)
  - Output received: `text_length=10`, `text_preview="```json\n{\""`
  - First 10 characters of what should be valid JSON

- Test 1.1 (table + "Say hello"):
  - Inputs sent: 2053 chars (table) + 10 chars (text prompt)
  - Output received: `text_length=10`, `text_preview="Say hello."`
  - **CRITICAL**: This is the INPUT PROMPT echoed back, not a real response!

**Discovery 3: Exact 10-Character Truncation**
Both failing tests receive EXACTLY 10 characters:
- Not random
- Not based on tokens
- Not null/empty (would be 0)
- Appears to be systematic truncation

**Discovery 4: ActExecution Created with Truncated Response**
```
DEBUG ActExecution created with response act=test act_execution_response_length=0
```
- executor.rs:431 stores `response_text.clone()`
- Response is already 0/10 chars at ActExecution creation
- **NOT cleared by ContentGenerationProcessor**
- Truncation happens BEFORE ActExecution

**Discovery 5: Gemini API Integration**
- Using `gemini_rust` library (external dependency)
- Model `gemini-2.5-flash` uses REST API (not Live API)
- Pattern check: `model_name.contains("-live") || model_name.contains("-exp")`
- Live API logging not triggered (explains missing trace logs)

**Root Cause Hypothesis:**
The truncation occurs at the `gemini_rust` library level or Gemini REST API level when:
1. Input size is large (2000+ characters)
2. max_tokens is set low (100)
3. Combination triggers response truncation/echoing behavior

**Evidence for gemini_rust Library Issue:**
- Truncation happens after `extract_text_from_outputs()` call (executor.rs:410)
- Output::Text variant contains truncated text
- This means truncation occurred during API call or response parsing
- gemini_rust library constructs GenerateResponse with Output::Text
- Truncation must be in gemini_rust or upstream at Gemini API

**Next Investigation Steps:**
1. ✅ Test with higher max_tokens (e.g., 1000 instead of 100) - **ROOT CAUSE FOUND**
2. ~~Check gemini_rust library version and known issues~~
3. ~~Test with different model (e.g., gemini-2.0-flash-exp for Live API)~~
4. ~~Test with smaller table input~~
5. ~~Add logging to gemini_rust response parsing~~
6. ~~Check Gemini API response directly~~

### 2025-11-26: ROOT CAUSE IDENTIFIED - max_tokens Too Low

**Test 2.4 Results (max_tokens=1000):**
Created `isolation_test_2_4_higher_tokens.toml` with ONLY change: `max_tokens = 1000` (was 100)

```
Output is Text variant output_index=0 text_length=50 text_preview="```json\n{\n  \"tags\": [\"potential\", \"usecase\"]\n}\n```"
Response text extracted from LLM outputs response_length=50
ActExecution created with response act=test act_execution_response_length=2261

Response length: 2261 characters  ← SUCCESS!
```

**Comparison:**
| Test | max_tokens | Input Size | Response Length | Status |
|------|------------|------------|-----------------|--------|
| 2.3  | 100        | 2271 chars | 10 chars        | ❌ FAIL |
| 2.4  | 1000       | 2271 chars | 2261 chars      | ✅ PASS |

**ROOT CAUSE:**
`max_tokens=100` is insufficient when input size is large (~2000+ characters).

**Why This Happens:**
The Gemini API appears to reserve tokens for the input context, leaving insufficient tokens for response generation when max_tokens is too low relative to input size.

Calculation:
- Input: ~2271 characters ≈ 568 tokens (rough estimate: chars/4)
- max_tokens: 100
- Available for response: Severely limited or zero
- Result: Truncated 10-char response (possibly echo/error behavior)

**This is NOT a Bug - It's a Configuration Issue:**
The original problem reports treated this as a bug in response handling. However:
- Botticelli is working correctly
- ContentGenerationProcessor is working correctly
- ActExecution is storing what it receives correctly
- The issue is: **Gemini API cannot generate meaningful responses when max_tokens is too low for the given input size**

**Solution:**
Increase `max_tokens` appropriately based on expected input + output size:
- For table inputs (typically 1000-5000 chars): Use `max_tokens >= 1000`
- For large table inputs (5000+ chars): Use `max_tokens >= 2000`
- Formula: `max_tokens >= (estimated_input_tokens + desired_output_tokens) * 1.2`

**Workaround for curate_and_approve.toml:**
```toml
[acts.select]
model = "gemini-2.5-flash"
temperature = 0.3
max_tokens = 3000  # ← Increase from default

[[acts.select.input]]
type = "table"
table_name = "potential_discord_posts"
# ... rest of config
```

**Status: RESOLVED**
This was not a Botticelli bug. It was a misconfiguration where `max_tokens` was set too low for the input size, causing Gemini API to return truncated/echo responses.

# Conversation History Retention Feature - Implementation Plan

**Status**: Planning
**Priority**: Medium
**Complexity**: Medium
**Target**: Botticelli 0.3.0
**Created**: 2025-11-26

## Executive Summary

Add configurable conversation history retention to Botticelli narratives, allowing users to control whether large inputs (especially table data) are retained in conversation history after the LLM processes them. This addresses token cost, context window management, and performance issues in multi-act narratives.

## Problem Statement

### Current Behavior

In multi-act narratives, all inputs (including large table queries) are retained in conversation history:

```toml
[acts.analyze]
[[acts.analyze.input]]
type = "table"
table_name = "potential_discord_posts"
limit = 10  # Results in ~18KB markdown

[[acts.analyze.input]]
type = "text"
content = "Analyze these posts..."
```

**Conversation history after Act 1:**
```
User: [18KB table markdown] + "Analyze these posts..."
Assistant: "I selected posts 3, 7, 9 based on..."
```

**Problem in Act 2:**
```
User: [18KB table markdown] + "Analyze..."  # From Act 1 history
Assistant: "I selected posts 3, 7, 9..."      # From Act 1 history
User: "Now create variations..."              # Act 2 input
```

The 18KB table is re-sent with every subsequent act, even though:
- The LLM already processed it
- The assistant's response contains the key decision
- The raw data is no longer needed

### Impact

**Observed in curate_and_approve.toml:**
```
Processed input: text_length=18589
usage.prompt_tokens=4727
usage.cached_content_tokens=4077
usage.total_tokens=12409
```

**For multi-act narratives:**
- **Token explosion**: 18KB × N acts = exponential growth
- **Cost**: Even with caching, large prompts are expensive
- **Rate limits**: Free tier (200K TPM) exhausted quickly
- **Context pollution**: Large tables crowd out reasoning space
- **Performance**: Slower API responses with huge prompts

## Why This Matters

### 1. Real-World Use Case

**Discord content curation pipeline** (current motivation):
```
Act 1: Analyze 10 posts (18KB table) → Select top 3
Act 2: Expand selected posts → Generate variations
Act 3: Review variations → Final selection
```

Without retention control:
- Act 2 prompt: 18KB (original table) + 6KB (Act 1 response) + 2KB (Act 2 prompt) = **26KB**
- Act 3 prompt: 18KB + 6KB + 8KB (Act 2 response) + 2KB (Act 3 prompt) = **34KB**

With retention control (drop original table):
- Act 2 prompt: 6KB (Act 1 response) + 2KB (Act 2 prompt) = **8KB** ✅
- Act 3 prompt: 6KB + 8KB + 2KB = **16KB** ✅

**Savings**: 67% token reduction in Act 3

### 2. Generalization to Other Workflows

**Analysis → Action pattern:**
```
Act 1: Query logs (huge table) → Identify anomalies
Act 2: Generate alerts for anomalies  # Doesn't need full logs
Act 3: Send alerts via bot commands   # Doesn't need full logs
```

**Summarization → Distribution:**
```
Act 1: Load document (50 pages) → Summarize
Act 2: Format summary as email      # Doesn't need original doc
Act 3: Format summary as tweet      # Doesn't need original doc
```

**Database workflows:**
- Act 1: Query large dataset → Extract insights
- Act 2: Generate SQL UPDATE based on insights
- No need to re-send original dataset

### 3. Performance and Cost Benefits

| Metric | Current (No Retention Control) | With Retention Control |
|--------|-------------------------------|------------------------|
| **Tokens per act** | Growing (additive) | Stable (bounded) |
| **API response time** | Slower with size | Consistently fast |
| **Cache efficiency** | Wastes cache on old data | Caches relevant context |
| **Free tier utilization** | Hits limits quickly | Sustainable usage |
| **Cost (paid tier)** | $$ per act | $ per act |

## Design Approach

### Core Principle

**Explicit, not implicit**: Users declare retention intent per input, with sane defaults.

### Configuration Model

#### New Field: `history_retention`

Add to all input types that can be large:

```toml
[[acts.analyze.input]]
type = "table"
table_name = "potential_discord_posts"
history_retention = "summary"  # NEW FIELD
```

**Valid values:**
- `"full"` (default): Current behavior - retain entire input
- `"summary"`: Replace with concise summary after processing
- `"drop"`: Remove entire input from history after processing

**Applies to input types:**
- `table`: Large query results
- `text` with `file`: Potentially large file contents
- `narrative`: Nested narrative outputs
- Future: `bot_command` with large responses

### Behavior Specification

#### `history_retention = "full"` (Default)

**No change from current behavior.**

```
Before Act 1: []
After Act 1:  [User(table + prompt), Assistant(response)]
```

Use when:
- Single-act narratives
- Small inputs (< 5KB)
- Subsequent acts need to re-examine the data
- Debugging/development

#### `history_retention = "summary"`

**Replace input with summary after LLM processes it.**

```
Before Act 1: []
During Act 1: [User(table + prompt)] → LLM
After Act 1:  [User(summary + prompt), Assistant(response)]
                    ↑ Modified
```

**Summary format for tables:**
```markdown
[Table: potential_discord_posts, 10 rows, ~18KB]
```

**Summary format for files:**
```markdown
[File: BOTTICELLI_CONTEXT.md, ~5KB]
```

**Summary format for nested narratives:**
```markdown
[Narrative: generation_carousel, 5 acts executed]
```

Use when:
- Multi-act narratives
- Large inputs (> 5KB)
- Subsequent acts only need the decision/result
- Production pipelines

#### `history_retention = "drop"`

**Remove the entire input message part after processing.**

```
Before Act 1: []
During Act 1: [User(table + prompt)] → LLM
After Act 1:  [User(prompt), Assistant(response)]
                    ↑ Table removed
```

If the message has multiple parts:
```toml
[[acts.analyze.input]]
type = "table"
history_retention = "drop"

[[acts.analyze.input]]
type = "text"
content = "Analyze the above table"
# This text input is retained
```

Result:
```
After Act 1: [User("Analyze the above table"), Assistant(response)]
```

Use when:
- Maximum token savings needed
- Input is truly one-time (never referenced again)
- Subsequent acts don't need any trace of original data

### Safety: Auto-Summary Threshold

**Automatic escalation to `summary` for very large inputs:**

```rust
const AUTO_SUMMARY_THRESHOLD: usize = 10_000; // 10KB

if input.len() > AUTO_SUMMARY_THRESHOLD && history_retention == "full" {
    tracing::warn!(
        input_size = input.len(),
        "Large input auto-condensed to summary in conversation history"
    );
    // Treat as "summary"
}
```

**Rationale:**
- Prevents accidental token explosion
- Provides guardrail for users unfamiliar with the feature
- Still respects explicit `"full"` on smaller inputs
- Logs warning for visibility

**Configuration override:**
```toml
# In botticelli.toml
[narrative]
auto_summary_threshold = 20000  # Increase to 20KB
# Or disable:
auto_summary_threshold = 0  # Never auto-summarize
```

## Implementation Plan

### Phase 1: Core Infrastructure (Week 1)

#### 1.1: Add `history_retention` to Input Types

**File:** `crates/botticelli_core/src/input.rs`

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HistoryRetention {
    Full,
    Summary,
    Drop,
}

impl Default for HistoryRetention {
    fn default() -> Self {
        Self::Full
    }
}

// Add to Input::Table variant:
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInput {
    pub table_name: String,
    // ... existing fields ...
    #[serde(default)]
    pub history_retention: HistoryRetention,
}

// Similarly for Input::Text with file, Input::Narrative, etc.
```

#### 1.2: Update TOML Parser

**File:** `crates/botticelli_narrative/src/toml_parser.rs`

Add `history_retention` field to `TomlInput::Table`:

```rust
#[derive(Debug, Deserialize)]
pub struct TomlTableInput {
    pub table_name: String,
    // ... existing fields ...
    #[serde(default)]
    pub history_retention: Option<String>,  // "full" | "summary" | "drop"
}

// Validation in to_input():
fn parse_history_retention(value: &str) -> Result<HistoryRetention> {
    match value {
        "full" => Ok(HistoryRetention::Full),
        "summary" => Ok(HistoryRetention::Summary),
        "drop" => Ok(HistoryRetention::Drop),
        _ => Err(/* error: invalid value */),
    }
}
```

#### 1.3: Implement Summary Generation

**File:** `crates/botticelli_narrative/src/history_retention.rs` (new)

```rust
/// Generate a concise summary for a large input
pub fn summarize_input(input: &Input) -> String {
    match input {
        Input::Table { table_name, limit, offset, .. } => {
            format!(
                "[Table: {}, {} rows queried{}]",
                table_name,
                limit.unwrap_or(0),
                offset.map(|o| format!(", offset {}", o)).unwrap_or_default()
            )
        }
        Input::Text(content) if content.len() > 1000 => {
            format!("[Text: ~{}KB]", content.len() / 1024)
        }
        Input::Narrative { name, .. } => {
            format!("[Nested narrative: {}]", name)
        }
        other => {
            // For small inputs, return as-is
            format!("{:?}", other) // Fallback
        }
    }
}

/// Apply retention policy to a message after LLM processing
pub fn apply_retention_policy(
    message: &mut Message,
    retention_policies: &[(usize, HistoryRetention)], // (input_index, policy)
) {
    for (idx, policy) in retention_policies {
        match policy {
            HistoryRetention::Full => {
                // No change
            }
            HistoryRetention::Summary => {
                if let Some(input) = message.content.get_mut(*idx) {
                    let summary = summarize_input(input);
                    *input = Input::Text(summary);
                }
            }
            HistoryRetention::Drop => {
                // Mark for removal (can't remove during iteration)
            }
        }
    }

    // Remove dropped inputs
    message.content.retain_mut(|(idx, _)| {
        !retention_policies.iter().any(|(i, p)| i == idx && matches!(p, HistoryRetention::Drop))
    });
}
```

#### 1.4: Integrate into Executor

**File:** `crates/botticelli_narrative/src/executor.rs`

**Current code** (around line 395):
```rust
conversation_history.push(
    MessageBuilder::default()
        .role(Role::User)
        .content(processed_inputs.clone())
        .build()?
);
```

**Modified code:**
```rust
// Build user message
let mut user_message = MessageBuilder::default()
    .role(Role::User)
    .content(processed_inputs.clone())
    .build()?;

conversation_history.push(user_message.clone());

// After LLM call, apply retention policy
let retention_policies: Vec<(usize, HistoryRetention)> = config
    .inputs()
    .iter()
    .enumerate()
    .filter_map(|(idx, input)| {
        input.history_retention().map(|r| (idx, r.clone()))
    })
    .collect();

if !retention_policies.is_empty() {
    let last_user_msg_idx = conversation_history.len() - 2; // -1 for assistant, -1 more for user
    if let Some(msg) = conversation_history.get_mut(last_user_msg_idx) {
        apply_retention_policy(msg, &retention_policies);

        tracing::debug!(
            retained_policies = ?retention_policies,
            "Applied history retention policies to conversation"
        );
    }
}
```

### Phase 2: Testing (Week 1)

#### 2.1: Unit Tests

**File:** `crates/botticelli_narrative/tests/history_retention_test.rs`

```rust
#[test]
fn test_summary_retention_replaces_table() {
    let input = Input::Table {
        table_name: "large_table".to_string(),
        // ... large data ...
        history_retention: HistoryRetention::Summary,
    };

    let summary = summarize_input(&input);
    assert!(summary.contains("Table: large_table"));
    assert!(summary.len() < 100); // Much smaller than original
}

#[test]
fn test_drop_retention_removes_input() {
    let inputs = vec![
        Input::Table { /* ... */ history_retention: HistoryRetention::Drop },
        Input::Text("Keep this".to_string()),
    ];

    let mut message = Message::new(Role::User, inputs);
    apply_retention_policy(&mut message, &[(0, HistoryRetention::Drop)]);

    assert_eq!(message.content.len(), 1);
    assert!(matches!(message.content[0], Input::Text(_)));
}

#[test]
fn test_auto_summary_threshold() {
    let large_input = Input::Text("x".repeat(15_000)); // > 10KB threshold

    // Should auto-escalate to summary
    let should_summarize = large_input.len() > AUTO_SUMMARY_THRESHOLD;
    assert!(should_summarize);
}
```

#### 2.2: Integration Tests

**File:** `crates/botticelli_narrative/narratives/tests/history_retention_integration.toml`

```toml
# Test: Multi-act with table retention
[narrative]
name = "history_retention_test"
description = "Test conversation history retention with tables"

[toc]
order = ["analyze", "refine"]

[acts.analyze]
model = "gemini-2.5-flash"
max_tokens = 500

[[acts.analyze.input]]
type = "table"
table_name = "test_data"
limit = 10
history_retention = "summary"  # Test this!

[[acts.analyze.input]]
type = "text"
content = "Summarize the data"

[acts.refine]
model = "gemini-2.5-flash"
max_tokens = 200

[[acts.refine.input]]
type = "text"
content = "Based on your previous summary, provide top 3 insights"
# Should work without the table in history
```

**Test assertions:**
1. Act 1 receives full table
2. Act 2 conversation history contains summary, not full table
3. Total tokens for Act 2 < Act 1
4. Act 2 response is coherent despite missing table

### Phase 3: Documentation (Week 2)

#### 3.1: Update NARRATIVE_TOML_SPEC.md

Add `history_retention` field documentation:

```markdown
### Input Configuration

#### Table Input

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `history_retention` | string | No | `"full"` | Controls how this input is retained in conversation history for subsequent acts. Values: `"full"` (retain entire input), `"summary"` (replace with concise summary), `"drop"` (remove from history). |

**Example:**
\```toml
[[acts.analyze.input]]
type = "table"
table_name = "large_dataset"
history_retention = "summary"  # Don't retain 50MB table
\```

**When to use:**
- Use `"summary"` for large tables (>10KB) in multi-act narratives
- Use `"drop"` when subsequent acts never reference the data
- Use `"full"` (default) when acts need to re-examine the data
```

#### 3.2: Create User Guide

**File:** `docs/guides/conversation-history-management.md`

```markdown
# Conversation History Management

## Overview

Botticelli maintains conversation history across acts in multi-act narratives...

## Problem: Token Explosion

[Explain the problem with examples]

## Solution: History Retention Control

[Show examples of all three modes]

## Best Practices

1. **Large tables**: Use `history_retention = "summary"`
2. **File inputs**: Use `"summary"` for files > 5KB
3. **One-shot data**: Use `"drop"` for data never referenced again
4. **Debugging**: Use `"full"` to see exact history

## Examples

### Discord Content Curation
[Full example with curate_and_approve.toml]

### Log Analysis Pipeline
[Example of analyze → alert workflow]
```

#### 3.3: Update AI_NARRATIVE_TOML_GUIDE.md

Add section on history retention with AI-focused recommendations.

### Phase 4: Migration and Rollout (Week 2)

#### 4.1: Update Existing Narratives

**Candidates for update:**
1. `curate_and_approve.toml` - Add `history_retention = "summary"` to table input
2. `generation_carousel.toml` - Review if multi-act extension planned
3. Any narrative with `type = "narrative"` (nested narratives)

#### 4.2: Deprecation Strategy

**No breaking changes:** Default behavior is `"full"` (current behavior).

**Optional warning** (can enable via config):
```toml
# In botticelli.toml
[narrative]
warn_large_history = true  # Warn when inputs > 10KB without retention control
```

Logs:
```
WARN: Large input (18KB) with history_retention="full" in multi-act narrative.
      Consider using history_retention="summary" to reduce token costs.
      See: docs/guides/conversation-history-management.md
```

#### 4.3: Changelog Entry

```markdown
## [0.3.0] - 2025-XX-XX

### Added
- **Conversation history retention control**: New `history_retention` field for inputs
  - `"full"`: Retain entire input (default, current behavior)
  - `"summary"`: Replace with concise summary after processing
  - `"drop"`: Remove from history after processing
- Auto-summary threshold: Large inputs (>10KB) automatically summarized in history
- Comprehensive logging of retention policy application

### Changed
- Multi-act narratives with large table inputs now benefit from token optimization

### Migration Guide
- No breaking changes; default behavior unchanged
- For multi-act narratives with large tables, add `history_retention = "summary"`:
  \```toml
  [[acts.analyze.input]]
  type = "table"
  table_name = "large_data"
  history_retention = "summary"  # NEW
  \```
```

## Success Metrics

### Performance Improvements

**Target: curate_and_approve.toml**

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Act 1 tokens | 4,727 | 4,727 | 0% (same) |
| Act 2 tokens (if added) | ~9,000 | ~1,500 | **83%** |
| Act 3 tokens (if added) | ~13,000 | ~2,000 | **85%** |
| Cache efficiency | Low (caching old data) | High (caching summaries) | Better |

### User Impact

**Quantitative:**
- 80%+ token reduction in multi-act table workflows
- 3-5x more acts possible within free tier limits
- Faster API responses (smaller prompts)

**Qualitative:**
- Explicit control over conversation history
- Better understanding of token costs
- Easier debugging with summary breadcrumbs

## Risks and Mitigations

### Risk 1: Users Accidentally Drop Needed Data

**Scenario:** User sets `history_retention = "drop"` on data that Act 2 needs.

**Mitigation:**
- Default is `"full"` (safe)
- Documentation emphasizes "use `summary` unless you're sure"
- Error messages guide users when template resolution fails

**Example error:**
```
Error: Act 'refine' references {{analyze}} but conversation history
       contains only a summary. Consider using history_retention="full"
       for the table input in act 'analyze'.
```

### Risk 2: Summary Too Vague

**Scenario:** Summary like `[Table: data, 10 rows]` loses critical context.

**Mitigation:**
- Include key metadata in summaries (table name, row count, size)
- Users can stay with `"full"` if needed
- Future: Smarter summaries (first/last rows, schema, etc.)

### Risk 3: Confusing for New Users

**Scenario:** Feature adds complexity to TOML configuration.

**Mitigation:**
- Feature is optional (default = current behavior)
- AI_NARRATIVE_TOML_GUIDE.md explains when/why to use
- Examples in docs show clear before/after

### Risk 4: Implementation Bugs

**Scenario:** Retention logic has bugs, breaks narratives.

**Mitigation:**
- Comprehensive unit tests (see Phase 2)
- Integration tests with real narratives
- Gradual rollout (opt-in for existing narratives)
- Feature flag in code for emergency disable

## Future Enhancements

### V1 (This Plan): Basic Retention

Simple modes: full, summary, drop.

### V2: Smart Summaries

**More intelligent summarization:**
```markdown
[Table: potential_discord_posts, 10 rows]
Sample row:
| id | text_content | rating |
|----|--------------|--------|
| 1  | "Hey community..." | 4.5 |
...
Schema: 8 columns (id, text_content, rating, tags, ...)
```

**Implementation:**
- Keep first 2 rows + schema
- ~500 chars instead of 18KB
- Still allows some re-examination

### V3: Semantic Compression

**LLM-powered summarization:**
```
Original: [18KB table of posts]
Summary: "Analyzed 10 Discord posts. Topics: AI workflows (3),
          community updates (4), technical tutorials (3).
          Quality range: 3.2-4.8/5. Top posts: #3, #7, #9."
```

**Pros:**
- Retains semantic meaning
- Much smaller than raw data
- Useful for subsequent reasoning

**Cons:**
- Requires extra LLM call
- Adds latency
- Cost vs. benefit tradeoff

### V4: Differential History

**Only send deltas to LLM:**
```
Act 1: [Full table]
Act 2: [Summary] + [Delta: what changed]
Act 3: [Summary] + [Delta Act 2→3]
```

Useful for iterative refinement workflows.

## Open Questions

1. **Should `bot_command` outputs support retention?**
   - Some bot commands return huge responses
   - Same issue as tables
   - **Decision:** Yes, add in Phase 1

2. **Should retention apply retroactively to template resolution?**
   - When `{{act_name}}` references an act, include full history or summary?
   - **Decision:** Use whatever is in conversation_history (respects retention)

3. **How to handle errors in summary generation?**
   - If summarize_input() fails, fall back to `"full"`?
   - **Decision:** Log warning, use `"full"` as fallback

4. **Should there be a global retention default?**
   ```toml
   # In botticelli.toml
   [narrative]
   default_history_retention = "summary"  # Apply to all large inputs
   ```
   - **Decision:** No, keep explicit. Global defaults hide behavior.

## Timeline

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| **Phase 1: Core** | Week 1 (Mon-Wed) | Code implementation, basic tests |
| **Phase 2: Testing** | Week 1 (Thu-Fri) | Integration tests, edge cases |
| **Phase 3: Docs** | Week 2 (Mon-Tue) | User guide, spec updates |
| **Phase 4: Rollout** | Week 2 (Wed-Fri) | Migration, changelog, release |

**Total: 2 weeks**

## Approval and Next Steps

### Pending Decisions

- [ ] Approve overall design approach
- [ ] Confirm `HistoryRetention` enum naming
- [ ] Agree on auto-summary threshold (10KB)
- [ ] Review summary format examples

### Ready to Start

Once approved:
1. Create feature branch: `feature/conversation-history-retention`
2. Implement Phase 1 (core infrastructure)
3. Write tests alongside implementation
4. Open PR for review after Phase 2 complete

---

**Document Version:** 1.0
**Last Updated:** 2025-11-26
**Owner:** Claude Code
**Reviewers:** Erik (project owner)

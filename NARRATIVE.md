# Narrative System Implementation Plan

## Overview

Implement a narrative execution system that reads multi-act prompts from TOML files and executes them in sequence against LLM APIs. This enables automated multi-step content generation workflows.

## Main Goals

1. Parse narrative TOML files with metadata, table of contents, and acts
2. Execute prompts in the order specified by the table of contents
3. Pass context between acts (each act can reference previous outputs)
4. Store the complete narrative execution history in the database
5. Provide a CLI interface to run narratives

## Example Use Case

The `narrations/mint.toml` file defines a three-act narrative for generating social media content:
- Act 1: Generate initial content
- Act 2: Critique the content
- Act 3: Improve based on critique

## Implementation Steps

### Step 1: Define Narrative Data Structures ✓ COMPLETE

**Completed:**
- Created `src/narrative/` module with proper organization:
  - `core.rs` - Data structures and parsing logic
  - `error.rs` - Error types following project conventions
  - `mod.rs` - Module exports only
- Implemented `Narrative` struct with:
  - `NarrativeMetadata` - name and description
  - `NarrativeToc` - ordered list of act names
  - `acts: HashMap<String, String>` - map of act names to prompts
- Added TOML parsing with `toml` crate
- Implemented `FromStr` trait for idiomatic parsing
- Added comprehensive validation:
  - Non-empty table of contents
  - All acts referenced in toc exist
  - No empty prompts
- Error handling with `NarrativeError` and `NarrativeErrorKind`
  - Integrated into crate-level `BoticelliError`
  - Uses `derive_new` for clean construction
  - Uses `derive_more::Display` for formatting
- Full compliance with CLAUDE.md guidelines:
  - Proper derives: `Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize`
  - Module organization with types in core.rs
  - Exported at crate level in lib.rs
- Tests: 5 passing tests covering validation and parsing

### Step 2: Create Narrative Parser ✓ COMPLETE

**Completed:**
- Parser integrated into `src/narrative/core.rs`
- `Narrative::from_file()` - loads from TOML file path
- `FromStr` trait implementation - parses from TOML string
- Comprehensive error handling:
  - File I/O errors (`FileRead`)
  - TOML parse errors (`TomlParse`)
  - Validation errors (`EmptyToc`, `MissingAct`, `EmptyPrompt`)
- Unit tests in `tests/narrative_test.rs`:
  - Loads `narrations/mint.toml` successfully
  - Validates empty toc rejection
  - Validates missing act detection
  - Validates empty prompt detection
  - Validates well-formed narratives

### Step 3: Implement Narrative Executor ✓ COMPLETE

**Completed:**
- Created `NarrativeExecutor<D: BoticelliDriver>` in `src/narrative/executor.rs`
- Implemented sequential act processing:
  - Builds `GenerateRequest` with conversation history
  - Calls LLM API using `BoticelliDriver::generate()`
  - Extracts text responses from `Output` enum
  - Maintains alternating User/Assistant message history
- Context passing strategy: conversation history approach
  - Each act sees all previous outputs as conversation context
  - Enables multi-step workflows (generate → critique → improve)
- Data structures:
  - `ActExecution` - stores inputs, model, temperature, max_tokens, response, and metadata
  - `NarrativeExecution` - aggregates complete execution with all acts
- Proper derives: `Debug, Clone, PartialEq, Serialize, Deserialize`
- Error handling with `BoticelliResult`
- Exported types at crate level in `lib.rs`
- Tests: 6 passing tests in `tests/narrative_executor_test.rs`
  - Mock driver for deterministic testing
  - Tests cover: single/multiple acts, context passing, driver access
  - Context tracking test validates conversation history growth
  - Trait abstraction test with in-memory provider
  - Multimodal configuration test

### Step 3.5: Architecture Improvements ✓ COMPLETE

**NarrativeProvider Trait Abstraction:**
- Created `NarrativeProvider` trait in `src/narrative/provider.rs`
- Decouples executor from TOML configuration format
- Trait methods:
  - `name()` - Get narrative identifier
  - `act_names()` - Get ordered act list
  - `get_act_config()` - Retrieve act configuration
- Benefits:
  - Format flexibility (easy to add YAML, JSON, database sources)
  - Better testability (simple mock implementations)
  - Reduced coupling (config changes don't ripple through executor)
- `Narrative` struct implements `NarrativeProvider`
- `NarrativeExecutor::execute()` is generic over `NarrativeProvider`

**Multimodal and Per-Act Configuration:**
- Created `ActConfig` struct for flexible act configuration:
  - `inputs: Vec<Input>` - Supports text, images, audio, video, documents
  - `model: Option<String>` - Per-act model override
  - `temperature: Option<f32>` - Per-act temperature override
  - `max_tokens: Option<u32>` - Per-act max_tokens override
- Builder pattern methods:
  - `ActConfig::from_text()` - Simple text constructor
  - `ActConfig::from_inputs()` - Multimodal constructor
  - `.with_model()`, `.with_temperature()`, `.with_max_tokens()` - Fluent API
- Executor applies per-act overrides to `GenerateRequest`
- `ActExecution` stores full configuration used for each act
- Tests demonstrate:
  - Per-act model selection (GPT-4, Claude, Gemini)
  - Per-act temperature/max_tokens overrides
  - Multimodal inputs (text + image in single act)

**TOML Specification Design:**
- Created `NARRATIVE_TOML_SPEC.md` - Complete specification
- Created `narrations/showcase.toml` - Comprehensive example
- Format features:
  - Backward compatible simple text: `act = "text"`
  - Structured acts with `[[acts.act_name.input]]` array-of-tables
  - Native TOML syntax (idiomatic, readable)
  - Multiple input types per act
  - Source flexibility: `url`, `base64`, `file` fields
  - Per-act configuration overrides
- Example narrative demonstrates:
  - 8 acts with different configurations
  - Vision (text+image), audio transcription, video analysis
  - Document review, creative brainstorming, technical synthesis
  - Different models per act (GPT-4, Claude, Gemini, Whisper)

**Note:** TOML parsing for new multimodal format not yet implemented.
Current parser only handles simple text acts. Parsing implementation deferred to future work.

### Step 4: Database Schema for Narrative Executions

- Create table `narrative_executions` to store execution metadata
  - `id` - unique identifier
  - `narrative_name` - which narrative was run
  - `started_at` - timestamp
  - `completed_at` - timestamp (nullable)
  - `status` - (running, completed, failed)
- Create table `narrative_act_outputs` to store individual act results
  - `id` - unique identifier
  - `execution_id` - foreign key to narrative_executions
  - `act_name` - which act this is from
  - `prompt` - the prompt that was sent
  - `response` - the LLM response
  - `sequence_number` - order in the execution
  - `created_at` - timestamp
- Add Diesel migrations
- Create Rust models for these tables

### Step 5: CLI Interface

- Use `clap` crate to define command-line arguments
- Add CLI command to run narratives (e.g., `--narrative narrations/mint.toml`)
- Add option to specify which LLM backend to use
- Display progress as acts execute
- Show final results and where they're stored

### Step 6: Testing and Documentation

- Write integration tests that run a test narrative end-to-end
- Add example narratives to demonstrate capabilities
- Document the TOML format specification
- Update README with narrative usage examples

## Resolved Design Decisions

1. **Context Passing** ✓ RESOLVED: Each act sees all previous outputs via conversation history.
   - Implemented using alternating User/Assistant messages
   - More flexible than immediate predecessor only
   - Enables complex multi-step workflows

2. **Multiple Models** ✓ RESOLVED: Yes, per-act model selection supported.
   - Implemented via `ActConfig.model` optional override
   - Act 1 can use GPT-4, Act 2 can use Claude, etc.
   - Enables using best model for each task type

3. **Multimodal Inputs** ✓ RESOLVED: Fully supported via `ActConfig.inputs: Vec<Input>`.
   - Acts can combine text, images, audio, video, documents
   - Flexible source types: URL, base64, file paths
   - TOML spec designed for all input types

4. **Configuration Format** ✓ RESOLVED: Trait-based abstraction with TOML implementation.
   - `NarrativeProvider` trait decouples format from execution
   - TOML spec uses idiomatic array-of-tables syntax
   - Easy to add YAML, JSON, or database sources later

## Open Questions

1. **Streaming**: Should we support streaming outputs for narrative execution?
   - Would require streaming version of executor
   - Could show incremental progress during long generations

2. **Error Handling**: If act 2 fails, should we store partial results or rollback?
   - Current: propagates error immediately
   - Could add retry logic, partial saving, or checkpoint/resume

3. **Variables**: Should we support variable substitution in prompts (e.g., `${act1.response}`)?
   - Current: entire conversation history available
   - Explicit variables could enable more precise references

4. **Parallelization**: Should we support parallel act execution for independent acts?
   - Current: strictly sequential
   - Could add DAG-based execution for independent branches

## Dependencies

- ✓ **Added**: `toml = "0.8"` - TOML parsing for narrative files
- ✓ **Added**: `clap = "4"` - CLI argument parsing
- ✓ **Added**: `derive-new = "0.7"` - Clean error construction
- Existing: `serde` for deserialization (already in project)
- Existing: `derive_more` for Display/Error derives
- Existing: Database infrastructure (Diesel) - for future steps
- Existing: BoticelliDriver trait for LLM calls - integrated ✓

## Current Implementation Status

### ✅ Completed (Fully Functional)
- Core data structures (`Narrative`, `NarrativeMetadata`, `NarrativeToc`)
- Simple text TOML parsing (`mint.toml` format)
- Narrative executor with conversation history
- Trait-based architecture (`NarrativeProvider`, `ActConfig`)
- Multimodal input support (architecture ready)
- Per-act configuration (model, temperature, max_tokens)
- Comprehensive test suite (11 tests passing)
- TOML specification document
- Example narratives (mint.toml, showcase.toml)

### 🚧 Next Implementation Tasks

**Immediate (Implement Multimodal TOML Parsing):**
1. Create serde deserialization for `ActConfig` from TOML
2. Implement custom deserializer for `Input` enum with `[[input]]` tables
3. Handle source type detection (url/base64/file)
4. Support mixed simple/structured acts in same narrative
5. Add validation for multimodal inputs
6. Update tests to parse and execute showcase.toml

**Near-term (Database Integration):**
1. Database schema (Step 4)
2. Diesel migrations
3. Models for narrative_executions and narrative_act_outputs
4. Save/load execution history

**Future (CLI and Advanced Features):**
1. CLI interface (Step 5)
2. Streaming support
3. Checkpoint/resume for long narratives
4. Variable substitution
5. Parallel execution for independent acts

## Files and Locations

**Core Implementation:**
- `src/narrative/core.rs` - Data structures and simple TOML parsing
- `src/narrative/provider.rs` - NarrativeProvider trait and ActConfig
- `src/narrative/executor.rs` - NarrativeExecutor implementation
- `src/narrative/error.rs` - Error types
- `src/narrative/mod.rs` - Module exports

**Tests:**
- `tests/narrative_test.rs` - Parser and validation tests (5 tests)
- `tests/narrative_executor_test.rs` - Executor tests (6 tests)

**Documentation:**
- `NARRATIVE.md` - This file (implementation plan)
- `NARRATIVE_TOML_SPEC.md` - Complete TOML format specification

**Examples:**
- `narrations/mint.toml` - Simple text-only narrative
- `narrations/showcase.toml` - Comprehensive multimodal example

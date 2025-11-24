# Narrative Composition Implementation

## Status: ✅ COMPLETED

Successfully implemented narrative composition, allowing narratives to reference and execute other narratives within the same TOML file.

## Implementation Summary

### Core Components

**MultiNarrative Container** (`multi_narrative.rs`):
- Loads all narratives from a TOML file into a HashMap
- Implements `NarrativeProvider` trait
- Provides `resolve_narrative()` to access sibling narratives
- Supports both with and without database features

**NarrativeProvider Trait** (`provider.rs`):
- Added `resolve_narrative(&self, name: &str) -> Option<&dyn NarrativeProvider>`
- Default implementation returns `None` for single narratives
- MultiNarrative overrides to return sibling narratives

**ActConfig** (`provider.rs`):
- Already had `narrative_ref: Option<String>` field
- Already had `from_narrative_ref()` constructor
- Already had `is_narrative_ref()` check method

**TOML Parsing** (`toml_parser.rs`):
- `TomlActConfig` already supported `narrative: Option<String>` field
- Updated `TomlAct::Structured` branch to handle narrative references
- Checks for mutual exclusivity between `narrative` and `input` fields

**Executor** (`executor.rs`):
- Updated `execute()` to accept `N: NarrativeProvider + ?Sized`
- Updated `execute_carousel()` to accept `?Sized`
- Updated `process_inputs()` to accept `?Sized`
- Implemented recursive execution for narrative references
- Resolves references via `narrative.resolve_narrative()`
- Executes nested narrative and collects all responses
- Adds combined response to conversation history

**CLI** (`cli/run.rs`):
- Conditionally loads `MultiNarrative` when `--narrative-name` provided
- Falls back to single `Narrative` for backwards compatibility
- Uses `Box<dyn NarrativeProvider>` to handle both types
- Passes trait object references to executor methods

### TOML Syntax

```toml
# Shared acts available to all narratives
[acts]
critique = "Analyze this content critically..."
refine = "Improve based on critique..."

# Multiple narrative definitions
[narratives.community]
name = "potential_posts_community"
description = "Generate community-focused posts"
template = "potential_posts"

[narratives.community.toc]
order = ["generate", "critique", "refine"]

[[narratives.community.acts.generate.input]]
type = "text"
content = "Generate a community-building post about Botticelli..."

# Wrapper narrative using composition
[narratives.batch_generate]
name = "generation_batch_50"
description = "Generate 50 posts across all categories"

[narratives.batch_generate.toc]
order = ["community", "problem", "tutorial", "usecase", "feature"]

[narratives.batch_generate.acts.community]
narrative = "community"  # References another narrative

[narratives.batch_generate.acts.problem]
narrative = "problem"
```

## Testing

Successfully tested with `generation_carousel.toml`:
- Loads 6 narratives from single file (batch_generate + 5 topic-specific)
- Executes wrapper narrative with 5 composition references
- Recursively executes each referenced narrative (3 acts each)
- Generates content into separate tables per topic
- Rate limiting applies correctly across all nested calls
- Budget multipliers work as expected (80% RPM/TPM/RPD)

## Architecture Benefits

1. **Modularity**: Reusable narrative components
2. **Composition**: Build complex workflows from simple parts
3. **DRY**: Share acts (critique, refine) across narratives
4. **Scalability**: Easy to add new topic narratives
5. **Testing**: Test individual narratives in isolation
6. **Maintenance**: Single file for related narratives

## Usage

Load multi-narrative file:
```bash
just narrate generation_carousel.batch_generate
```

The CLI detects the narrative name and loads `MultiNarrative` instead of single `Narrative`, enabling composition features automatically.

## Related Files

- `crates/botticelli_narrative/src/multi_narrative.rs` - Container
- `crates/botticelli_narrative/src/executor.rs` - Recursive execution
- `crates/botticelli_narrative/src/toml_parser.rs` - Parse narrative refs
- `crates/botticelli_narrative/src/provider.rs` - Trait definition
- `crates/botticelli/src/cli/run.rs` - CLI integration
- `crates/botticelli_narrative/narratives/discord/generation_carousel.toml` - Example

## Future Enhancements

Potential improvements:
- Cross-file narrative references
- Parameterized narratives (pass arguments)
- Conditional narrative execution
- Parallel narrative execution (when independent)
- Narrative recursion limits (prevent infinite loops)

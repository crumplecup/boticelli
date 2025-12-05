# Narrative TOML Validator Design

## Overview

A comprehensive validator for narrative TOML files that catches common syntax errors before execution and provides actionable, context-aware error messages. The validator integrates into the main `botticelli` binary as a subcommand and runs automatically during narrative parsing.

## Design Principles

1. **Single binary**: Validation is a subcommand of `botticelli`, not a separate binary
2. **Automatic validation**: Narratives are validated before execution without explicit flag
3. **Clean validation = proceed**: If validation passes, execution begins immediately
4. **Actionable errors**: Every error includes specific fix suggestions

## Motivation

**Current state:**

- Errors only surface during execution
- Generic TOML parse errors lack narrative-specific context
- Common mistakes (`[[acts]]` instead of `[acts.name]`) produce cryptic messages
- No way to validate narratives without running them

**Goals:**

- Catch syntax errors at parse time with clear messages
- Provide actionable fix suggestions for common mistakes
- Enable pre-flight validation (CI/CD, editor integration)
- Improve developer experience and reduce debugging time

## Validation Layers

### 1. TOML Syntax Validation (Already Exists)

Basic TOML parsing via `toml::from_str()`. Catches:

- Invalid TOML syntax
- Type mismatches
- Missing required fields

**Current error quality:** Generic TOML errors without narrative context.

### 2. Structural Validation (Partially Exists)

Validates narrative structure via `Narrative::validate()`. Currently checks:

- ✅ `toc.order` is not empty (`EmptyToc`)
- ✅ All acts in `toc.order` exist in `acts` map (`MissingAct`)
- ✅ All acts have at least one input or are narrative refs (`EmptyPrompt`)

**Gaps:**

- No validation of resource references (e.g., `"bots.undefined"`)
- No detection of common syntax mistakes (array vs table syntax)
- No validation of nested narrative paths
- No validation of media file paths
- No validation of model names against known providers

### 3. Semantic Validation (NEW - This Design)

Context-aware validation with actionable suggestions:

#### Common Syntax Mistakes

| Mistake                        | Detection                                  | Suggestion                                                                                      |
| ------------------------------ | ------------------------------------------ | ----------------------------------------------------------------------------------------------- |
| `[[acts]]`                     | Array of tables instead of table of tables | Use `[acts.name]` or `acts.name = "..."`                                                        |
| `[[narrative]]`                | Multiple narrative sections                | Use single `[narrative]` section                                                                |
| `[acts] name = "..."`          | Direct field in `[acts]`                   | Use `acts.name = "..."` or `[acts.name]`                                                        |
| Missing `[toc]` or `toc` field | No table of contents                       | Add `[toc]`<br>`order = ["act1", "act2"]` or<br>`toc = ["act1", "act2"]` in `[narratives.name]` |
| `order = "act1"`               | String instead of array                    | Change to `order = ["act1"]`                                                                    |
| Empty `toc.order`              | No acts to execute                         | Add at least one act to `toc.order`                                                             |

#### Resource Reference Validation

Validate that all references resolve:

```toml
[bots.get_stats]
platform = "discord"
command = "server.get_stats"

[acts]
fetch = "bots.get_stats"     # ✅ Valid - bots.get_stats exists
analyze = "bots.undefined"   # ❌ Error: bot 'undefined' not defined in [bots] section
query = "tables.users"       # ❌ Error: table 'users' not defined in [tables] section
```

**Error messages:**

```
Error: Reference 'bots.undefined' in act 'analyze' not found
  → No bot named 'undefined' is defined in the [bots] section

  Defined bots:
    - get_stats

  Fix: Define the bot or use an existing one:
    [bots.undefined]
    platform = "discord"
    command = "..."
```

#### Nested Narrative Validation

```toml
[acts]
preprocess = "narrative:data_prep"  # Validate path exists
```

**Validation:**

- Check if `narratives/data_prep.toml` exists
- Optionally: Recursively validate nested narrative
- Detect circular dependencies

#### Media File Validation

```toml
[media.logo]
file = "./logo.png"  # Check file exists

[media.screenshot]
url = "https://example.com/image.jpg"  # Validate URL format

[media.invalid]
# ❌ Error: Must specify one of: file, url, or base64
```

**Checks:**

- File paths exist (relative to narrative file)
- URLs have valid format
- At least one source specified
- MIME type inference possible or explicitly set

#### Model Name Validation

```toml
[narrative]
model = "gemini-2.0-flash-exp"  # ✅ Known model

[acts.custom]
model = "gpt-5-turbo"  # ⚠️ Warning: Unknown model (typo in gpt-4-turbo?)
```

**Strategy:**

- Maintain list of known model names per provider
- Warn (not error) on unknown models
- Suggest closest match on typos

#### Table Reference Validation

```toml
[tables.users]
table_name = "users"

[acts]
query = "tables.users"  # ✅ Valid

[acts.invalid]
[[acts.invalid.input]]
type = "table"
table_name = "undefined"  # ⚠️ Warning: Direct table reference without [tables] definition
```

**Behavior:**

- Friendly syntax references must resolve
- Verbose syntax direct references: warn but allow (runtime flexibility)

## Validator Implementation

### Module Structure

```
crates/botticelli_narrative/src/
├── validator.rs           # NEW - Main validator
├── validator/
│   ├── syntax.rs          # Syntax pattern detection
│   ├── semantic.rs        # Resource reference validation
│   └── suggestions.rs     # Error message generation with fix suggestions
```

### Validator API

```rust
use crate::validator::{ValidationResult, ValidationWarning, ValidationError};

/// Validate a narrative TOML string.
pub fn validate_narrative_toml(toml: &str) -> ValidationResult {
    // Returns all errors and warnings with fix suggestions
}

/// Validate a narrative file.
pub fn validate_narrative_file(path: impl AsRef<Path>) -> ValidationResult {
    // Convenience wrapper
}

/// Configuration for validation behavior.
pub struct ValidationConfig {
    /// Check that nested narrative files exist
    pub validate_nested_narratives: bool,
    /// Check that media files exist
    pub validate_media_files: bool,
    /// Warn on unknown model names
    pub warn_unknown_models: bool,
    /// Base directory for relative path resolution
    pub base_dir: Option<PathBuf>,
}
```

### Validation Result Structure

```rust
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

pub struct ValidationError {
    pub kind: ValidationErrorKind,
    pub location: Option<ValidationLocation>,
    pub message: String,
    pub suggestion: Option<String>,
}

pub struct ValidationWarning {
    pub kind: ValidationWarningKind,
    pub location: Option<ValidationLocation>,
    pub message: String,
}

pub struct ValidationLocation {
    pub line: usize,
    pub column: usize,
    pub section: Option<String>,  // e.g., "acts.fetch_data"
}

pub enum ValidationErrorKind {
    InvalidSyntax,
    MissingSection,
    UndefinedReference,
    EmptyToc,
    MissingAct,
    EmptyPrompt,
    FileNotFound,
    CircularDependency,
}

pub enum ValidationWarningKind {
    UnknownModel,
    UnusedResource,
    DirectTableReference,
    LargeMediaFile,
}
```

### Error Message Examples

#### Example 1: Array of Tables Syntax Error

**Input:**

```toml
[[acts]]
name = "fetch"
prompt = "Get data"
```

**Output:**

```
Error: Invalid syntax at line 1
  → Found [[acts]] but acts should be a table of tables, not an array

  You have two options:

  1. Use inline table syntax (recommended for simple prompts):
     [acts]
     fetch = "Get data"

  2. Use table syntax (for acts with configuration):
     [acts.fetch]
     prompt = "Get data"
     model = "gemini-2.0-flash-exp"
```

#### Example 2: Undefined Resource Reference

**Input:**

```toml
[bots.get_stats]
platform = "discord"
command = "server.get_stats"

[acts]
analyze = "bots.stats"  # Typo: should be 'get_stats'
```

**Output:**

```
Error: Undefined reference 'bots.stats' at line 6 in act 'analyze'
  → No bot named 'stats' is defined

  Available bots:
    - get_stats

  Did you mean 'bots.get_stats'?

  Fix:
    analyze = "bots.get_stats"
```

#### Example 3: Missing File

**Input:**

```toml
[media.logo]
file = "./missing.png"
```

**Output:**

```
Error: Media file not found at line 2
  → File './missing.png' does not exist

  Searched in: /path/to/narratives/missing.png

  Check that:
    - File path is correct
    - File exists relative to narrative file
    - File permissions allow reading
```

## CLI Tool

This should be a feature available on our core "botticelli" binary, with a --validate flag or something similar. Likewise, we may want to use this internally to validate narratives before running them, where a clean validation means it is good to go.

I do not like having to specify --bin botticelli because there are extra binaries. We should have only one business binary.

### Binary: `botticelli-narrative-validator`

```bash
# Validate single file
botticelli-narrative-validator narratives/my_narrative.toml

# Validate all narratives in directory
botticelli-narrative-validator narratives/

# JSON output for CI/CD
botticelli-narrative-validator --json narratives/ > validation.json

# Strict mode: warnings become errors
botticelli-narrative-validator --strict narratives/

# Options
--validate-files        # Check media/nested narrative files exist (default: true)
--validate-models       # Warn on unknown models (default: true)
--base-dir <path>       # Base directory for relative paths
--json                  # JSON output for programmatic use
--strict                # Treat warnings as errors
--quiet                 # Only show errors, not warnings
```

### Exit Codes

- `0` - Success, no errors or warnings
- `1` - Validation errors found
- `2` - Only warnings found (or with `--strict`)
- `3` - Internal validator error

### JSON Output Format

```json
{
  "valid": false,
  "file": "narratives/test.toml",
  "errors": [
    {
      "kind": "UndefinedReference",
      "location": {
        "line": 6,
        "column": 10,
        "section": "acts.analyze"
      },
      "message": "Undefined reference 'bots.stats'",
      "suggestion": "Did you mean 'bots.get_stats'?"
    }
  ],
  "warnings": [
    {
      "kind": "UnknownModel",
      "location": {
        "line": 3,
        "column": 8,
        "section": "narrative"
      },
      "message": "Unknown model 'gpt-5-turbo'"
    }
  ]
}
```

## Integration Points

### 1. Narrative Parsing (`core.rs`)

Add automatic pre-flight validation before parsing:

```rust
impl Narrative {
    pub fn from_toml_str(s: &str, name: Option<&str>) -> Result<Self, NarrativeError> {
        // NEW: Run validator first (automatic validation)
        let validation = crate::validator::validate_narrative_toml(s)?;
        if !validation.errors.is_empty() {
            return Err(NarrativeError::new(
                NarrativeErrorKind::ValidationFailed(validation)
            ));
        }

        // Existing parsing logic
        let toml_narrative_file: toml_parser::TomlNarrativeFile = toml::from_str(s)
            .map_err(|e| NarrativeError::new(NarrativeErrorKind::TomlParse(e.to_string())))?;

        // ...
    }
}
```

**Key behavior:**
- Validation runs automatically on every narrative load
- Clean validation = proceed to execution
- Any validation errors = immediate failure with actionable messages
- No separate validation step needed for normal execution

### 2. MCP Resource (`narrative.rs`)

Validate on read:

```rust
impl McpResource for NarrativeResource {
    async fn read(&self, uri: &str) -> McpResult<String> {
        let name = self.parse_uri(uri)?;
        let content = self.read_narrative(&name)?;

        // NEW: Validate before returning
        let validation = crate::validator::validate_narrative_toml(&content)
            .map_err(|e| McpError::ValidationFailed(e.to_string()))?;

        if !validation.errors.is_empty() {
            return Err(McpError::InvalidResource(format!(
                "Narrative '{}' has validation errors:\n{}",
                name,
                validation.format_errors()
            )));
        }

        Ok(content)
    }
}
```

### 3. CI/CD Integration

GitHub Actions workflow:

```yaml
name: Validate Narratives

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release
      - run: |
          ./target/release/botticelli validate \
            --json \
            --strict \
            crates/botticelli_narrative/narratives/ \
            > validation.json
      - name: Upload validation results
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: validation-errors
          path: validation.json
```

### 4. Editor Integration (Future)

Language server protocol (LSP) integration for real-time validation:

- VS Code extension
- Neovim integration
- Syntax highlighting + inline diagnostics
- Auto-completion for resource references

## Implementation Phases

### Phase 1: Core Validator (MVP) ✅ COMPLETE

- [x] Syntax pattern detection (`[[acts]]` → error)
- [x] Resource reference validation (bots, tables, media)
- [x] File existence checks (media, nested narratives) - Structure only
- [x] Basic error messages with suggestions
- [x] `botticelli validate` subcommand with basic output
- [x] Automatic validation in `Narrative::from_toml_str()` - Ready for integration

### Phase 2: Enhanced Diagnostics ✅ COMPLETE

- [x] Model name validation with fuzzy matching (Levenshtein distance ≤3)
- [x] Unused resource detection (bots, tables, media)
- [x] JSON output format
- [ ] Circular dependency detection (deferred - requires graph analysis)
- [ ] Location tracking (line/column numbers) (limited by TOML parser capabilities)
- [ ] Colored terminal output (deferred - would use termcolor crate)

### Phase 3: Advanced Features ✅ COMPLETE

- [x] Circular dependency detection (using petgraph + Kosaraju's SCC algorithm)
- [x] Self-reference detection
- [ ] Recursive nested narrative validation (requires file system integration)
- [ ] Schema validation for custom extensions (deferred)
- [ ] Performance optimization (parallel validation) (premature optimization)
- [ ] Watch mode (`--watch` for development) (separate feature)
- [ ] Integration with MCP resource system (separate integration)
- [ ] Validation caching (premature optimization)

### Phase 4: Developer Experience

- [ ] LSP server for editor integration
- [ ] VS Code extension
- [ ] Auto-fix capabilities (apply suggestions)
- [ ] Validation rule configuration
- [ ] Custom rule plugins

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_array_of_tables_acts() {
        let toml = r#"
            [[acts]]
            name = "test"
        "#;

        let result = validate_narrative_toml(toml);
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(
            result.errors[0].kind,
            ValidationErrorKind::InvalidSyntax
        ));
        assert!(result.errors[0].message.contains("[[acts]]"));
        assert!(result.errors[0].suggestion.is_some());
    }

    #[test]
    fn test_undefined_bot_reference() {
        let toml = r#"
            [bots.get_stats]
            platform = "discord"
            command = "test"

            [toc]
            order = ["fetch"]

            [acts]
            fetch = "bots.undefined"
        "#;

        let result = validate_narrative_toml(toml);
        assert_eq!(result.errors.len(), 1);
        assert!(matches!(
            result.errors[0].kind,
            ValidationErrorKind::UndefinedReference
        ));
    }
}
```

### Integration Tests

Test against real narrative files:

```rust
#[test]
fn test_validate_all_example_narratives() {
    let narratives_dir = PathBuf::from("narratives");

    for entry in std::fs::read_dir(narratives_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension() == Some("toml".as_ref()) {
            let result = validate_narrative_file(&path);

            // All example narratives should be valid
            assert!(
                result.errors.is_empty(),
                "Narrative {:?} has errors: {:?}",
                path,
                result.errors
            );
        }
    }
}
```

## Success Metrics

- **Error detection rate**: Catch 95%+ of common mistakes before execution
- **False positive rate**: <5% incorrect errors/warnings
- **Performance**: Validate 100+ narratives in <1 second
- **Developer satisfaction**: Reduced time debugging TOML syntax issues
- **Adoption**: Integrated into 100% of narrative workflows (CI/CD, development)

## Binary Consolidation

**Current state:**
- `botticelli` - Main CLI (src/main.rs)
- `actor-server` - Actor system server (crates/botticelli_actor/src/bin/actor-server.rs)
- `botticelli-mcp` - MCP server (crates/botticelli_mcp/src/bin/botticelli-mcp.rs)

**Target state (single business binary):**
```bash
botticelli                    # Main CLI with subcommands
botticelli validate <path>    # Validate narratives
botticelli execute <path>     # Execute narrative
botticelli server             # Start actor server
botticelli mcp                # Start MCP server
botticelli tui                # Launch TUI
```

**Implementation:**
1. Move `actor-server` functionality to `botticelli server` subcommand
2. Move `botticelli-mcp` functionality to `botticelli mcp` subcommand
3. Keep single `[[bin]]` entry in workspace
4. Use clap subcommands for all functionality

**Benefits:**
- No need for `--bin` flag: `cargo build` or `cargo run` just works
- Consistent user experience: all features under one command
- Easier distribution: single binary to install
- Simpler documentation: one command reference

## Future Enhancements

1. **Auto-fix mode**: Automatically apply suggestions
2. **Interactive mode**: Prompt user to fix errors interactively
3. **Validation profiles**: Strict/lenient validation levels
4. **Custom validators**: Plugin system for organization-specific rules
5. **Benchmark suite**: Performance regression testing
6. **Documentation generation**: Extract docs from TOML structure
7. **Schema export**: Generate JSON schema from validation rules

## Implementation Status

### ✅ Completed Phases

- **Phase 1**: Core validator infrastructure (`botticelli_narrative/src/validator/`)
  - Error/warning types with `derive_more`
  - Validation result aggregation
  - Context-aware error messages
  
- **Phase 2**: Enhanced model validation
  - Structural validation (TOC, acts, bots)
  - File existence checks
  - Model name warnings
  - Reference resolution
  
- **Phase 3**: Execution tooling
  - MCP `validate_narrative` tool (validation only)
  - MCP `execute_narrative` tool (full execution with auto-validation)
  
- **Phase 4**: Multi-LLM backend support
  - Generic `generate` tool (dynamic backend selection)
  - Backend-specific tools: Gemini, Anthropic, Ollama, HuggingFace, Groq
  - Environment-based configuration
  
- **Phase 5**: Integration
  - Auto-validation in narrative execution path
  - MCP server tool registration
  
- **Phase 6**: CLI Integration
  - `botticelli validate` subcommand already implemented
  - JSON and human-readable output formats
  - Strict mode and quiet mode options
  - File and directory validation support

### 🔄 In Progress

- Graph analysis (cycles, unreachable acts)
- Performance optimizations
- Test coverage for edge cases

### 📋 Planned

- Auto-fix mode
- Interactive validation
- Custom validator plugins
- Schema export

## User Guide

### Validating Narratives

**CLI validation:**
```bash
# Validate single file
botticelli validate path/to/narrative.toml

# Validate directory
botticelli validate narratives/

# JSON output (for CI/CD)
botticelli validate narrative.toml --format json

# Strict mode (warnings as errors)
botticelli validate narrative.toml --strict

# Quiet mode (errors only)
botticelli validate narrative.toml --quiet
```

**MCP validation (for AI agents):**
```json
{
  "tool": "validate_narrative",
  "arguments": {
    "path": "narrative.toml",
    "validate_files": true,
    "validate_models": true
  }
}
```

**Automatic validation:**
- All narratives are validated before execution
- Use `botticelli execute` - validation runs automatically
- Clean validation = immediate execution
- Validation errors = abort with detailed messages

### Execution with Auto-Validation

**CLI execution:**
```bash
# Validates then executes
botticelli run --narrative path/to/narrative.toml
```

**MCP execution:**
```json
{
  "tool": "execute_narrative",
  "arguments": {
    "narrative_path": "narrative.toml",
    "save_results": true
  }
}
```

## References

- [NARRATIVE_TOML_SPEC.md](./NARRATIVE_TOML_SPEC.md) - TOML format specification
- [botticelli_error/src/narrative.rs](./crates/botticelli_error/src/narrative.rs) - Error types
- [botticelli_narrative/src/core.rs](./crates/botticelli_narrative/src/core.rs) - Current validation
- [botticelli_narrative/src/toml_parser.rs](./crates/botticelli_narrative/src/toml_parser.rs) - TOML parsing
- [botticelli_mcp/src/tools/validate_narrative.rs](./crates/botticelli_mcp/src/tools/validate_narrative.rs) - MCP validation tool
- [botticelli_mcp/src/tools/execute_narrative.rs](./crates/botticelli_mcp/src/tools/execute_narrative.rs) - MCP execution tool
- [MCP_MULTI_LLM_IMPLEMENTATION.md](./MCP_MULTI_LLM_IMPLEMENTATION.md) - Multi-backend LLM design

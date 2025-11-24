# Narrative Composition Implementation

## Current State

The infrastructure for narrative composition already exists:

1. **ActConfig** has `narrative_ref: Option<String>` field
2. **TomlActConfig** supports `narrative` field for TOML parsing  
3. **Executor** has skeleton code (lines 285-314) that detects narrative refs but doesn't execute them

## Agreed Design: Shared Acts

**Key principle**: Acts defined in `[acts.*]` are **shared** across ALL narratives in the file. 

```toml
# Shared act definitions (available to ALL narratives)
[acts.critique]
model = "gemini-2.0-flash-exp"
[[acts.critique.input]]
type = "text"
content = "Analyze and provide improvements..."

[acts.refine]
model = "gemini-2.0-flash-exp"
[[acts.refine.input]]
type = "text"
content = "Improve based on critique..."

# Narratives reference shared acts by name
[[narrative]]
name = "tech_topic"
toc = ["critique", "refine"]  # Uses shared acts

[[narrative]]
name = "feature_topic"  
toc = ["critique", "refine"]  # Uses SAME shared acts
```

## Implementation Steps

### Step 1: Multi-Narrative File Structure

Create a type to represent the parsed multi-narrative TOML:

```rust
pub struct MultiNarrativeFile {
    acts: HashMap<String, ActConfig>,  // Shared acts
    narratives: HashMap<String, NarrativeData>,  // Individual narratives
}

struct NarrativeData {
    toc: Vec<String>,  // Act names to execute
    mode: Option<Mode>,
    // ... other narrative fields
}
```

### Step 2: Update from_file to Load Shared Acts

When loading a narrative from file:

```rust
pub fn from_file(path: &Path, name: Option<&str>) -> Result<Self> {
    // 1. Parse file
    let file_content = MultiNarrativeFile::parse(path)?;
    
    // 2. Get requested narrative (or single if only one)
    let narrative_data = file_content.get_narrative(name)?;
    
    // 3. Build Narrative with shared acts
    let mut acts = Vec::new();
    for act_name in &narrative_data.toc {
        let act_config = file_content.acts.get(act_name)
            .ok_or_else(|| error!("Act '{}' not found", act_name))?
            .clone();
        acts.push(act_config);
    }
    
    // 4. Create Narrative with resolved acts
    Narrative::new(narrative_data.name, acts, narrative_data.mode)
}
```

### Step 3: Update TOML Parsing

Parse `[acts.*]` section separately from `[[narrative]]` sections:

```rust
#[derive(Deserialize)]
struct TomlMultiNarrativeFile {
    #[serde(default)]
    acts: HashMap<String, TomlActConfig>,
    
    #[serde(default)]
    narrative: Vec<TomlNarrativeData>,
}

#[derive(Deserialize)]
struct TomlNarrativeData {
    name: String,
    toc: Vec<String>,  // Act names only
    mode: Option<String>,
    // ... other fields
}
```

### Step 4: Narrative References (Composition)

For narrative-as-act (one narrative calling another):

```rust
if config.is_narrative_ref() {
    let ref_name = config.narrative_ref().as_ref().unwrap();
    
    // Get referenced narrative from provider
    let child_narrative = narrative.get_narrative(ref_name)
        .ok_or_else(|| NarrativeError::new(
            NarrativeErrorKind::ConfigurationError(
                format!("Referenced narrative '{}' not found", ref_name)
            )
        ))?;
    
    // Execute recursively
    let child_execution = self.execute(child_narrative).await?;
    
    // Extract final response from child
    let response = child_execution.final_response()
        .unwrap_or_else(|| "[Empty narrative response]".to_string());
    
    // Record as act execution
    act_executions.push(ActExecution {
        act_name: act_name.clone(),
        inputs: Vec::new(), // Child narrative's inputs are internal
        model: config.model().clone(),
        temperature: *config.temperature(),
        max_tokens: *config.max_tokens(),
        response,
        sequence_number,
    });
    
    continue;
}
```

### Step 5: Update from_file Methods

Update `Narrative::from_file` to support multi-narrative files:

```rust
pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
    // Detect if file has [narratives] section
    // If yes, load as multi-narrative with first narrative as primary
    // If no, load as single narrative (backward compatible)
}

pub fn from_file_named<P: AsRef<Path>>(path: P, name: &str) -> Result<Self> {
    // Load specific narrative from multi-narrative file
}
```

### Step 6: CLI Integration

Update CLI to pass narrative name to loader:

```rust
// Already done - CLI supports --narrative-name
let narrative = if let Some(name) = &args.narrative_name {
    Narrative::from_file_named(&path, name)?
} else {
    Narrative::from_file(&path)?
};
```

## Testing Strategy

1. **Unit Tests**: Test multi-narrative loading, child resolution
2. **Integration Tests**: Test carousel with 5 sub-narratives  
3. **API Test**: Run `generation_carousel.toml` with actual API

## Files to Modify

1. `crates/botticelli_narrative/src/core.rs` - Add multi-narrative loading
2. `crates/botticelli_narrative/src/executor.rs` - Implement recursive execution
3. `crates/botticelli_narrative/src/provider.rs` - Add get_narrative method
4. `tests/narrative_composition_test.rs` - Add tests

## Success Criteria

✅ Can load multi-narrative TOML files  
✅ Can execute narratives that reference other narratives  
✅ Works with carousel mode  
✅ Proper error handling for missing references  
✅ Tests pass

## Notes

- Keep backward compatibility with single-narrative files
- Prevent infinite recursion (detect cycles)
- Clear error messages for missing references
- Consider depth limits for safety

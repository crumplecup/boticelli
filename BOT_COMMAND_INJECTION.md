# Bot Command Dynamic Argument Injection

## Problem

Bot commands defined in `[bots.name]` sections currently have static arguments:

```toml
[bots.publish_message]
platform = "discord"
command = "messages.send"
channel_id = "123456"
content = "Static message"  # ❌ Cannot use LLM output
```

We need a way to inject previous act outputs into bot command arguments.

## Use Case

The `publish_welcome.toml` narrative:
1. Generates 9 welcome message candidates
2. LLM selects the best one
3. **Needs to publish the selected message** (not hardcoded text)

## Solution Options

### Option 1: Template Syntax in Bot Args

Use template placeholders in bot command definitions:

```toml
[bots.publish_message]
platform = "discord"
command = "messages.send"
channel_id = "123456"
content = "{{previous_act}}"  # Inject from previous act
```

**Pros:**
- Simple, intuitive syntax
- Familiar pattern (Handlebars-like)
- Works well for simple cases

**Cons:**
- Requires template parsing
- Limited to previous act only
- Hard to reference specific acts

### Option 2: Special Input Type in Acts

Define bot commands as inputs with special handling:

```toml
[acts]
publish_message = [
    {
        type = "bot",
        platform = "discord",
        command = "messages.send",
        channel_id = "123456",
        content_from_previous = true
    }
]
```

**Pros:**
- Clear intent
- Flexible (can reference any previous act)
- Type-safe in code

**Cons:**
- Verbose
- Breaks friendly syntax pattern
- Requires array-of-table syntax

### Option 3: Reference Notation

Use a reference syntax to pull from previous acts:

```toml
[bots.publish_message]
platform = "discord"
command = "messages.send"
channel_id = "123456"
content = "@select_best"  # Reference act by name
```

**Pros:**
- Explicit about dependencies
- Can reference any previous act
- Clean syntax

**Cons:**
- New syntax to learn
- Requires parsing

### Option 4: Bot Commands as Outputs

Flip the model: bot commands execute AFTER the act completes, using its output:

```toml
[acts.publish_message]
input = "The best message has been selected"

[[acts.publish_message.output]]
type = "bot"
platform = "discord"
command = "messages.send"
channel_id = "123456"
use_response = true  # Use LLM response as content
```

**Pros:**
- Clear data flow (input → LLM → output/action)
- Separates read (input) from write (output) operations
- Aligns with "actions as consequences" model

**Cons:**
- Major spec change
- Complex implementation
- May be overkill

## Recommended Solution: Option 1 + Option 3 Hybrid

Use template syntax with act reference capability:

```toml
[bots.publish_message]
platform = "discord"
command = "messages.send"
channel_id = "123456"
content = "{{select_best}}"  # Reference specific act
# OR
content = "{{previous}}"      # Reference previous act
```

### Implementation Plan

1. **Parser Changes** (`botticelli_narrative/src/parser.rs`):
   - Detect `{{...}}` template syntax in bot command string fields
   - Store template metadata with bot command config

2. **Executor Changes** (`botticelli_narrative/src/executor.rs`):
   - When processing bot commands in acts, check for templates
   - Resolve templates against execution context (previous acts)
   - Substitute resolved values before sending to bot registry

3. **Template Resolution**:
   ```rust
   fn resolve_template(
       template: &str,
       act_executions: &[ActExecution],
       current_index: usize,
   ) -> Result<String> {
       // Parse {{act_name}} or {{previous}}
       // Look up act in execution history
       // Return act's response text
   }
   ```

4. **TOML Spec Update**:
   - Document template syntax
   - Provide examples
   - Explain execution order requirements

### Special Cases

**Multiple templates:**
```toml
content = "Selected: {{select_best}} (from {{generate_content}})"
```

**JSON fields:**
```toml
# For structured commands
embed = '''
{
  "title": "Welcome",
  "description": "{{select_best}}"
}
'''
```

**Error handling:**
- Referenced act doesn't exist → error
- Referenced act hasn't run yet → error
- Template syntax malformed → error

## Next Steps

1. Implement template parsing in bot command parser
2. Add template resolution to executor
3. Update NARRATIVE_TOML_SPEC.md with template syntax
4. Test with publish_welcome.toml
5. Consider extending to table queries if useful

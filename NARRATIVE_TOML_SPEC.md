# Narrative TOML Specification

This document defines the TOML configuration format for multi-act narrative execution.

## Overview

A narrative TOML file consists of three main sections:
1. `[narration]` - Metadata about the narrative
2. `[toc]` - Table of contents defining execution order
3. `[acts]` - Act definitions with prompts and optional configurations

## Basic Structure

```toml
[narration]
name = "narrative_name"
description = "What this narrative does"

[toc]
order = ["act1", "act2", "act3"]

[acts]
act1 = "Simple text prompt"
act2 = "Another text prompt"
```

## Section Reference

### `[narration]` - Metadata

Required fields:
- `name` (string): Unique identifier for this narrative
- `description` (string): Human-readable description

### `[toc]` - Table of Contents

Required fields:
- `order` (array of strings): Act names in execution order

Acts execute sequentially in this order, with each act seeing previous outputs as conversation context.

### `[acts]` - Act Definitions

Acts can be defined in two ways:

#### Simple Text Acts (Backward Compatible)

```toml
[acts]
act_name = "Text prompt goes here"
```

This creates an act with:
- Single text input
- No model override (uses executor default)
- No temperature/max_tokens overrides

#### Structured Acts (Full Configuration)

```toml
[acts.act_name]
inputs = [...]      # Required: array of input objects
model = "..."       # Optional: model override
temperature = 0.7   # Optional: temperature override (0.0 - 1.0)
max_tokens = 1000   # Optional: max tokens override
```

## Input Types

### Text Input

```toml
{ type = "text", content = "The text content" }
```

### Image Input

```toml
# From URL
{ type = "image", mime = "image/png", source = { url = "https://example.com/image.png" } }

# From base64
{ type = "image", mime = "image/jpeg", source = { base64 = "iVBORw0KGgo..." } }

# From file path
{ type = "image", mime = "image/png", source = { file = "/path/to/image.png" } }
```

Supported MIME types: `image/png`, `image/jpeg`, `image/webp`, `image/gif`

### Audio Input

```toml
{ type = "audio", mime = "audio/mp3", source = { url = "https://example.com/audio.mp3" } }
```

Supported MIME types: `audio/mp3`, `audio/wav`, `audio/ogg`, `audio/webm`

### Video Input

```toml
{ type = "video", mime = "video/mp4", source = { url = "https://example.com/video.mp4" } }
```

Supported MIME types: `video/mp4`, `video/webm`, `video/avi`, `video/mov`

### Document Input

```toml
{
    type = "document",
    mime = "application/pdf",
    source = { url = "https://example.com/doc.pdf" },
    filename = "doc.pdf"  # Optional
}
```

Supported MIME types: `application/pdf`, `text/plain`, `text/markdown`, `application/json`

## Source Types

Media sources can be specified in three ways:

### URL Source
```toml
source = { url = "https://example.com/media.png" }
```

### Base64 Source
```toml
source = { base64 = "iVBORw0KGgoAAAANSUhEUgAAA..." }
```

### File Source
```toml
source = { file = "/absolute/path/to/media.png" }
```

## Configuration Overrides

### Model Override

Specify which LLM model to use for this act:

```toml
[acts.vision_task]
inputs = [...]
model = "gemini-pro-vision"
```

Common values:
- `"gpt-4"`, `"gpt-4-turbo"`, `"gpt-3.5-turbo"`
- `"claude-3-opus-20240229"`, `"claude-3-5-sonnet-20241022"`
- `"gemini-pro"`, `"gemini-pro-vision"`

### Temperature Override

Controls randomness/creativity (0.0 = deterministic, 1.0 = creative):

```toml
[acts.creative_task]
inputs = [...]
temperature = 0.9  # High creativity
```

```toml
[acts.analytical_task]
inputs = [...]
temperature = 0.2  # Low randomness, more focused
```

### Max Tokens Override

Limits the response length:

```toml
[acts.brief_summary]
inputs = [...]
max_tokens = 200  # Short response
```

## Complete Examples

### Example 1: Simple Text-Only Narrative (mint.toml style)

```toml
[narration]
name = "mint"
description = "Generate social media content"

[toc]
order = ["act1", "act2", "act3"]

[acts]
act1 = "Create social media posts for MINT homeless shelter"
act2 = "Critique the posts for quality and impact"
act3 = "Improve the posts based on critique"
```

### Example 2: Vision Analysis with Model Override

```toml
[narration]
name = "logo_review"
description = "Analyze a logo design"

[toc]
order = ["analyze", "suggest_improvements"]

[acts.analyze]
inputs = [
    { type = "text", content = "Analyze this logo for visual appeal, memorability, and brand alignment" },
    { type = "image", mime = "image/png", source = { url = "https://example.com/logo.png" } }
]
model = "gemini-pro-vision"
temperature = 0.3

[acts.suggest_improvements]
inputs = [
    { type = "text", content = "Suggest 5 specific improvements to make this logo more effective" }
]
temperature = 0.7
```

### Example 3: Multi-Modal Act

```toml
[acts.comprehensive_analysis]
inputs = [
    { type = "text", content = "Analyze these materials together" },
    { type = "image", mime = "image/png", source = { url = "https://example.com/chart.png" } },
    { type = "document", mime = "application/pdf", source = { url = "https://example.com/report.pdf" } },
    { type = "audio", mime = "audio/mp3", source = { url = "https://example.com/interview.mp3" } }
]
model = "claude-3-opus-20240229"
temperature = 0.3
max_tokens = 2000
```

### Example 4: Per-Act Model Selection

```toml
[narration]
name = "multi_model_analysis"
description = "Use different models for different strengths"

[toc]
order = ["creative", "analytical", "technical"]

# GPT-4 for creative tasks
[acts.creative]
inputs = [{ type = "text", content = "Brainstorm 10 innovative features" }]
model = "gpt-4"
temperature = 0.9

# Claude for analytical tasks
[acts.analytical]
inputs = [{ type = "text", content = "Analyze the feasibility of each feature" }]
model = "claude-3-opus-20240229"
temperature = 0.3

# Gemini for technical tasks
[acts.technical]
inputs = [{ type = "text", content = "Create a technical implementation plan" }]
model = "gemini-pro"
temperature = 0.2
```

## Best Practices

1. **Context Passing**: Each act sees all previous outputs. Design prompts accordingly.

2. **Temperature Guidelines**:
   - 0.0-0.3: Analytical, factual, deterministic tasks
   - 0.4-0.7: Balanced tasks
   - 0.8-1.0: Creative, exploratory tasks

3. **Model Selection**:
   - Vision tasks: `gemini-pro-vision`, `gpt-4-vision-preview`
   - Audio transcription: `whisper-large-v3`
   - Document analysis: `claude-3-opus-20240229`
   - Creative writing: `gpt-4`, `claude-3-opus-20240229`
   - Fast tasks: `gpt-3.5-turbo`, `claude-3-haiku-20240307`

4. **Mixing Formats**: You can mix simple and structured acts in the same narrative:
   ```toml
   [acts]
   simple_act = "Just text"

   [acts.complex_act]
   inputs = [...]
   model = "gpt-4"
   ```

5. **Act Naming**: Use descriptive act names that indicate their purpose.

## See Also

- `narrations/mint.toml` - Simple text-only example
- `narrations/showcase.toml` - Comprehensive feature demonstration

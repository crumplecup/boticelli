# Anthropic (Claude) Integration Plan

**Date:** 2025-12-01  
**Goal:** Add Anthropic Claude support to Botticelli using `claude-client` crate  
**Strategy:** Follow Gemini/Ollama pattern with feature-gated integration

## Status: PLANNED

- [ ] Phase 1: Dependencies & Feature Flags
- [ ] Phase 2: Error Handling
- [ ] Phase 3: Module Structure
- [ ] Phase 4: Type Conversions
- [ ] Phase 5: Client Implementation  
- [ ] Phase 6: Driver Implementation
- [ ] Phase 7: Configuration
- [ ] Phase 8: Testing
- [ ] Phase 9: Documentation

---

## Overview

**Crate Selection:** `claude-client` from https://crates.io/crates/claude-client

### Why claude-client?

1. **Native Rust** - Not a wrapper around Python/JS
2. **Streaming support** - Built-in SSE handling
3. **Type-safe API** - Strong Rust types
4. **Active maintenance** - Recent updates, good docs
5. **Async/await** - Built on tokio
6. **Focused** - Single-purpose, no bloat

### Models Supported

- **claude-3-5-sonnet-20241022** - Latest, best reasoning
- **claude-3-opus-20240229** - Most capable
- **claude-3-haiku-20240307** - Fastest, cheapest (**free tier**)

### Free Tier

✅ **Free tier available:** claude-3-haiku-20240307 with rate limits for development/testing

---

## Phase 1: Dependencies & Feature Flags

### Update `crates/botticelli_models/Cargo.toml`

```toml
[dependencies]
# Existing
gemini-rust = { version = "1.5", optional = true }
ollama-rs = { version = "0.3", optional = true }

# Add Anthropic
claude-client = { version = "0.4", optional = true }

[features]
default = []
models = []  # Base models feature
gemini = ["dep:gemini-rust", "models"]
ollama = ["dep:ollama-rs", "models"]
anthropic = ["dep:claude-client", "models"]  # New
```

### Update workspace `Cargo.toml`

```toml
[features]
default = ["gemini"]
local = ["ollama", "gemini", "anthropic"]  # Free tiers
all-providers = ["gemini", "ollama", "anthropic"]
```

### Update `crates/botticelli_error/Cargo.toml`

```toml
[dependencies]
claude-client = { version = "0.4", optional = true }

[features]
models = []
anthropic = ["dep:claude-client", "models"]
```

---

## Phase 2: Error Handling

### Add to `crates/botticelli_error/src/models.rs`

```rust
#[cfg(feature = "anthropic")]
use claude_client::ClaudeError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, derive_more::Display)]
#[cfg(feature = "models")]
pub enum ModelsErrorKind {
    // Existing
    #[display("Builder error: {}", _0)]
    Builder(String),
    
    #[cfg(feature = "gemini")]
    #[display("Gemini: {}", _0)]
    #[from(GeminiErrorKind)]
    Gemini(GeminiErrorKind),
    
    #[cfg(feature = "ollama")]
    #[display("Ollama: {}", _0)]
    #[from(OllamaErrorKind)]
    Ollama(OllamaErrorKind),
    
    // Add Anthropic
    #[cfg(feature = "anthropic")]
    #[display("Anthropic: {}", _0)]
    #[from(AnthropicErrorKind)]
    Anthropic(AnthropicErrorKind),
}
```

### Create `AnthropicErrorKind`

```rust
#[cfg(feature = "anthropic")]
#[derive(Debug, Clone, PartialEq, Eq, Hash, derive_more::Display)]
pub enum AnthropicErrorKind {
    #[display("API error: {}", _0)]
    ApiError(String),
    
    #[display("Authentication failed")]
    AuthError,
    
    #[display("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[display("Invalid request: {}", _0)]
    InvalidRequest(String),
    
    #[display("Client error: {}", _0)]
    ClientError(String),
}

#[cfg(feature = "anthropic")]
impl From<ClaudeError> for AnthropicErrorKind {
    fn from(err: ClaudeError) -> Self {
        // Convert claude-client errors to our error kind
        Self::ClientError(err.to_string())
    }
}
```

---

## Phase 3: Module Structure

```
crates/botticelli_models/src/
├── lib.rs              # Crate-level exports
├── gemini/
├── ollama/
└── anthropic/          # New module
    ├── mod.rs          # Private mod declarations, pub use exports
    ├── client.rs       # AnthropicClient wrapper
    ├── conversion.rs   # Botticelli ↔ Claude types
    └── driver.rs       # Driver trait implementation
```

### `crates/botticelli_models/src/anthropic/mod.rs`

```rust
#[cfg(feature = "anthropic")]
mod client;
#[cfg(feature = "anthropic")]
mod conversion;
#[cfg(feature = "anthropic")]
mod driver;

#[cfg(feature = "anthropic")]
pub use client::AnthropicClient;
```

### Update `crates/botticelli_models/src/lib.rs`

```rust
#[cfg(feature = "anthropic")]
mod anthropic;

#[cfg(feature = "anthropic")]
pub use anthropic::AnthropicClient;
```

---

## Phase 4: Type Conversions

### `crates/botticelli_models/src/anthropic/conversion.rs`

```rust
use crate::{Message, Role, Input, Output, GenerateResponse};
use botticelli_error::ModelsResult;
use claude_client::messages::{
    Message as ClaudeMessage,
    MessageRequest,
    Role as ClaudeRole,
    Content,
};

/// Convert Botticelli messages to Claude format.
#[instrument(skip(messages))]
pub fn to_claude_messages(messages: &[Message]) -> (Option<String>, Vec<ClaudeMessage>) {
    let mut system: Option<String> = None;
    let mut claude_messages = Vec::new();
    
    for msg in messages {
        match msg.role() {
            Role::System => {
                // Claude uses separate system field
                system = Some(extract_text(msg.content()));
            }
            Role::User => {
                claude_messages.push(ClaudeMessage {
                    role: ClaudeRole::User,
                    content: vec![Content::Text {
                        text: extract_text(msg.content()),
                    }],
                });
            }
            Role::Model => {
                claude_messages.push(ClaudeMessage {
                    role: ClaudeRole::Assistant,
                    content: vec![Content::Text {
                        text: extract_text(msg.content()),
                    }],
                });
            }
        }
    }
    
    (system, claude_messages)
}

/// Extract text from Botticelli content.
fn extract_text(content: &[Input]) -> String {
    content.iter()
        .filter_map(|input| match input {
            Input::Text(text) => Some(text.as_str()),
            Input::Image(_) => Some("[Image content]"),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Convert Claude response to Botticelli format.
#[instrument(skip(response))]
pub fn from_claude_response(
    response: claude_client::messages::MessageResponse,
) -> ModelsResult<GenerateResponse> {
    debug!(
        input_tokens = response.usage.input_tokens,
        output_tokens = response.usage.output_tokens,
        "Converting Claude response"
    );
    
    let text = response.content
        .into_iter()
        .filter_map(|content| match content {
            Content::Text { text } => Some(text),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    
    GenerateResponse::builder()
        .outputs(vec![Output::Text(text)])
        .build()
        .map_err(Into::into)
}
```

---

## Phase 5: Client Implementation

### `crates/botticelli_models/src/anthropic/client.rs`

```rust
use claude_client::ClaudeClient as ClaudeClientImpl;
use botticelli_error::{ModelsError, ModelsResult};
use tracing::{debug, info, instrument};

/// Anthropic Claude client wrapper.
#[derive(Debug, Clone)]
pub struct AnthropicClient {
    /// Underlying claude-client
    client: ClaudeClientImpl,
    
    /// Model name
    model: String,
    
    /// Max tokens per response
    max_tokens: u32,
}

impl AnthropicClient {
    /// Create new Anthropic client.
    #[instrument(skip(api_key))]
    pub fn new(
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> ModelsResult<Self> {
        let api_key = api_key.into();
        let model = model.into();
        
        info!(model = %model, "Creating Anthropic client");
        
        let client = ClaudeClientImpl::new(api_key)
            .map_err(|e| ModelsError::new(e.into()))?;
        
        Ok(Self {
            client,
            model,
            max_tokens: 4096,  // Default
        })
    }
    
    /// Set max tokens.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }
    
    /// Get model name.
    pub fn model(&self) -> &str {
        &self.model
    }
}
```

---

## Phase 6: Driver Implementation

### `crates/botticelli_models/src/anthropic/driver.rs`

```rust
use async_trait::async_trait;
use crate::{Driver, GenerateRequest, GenerateResponse, StreamChunk};
use botticelli_error::ModelsResult;
use super::{AnthropicClient, conversion};
use tracing::{debug, instrument};
use claude_client::messages::MessageRequest;
use futures::stream::Stream;

#[async_trait]
impl Driver for AnthropicClient {
    #[instrument(skip(self, request), fields(model = %self.model()))]
    async fn generate(
        &self,
        request: GenerateRequest,
    ) -> ModelsResult<GenerateResponse> {
        debug!("Generating with Anthropic");
        
        // Convert messages
        let (system, messages) = conversion::to_claude_messages(request.messages());
        
        // Build Claude request
        let mut claude_request = MessageRequest::new(
            self.model().to_string(),
            messages,
            self.max_tokens,
        );
        
        if let Some(sys) = system {
            claude_request = claude_request.with_system(sys);
        }
        
        // Send request
        let response = self.client
            .create_message(claude_request)
            .await
            .map_err(|e| ModelsError::new(e.into()))?;
        
        // Convert response
        conversion::from_claude_response(response)
    }
    
    async fn stream_generate(
        &self,
        _request: GenerateRequest,
    ) -> ModelsResult<Box<dyn Stream<Item = ModelsResult<StreamChunk>> + Unpin + Send>> {
        // TODO: Implement streaming with claude-client
        todo!("Anthropic streaming not yet implemented")
    }
}
```

---

## Phase 7: Configuration

### Update `botticelli.toml`

```toml
[providers.anthropic]
default_tier = "standard"

[providers.anthropic.tiers.standard]
name = "Standard (Free Tier)"
rpm = 50
tpm = 40000
rpd = null
max_concurrent = 5

[providers.anthropic.tiers.pro]
name = "Pro"
rpm = 4000
tpm = 400000
rpd = null
max_concurrent = 10

[providers.anthropic.models]
"claude-3-5-sonnet-20241022" = { tier = "pro" }
"claude-3-opus-20240229" = { tier = "pro" }
"claude-3-haiku-20240307" = { tier = "standard" }  # Free tier
```

### Environment Variables

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
```

---

## Phase 8: Testing

### Create `crates/botticelli_models/tests/anthropic_test.rs`

```rust
#[cfg(feature = "anthropic")]
use botticelli_models::{AnthropicClient, Driver, GenerateRequest, Message, Role, Input};
#[cfg(feature = "anthropic")]
use botticelli_error::ModelsResult;

#[cfg(feature = "anthropic")]
#[tokio::test]
#[cfg_attr(not(feature = "api"), ignore)]
async fn test_anthropic_basic_generation() -> ModelsResult<()> {
    use std::env;
    
    let api_key = env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY not set");
    
    let client = AnthropicClient::new(
        api_key,
        "claude-3-haiku-20240307",  // Free tier
    )?;
    
    let message = Message::builder()
        .role(Role::User)
        .content(vec![Input::Text("Say hello in 5 words".to_string())])
        .build()
        .map_err(Into::into)?;
    
    let request = GenerateRequest::builder()
        .messages(vec![message])
        .build()
        .map_err(Into::into)?;
    
    let response = client.generate(request).await?;
    
    assert!(!response.outputs().is_empty());
    
    Ok(())
}

#[cfg(feature = "anthropic")]
#[tokio::test]
#[cfg_attr(not(feature = "api"), ignore)]
async fn test_anthropic_conversation() -> ModelsResult<()> {
    // Test multi-turn conversation
    todo!()
}
```

### Run tests

```bash
just test botticelli_models anthropic_test
```

---

## Phase 9: Documentation

### Create `ANTHROPIC.md`

Document:
- Getting API key (free tier signup)
- Supported models
- Rate limits per tier
- Configuration examples
- Narrative usage
- Cost considerations
- Troubleshooting

### Update README.md

Add Anthropic to supported providers list.

---

## Success Criteria

- [x] `claude-client` dependency added with feature gate
- [ ] `AnthropicClient` implements `Driver` trait
- [ ] Feature flag `anthropic` builds without errors
- [ ] Can run narratives with `provider = "anthropic"`
- [ ] Error handling for API errors, auth, rate limits
- [ ] Tracing instrumentation complete
- [ ] Tests pass with `just test-api`
- [ ] Documentation complete
- [ ] Zero clippy warnings
- [ ] `just check-all` passes

---

## Timeline

**Estimated:** 3-5 days (simpler than Ollama due to mature crate)

- **Day 1:** Phases 1-3 (setup, errors, structure)
- **Day 2:** Phases 4-5 (conversions, client)
- **Day 3:** Phase 6 (driver implementation)
- **Day 4:** Phases 7-8 (config, testing)
- **Day 5:** Phase 9 (documentation, review)

---

## Benefits

### vs Manual HTTP (Original Plan)

- ✅ Less boilerplate
- ✅ Built-in error handling
- ✅ Streaming support included
- ✅ Type safety enforced
- ✅ Maintained by community

### Product Benefits

- **Best reasoning** - Claude excels at complex tasks
- **Long context** - 200K tokens
- **Safety** - Built-in safety features
- **Free tier** - claude-3-haiku for development

---

## Future Enhancements

- Streaming support
- Tool use (function calling)
- Vision support (image inputs)
- Prompt caching
- Batch processing

---

_Last Updated: 2025-12-01_  
_Status: Planned_  
_Estimated Effort: 3-5 days_  
_Priority: After Ollama stabilizes_

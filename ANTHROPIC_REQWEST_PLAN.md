# Anthropic Integration Plan (reqwest-based)

**Status**: Planning  
**Approach**: Direct reqwest HTTP client implementation  
**Reason**: Avoids `claude-client` dependency issues and cyclic dependencies

## Overview

Implement Anthropic Claude API support using direct reqwest HTTP calls, following the same pattern as Gemini integration. This approach gives us full control and avoids external dependency issues.

## Feature Gate Strategy

- `models` - Generic LLM functionality (shared by gemini, ollama, anthropic)
- `anthropic` - Anthropic-specific implementation
- `local` - Includes all locally testable providers (gemini, ollama, anthropic, perplexity, groq)

## Implementation Phases

### Phase 1: Core Types ✅ (Complete)

**File**: `crates/botticelli_core/src/anthropic.rs`

```rust
// Private module
mod anthropic;

// Crate-level exports
pub use anthropic::{AnthropicClient, AnthropicConfig};
```

**Types**:
- `AnthropicConfig` - API key, model, endpoint configuration
- `AnthropicClient` - HTTP client wrapper with reqwest
- Request/Response DTOs matching Anthropic API format

**Feature gates**: All code under `#[cfg(feature = "anthropic")]`

### Phase 2: Error Handling ✅ (Complete)

**File**: `crates/botticelli_error/src/models.rs`

Add `AnthropicErrorKind` variant to `ModelsErrorKind`:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, derive_more::Display)]
#[cfg(feature = "anthropic")]
pub enum AnthropicErrorKind {
    #[display("API error: {}", _0)]
    Api(String),
    
    #[display("HTTP error: {}", _0)]
    Http(String),
    
    #[display("Invalid response: {}", _0)]
    InvalidResponse(String),
}
```

Add to `ModelsErrorKind`:

```rust
#[cfg(feature = "anthropic")]
#[from(AnthropicErrorKind)]
Anthropic(AnthropicErrorKind),
```

### Phase 3: Request/Response DTOs

**File**: `crates/botticelli_core/src/anthropic.rs`

All DTOs follow CLAUDE.md guidelines - private fields with getters and builders:

```rust
use derive_getters::Getters;

#[derive(Debug, Clone, Serialize, Getters, derive_builder::Builder)]
#[builder(setter(into, strip_option), default)]
pub struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

impl AnthropicRequest {
    pub fn builder() -> AnthropicRequestBuilder {
        AnthropicRequestBuilder::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Getters, derive_builder::Builder)]
#[builder(setter(into))]
pub struct AnthropicMessage {
    role: String,  // "user" or "assistant"
    content: String,
}

impl AnthropicMessage {
    pub fn builder() -> AnthropicMessageBuilder {
        AnthropicMessageBuilder::default()
    }
}

#[derive(Debug, Clone, Deserialize, Getters)]
pub struct AnthropicResponse {
    id: String,
    model: String,
    content: Vec<ContentBlock>,
    stop_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
}
```

**Key differences from Gemini**:
- Anthropic uses `messages` array with role/content
- System prompt is separate field, not in messages
- Response has structured content blocks
- No streaming in initial implementation

### Phase 4: HTTP Client Implementation

**File**: `crates/botticelli_core/src/anthropic.rs`

```rust
pub struct AnthropicClient {
    client: reqwest::Client,
    config: AnthropicConfig,
}

impl AnthropicClient {
    pub fn new(config: AnthropicConfig) -> ModelsResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| AnthropicErrorKind::Http(e.to_string()))?;
        
        Ok(Self { client, config })
    }
    
    #[instrument(skip(self, request))]
    pub async fn generate(
        &self,
        request: AnthropicRequest,
    ) -> ModelsResult<AnthropicResponse> {
        let url = format!("{}/v1/messages", self.config.endpoint());
        
        let response = self.client
            .post(&url)
            .header("x-api-key", self.config.api_key())
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await
            .map_err(|e| AnthropicErrorKind::Http(e.to_string()))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AnthropicErrorKind::Api(
                format!("HTTP {}: {}", status, body)
            ).into());
        }
        
        response.json()
            .await
            .map_err(|e| AnthropicErrorKind::InvalidResponse(e.to_string()).into())
    }
}
```

### Phase 5: Type Conversions

**File**: `crates/botticelli_core/src/anthropic.rs`

Convert between Botticelli types and Anthropic types:

```rust
impl From<&GenerateRequest> for AnthropicRequest {
    fn from(req: &GenerateRequest) -> Self {
        // Extract system prompt from messages
        let (system, messages) = extract_system_and_messages(req.messages());
        
        Self {
            model: req.model().to_string(),
            messages,
            max_tokens: req.max_tokens().unwrap_or(1024),
            temperature: req.temperature(),
            system,
        }
    }
}

impl TryFrom<AnthropicResponse> for GenerateResponse {
    type Error = ModelsError;
    
    fn try_from(resp: AnthropicResponse) -> Result<Self, Self::Error> {
        let text = resp.content
            .into_iter()
            .filter_map(|block| match block {
                ContentBlock::Text { text } => Some(text),
            })
            .collect::<Vec<_>>()
            .join("\n");
        
        GenerateResponse::builder()
            .outputs(vec![Output::Text(text)])
            .build()
            .map_err(Into::into)
    }
}
```

### Phase 6: BotticelliDriver Implementation

**File**: `crates/botticelli_core/src/anthropic.rs`

```rust
#[async_trait]
impl BotticelliDriver for AnthropicClient {
    #[instrument(skip(self))]
    async fn generate(&self, request: &GenerateRequest) -> ModelsResult<GenerateResponse> {
        let anthropic_req = AnthropicRequest::from(request);
        let anthropic_resp = self.generate(anthropic_req).await?;
        anthropic_resp.try_into()
    }
    
    #[instrument(skip(self))]
    async fn generate_stream(
        &self,
        _request: &GenerateRequest,
    ) -> ModelsResult<BoxStream<'static, ModelsResult<StreamChunk>>> {
        // Not implemented initially
        Err(AnthropicErrorKind::Api("Streaming not yet implemented".to_string()).into())
    }
}
```

### Phase 7: Configuration Integration

**File**: `crates/botticelli_core/src/config.rs`

Add Anthropic variant to `BackendConfig`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum BackendConfig {
    #[cfg(feature = "gemini")]
    Gemini(GeminiConfig),
    
    #[cfg(feature = "ollama")]
    Ollama(OllamaConfig),
    
    #[cfg(feature = "anthropic")]
    Anthropic(AnthropicConfig),
}
```

Update `create_driver()`:

```rust
#[cfg(feature = "anthropic")]
BackendConfig::Anthropic(config) => {
    Box::new(AnthropicClient::new(config.clone())?)
}
```

### Phase 8: Testing

**File**: `tests/anthropic_test.rs`

```rust
#[tokio::test]
#[cfg_attr(not(feature = "api"), ignore)]
async fn test_anthropic_generate() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")?;
    
    let config = AnthropicConfig::builder()
        .api_key(api_key)
        .model("claude-3-haiku-20240307")
        .build()?;
    
    let client = AnthropicClient::new(config)?;
    
    let request = GenerateRequest::builder()
        .messages(vec![
            Message::builder()
                .role(Role::User)
                .content(vec![Input::Text("Say 'Hello, World!' in exactly 3 words.".to_string())])
                .build()?
        ])
        .max_tokens(10)
        .build()?;
    
    let response = client.generate(&request).await?;
    assert!(!response.outputs().is_empty());
    
    Ok(())
}
```

### Phase 9: Documentation

**File**: `ANTHROPIC.md`

Document:
- Configuration requirements (API key)
- Supported models
- Free tier details (Claude Haiku)
- Usage examples
- Limitations (no streaming initially)

### Phase 10: Integration Testing

Run full test suite:
```bash
just test-api  # With ANTHROPIC_API_KEY set
just check-features
just check-all
```

## API Details

**Endpoint**: `https://api.anthropic.com/v1/messages`

**Headers**:
- `x-api-key`: API key
- `anthropic-version`: "2023-06-01"
- `content-type`: "application/json"

**Request format**:
```json
{
  "model": "claude-3-haiku-20240307",
  "max_tokens": 1024,
  "messages": [
    {"role": "user", "content": "Hello!"}
  ]
}
```

**Response format**:
```json
{
  "id": "msg_...",
  "type": "message",
  "role": "assistant",
  "content": [
    {"type": "text", "text": "Hello! How can I help?"}
  ],
  "model": "claude-3-haiku-20240307",
  "stop_reason": "end_turn"
}
```

## Free Tier Models

- `claude-3-haiku-20240307` - Fastest, most affordable
- `claude-3-sonnet-20240229` - Balanced
- `claude-3-opus-20240229` - Most capable

All have generous free tiers for testing.

## Benefits of reqwest Approach

1. **No external dependencies** - Full control over HTTP layer
2. **No cyclic dependencies** - Clean crate structure
3. **Simple to maintain** - Direct API calls
4. **Flexible** - Easy to add streaming later
5. **Consistent** - Matches Gemini pattern

## Testing Strategy

1. **Unit tests** - Type conversions, config validation
2. **Integration tests** - Real API calls (gated with `api` feature)
3. **Minimal token usage** - Short prompts, low `max_tokens`
4. **Error handling** - Test invalid keys, rate limits

## Next Steps

After Anthropic is stable:
- Add streaming support
- Implement function calling
- Add vision capabilities (image inputs)
- Consider caching strategies

# Gemini Streaming Implementation Plan

## Overview

This document outlines the plan to implement streaming support for Gemini models with bidirectional streaming capabilities (`bidiGenerateContent`) in the Boticelli library.

**CRITICAL FINDING**: The model `gemini-2.0-flash-live` **does not exist**. After querying the Gemini API, we found:
- ✅ 50 models available in v1beta API
- ✅ 2 models support `bidiGenerateContent` (bidirectional streaming):
  - `gemini-2.0-flash-exp`
  - `gemini-2.0-flash-exp-image-generation`
- ❌ **NO models with "live" in their name exist**

**Updated Goal**: Implement support for bidirectional streaming models (`gemini-2.0-flash-exp`) to access experimental features and potentially better rate limits.

**Date**: 2025-01-17  
**Status**: Investigation Phase - Step 2 Complete ✅  
**Priority**: MEDIUM (enables experimental features)  
**Complexity**: HIGH (requires WebSocket/bidirectional streaming, not simple HTTP streaming)

---

## API Investigation Results (2025-01-17)

### Available Models Query

Using the Gemini v1beta API endpoint `GET /v1beta/models`, we discovered:

**Total Models**: 50

**Models Supporting `bidiGenerateContent` (Bidirectional Streaming)**:
1. `gemini-2.0-flash-exp` - Gemini 2.0 Flash Experimental
   - Methods: `generateContent`, `countTokens`, `bidiGenerateContent`
2. `gemini-2.0-flash-exp-image-generation` - Gemini 2.0 Flash (Image Generation) Experimental
   - Methods: `generateContent`, `countTokens`, `bidiGenerateContent`

**Key Finding**: The test assumption that `gemini-2.0-flash-live` exists was **INCORRECT**. No models with "live" in the name exist in the v1beta API.

### Test Behavior Explained

When running `test_streaming_with_live_model`:
- ✅ Test **IS** hitting the API (confirmed by 404 response from Google servers)
- ✅ API key authentication works
- ❌ Model lookup fails with 404: "models/gemini-2.0-flash-live is not found"
- ❌ No token usage recorded (request rejected before processing)

### What is `bidiGenerateContent`?

Based on the API discovery:
- **NOT** simple HTTP streaming (like SSE - Server-Sent Events)
- **IS** bidirectional streaming - requires WebSocket or gRPC
- Enables real-time, interactive conversations
- Allows client to send inputs while receiving outputs
- Used for experimental features (voice, multimodal interactions)

---

## Business Case

### Original Assumption - DEBUNKED ❌

**Original Claim**: Live models have higher rate limits on free tier
**Reality**: No "live" models exist; experimental models may have different limits

### Updated Value Proposition

**Why Support Bidirectional Streaming?**
1. Access to experimental models (`gemini-2.0-flash-exp`)
2. Future-proofing for interactive features
3. Potential for better rate limits on experimental models
4. Enable voice/multimodal interactions (future consideration)

**Trade-off Analysis**:
- ✅ Enables cutting-edge features
- ✅ Access to experimental models
- ❌ Higher complexity (WebSocket/gRPC vs simple HTTP)
- ❌ May have unstable API surface
- ❌ Experimental models may be removed/changed

**Recommendation**: **DEFER** until we validate that experimental models actually have better rate limits or provide features we need.

---

## Current State

### What Works
- ✅ All non-streaming Gemini models (2.0-flash, 2.5-flash, 2.5-pro, etc.)
- ✅ Rate limiting per model
- ✅ Model pooling with lazy initialization
- ✅ Multi-model narratives
- ✅ Vision support (base64 images)
- ✅ Async operations with tokio

### What Doesn't Work
- ❌ Bidirectional streaming models (`gemini-2.0-flash-exp`)
- ❌ WebSocket/gRPC-based real-time interactions
- ⚠️ **Note**: "Live models" don't exist - original assumption was incorrect

### Current Architecture

```
┌──────────────────┐
│ GeminiClient     │
│ (BoticelliDriver)│
└────────┬─────────┘
         │
         ├─> ModelPool (HashMap<model_name, ClientWrapper>)
         │   └─> RateLimiter -> gemini_rust::Gemini
         │
         └─> generate() -> GenerateResponse (blocking, returns complete response)
```

---

## Problem Analysis

### What Are Bidirectional Streaming Models?

**CORRECTION**: Original document assumed "live" models existed. After API investigation:

**Reality**:
- ❌ No "gemini-2.0-flash-live" model exists
- ✅ 2 experimental models support `bidiGenerateContent`
- ✅ These require bidirectional streaming (WebSocket/gRPC, not simple HTTP SSE)

**Bidirectional Streaming Models** (`gemini-2.0-flash-exp`):
- Real-time, interactive conversations
- Voice/audio interactions
- Client can send while receiving responses
- More complex than unidirectional streaming

### Why Original Assumption Was Wrong

**Original Claim**: "Live models have better rate limits"
**Source**: Likely confusion with:
1. Experimental models (which may have different limits)
2. Gemini Live API (a separate product for voice/video interactions)
3. Streaming vs batch processing (streaming feels faster but same limits)

**What We Actually Need**:
- If goal is **better rate limits**: Check if experimental models have this
- If goal is **streaming responses**: Standard models support SSE streaming
- If goal is **interactive features**: Need bidirectional streaming (complex)

### Current Limitations

1. **gemini_rust Library**: Supports SSE streaming ✅, unclear on bidiGenerateContent ❓
2. **BoticelliDriver Trait**: Returns complete `GenerateResponse`, not streaming
3. **Narrative Executor**: Expects complete responses, not incremental
4. **Rate Limiting**: Designed for request/response, needs streaming adaptation

---

## Investigation Phase

### Step 1: Assess gemini_rust Library ✅ COMPLETE

**Investigation Date**: 2025-01-17

#### Findings: gemini_rust DOES Support Streaming! 🎉

**Version**: gemini-rust 1.5.1 (used by Boticelli)

**Location**: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gemini-rust-1.5.1`

#### Streaming API Summary

**Method**: `generate_content_stream()`

```rust
pub(crate) async fn generate_content_stream(
    &self,
    request: GenerateContentRequest,
) -> Result<impl TryStreamExt<Ok = GenerationResponse, Error = Error> + Send + use<>, Error>
```

**Key Points**:
1. ✅ **Protocol**: Server-Sent Events (SSE) via `alt=sse` query parameter
2. ✅ **Stream Type**: Uses `futures::Stream` with `TryStreamExt`
3. ✅ **Library**: `eventsource_stream` crate for SSE parsing
4. ✅ **Response Format**: JSON chunks as `GenerationResponse` structs
5. ✅ **Builder API**: `execute_stream()` on generation builder

#### Code Example from gemini_rust

```rust
// From examples/basic_streaming.rs
let mut stream = client
    .generate_content()
    .with_user_message("Tell me a story")
    .execute_stream()
    .await?;

// Process chunks as they arrive
while let Some(chunk) = stream.try_next().await? {
    let text = chunk.text();
    println!("{}", text);
}
```

#### API Structure

**Dependencies**:
- `futures` crate: `Stream`, `StreamExt`, `TryStreamExt` traits
- `eventsource_stream`: SSE event parsing
- `async_stream` macro: For creating custom streams

**GenerationResponse** (per chunk):
```rust
pub struct GenerationResponse {
    pub candidates: Vec<Candidate>,
    pub prompt_feedback: Option<PromptFeedback>,
    pub usage_metadata: Option<UsageMetadata>,
    pub model_version: Option<String>,
}
```

Each chunk contains:
- Text content (via `chunk.text()` helper)
- Finish reason (when complete)
- Usage metadata (tokens consumed)

#### URL Pattern

```
POST https://generativelanguage.googleapis.com/v1beta/models/{model}:streamGenerateContent?alt=sse
```

The `streamGenerateContent` endpoint with `alt=sse` enables streaming.

#### Examples Available

gemini_rust includes two streaming examples:
1. **`basic_streaming.rs`**: Simple streaming with real-time output
2. **`streaming.rs`**: More advanced streaming features

Both demonstrate:
- Creating streaming requests
- Processing chunks incrementally
- Handling completion
- Error handling

#### Key Insights

1. **No "Live" Model Needed**: Regular models (gemini-2.0-flash, gemini-2.5-flash) already support streaming
2. **SSE Protocol**: Uses standard Server-Sent Events, not WebSocket
3. **Unidirectional**: Client sends request once, server streams response (not bidirectional)
4. **Same Authentication**: Uses same API key as non-streaming requests
5. **Incremental Text**: Each chunk contains partial text that should be concatenated

#### Questions Resolved

| Question | Answer |
|----------|--------|
| Does gemini_rust support streaming? | ✅ YES |
| What format? | `futures::Stream` with `TryStreamExt` |
| Protocol? | Server-Sent Events (SSE) |
| Bidirectional? | No - unidirectional (server → client) |
| API surface? | `execute_stream()` on builder, returns stream of `GenerationResponse` |

#### Implications for Boticelli

**Good News**:
- gemini_rust already has robust streaming support
- We don't need to fork or implement HTTP directly
- API is clean and idiomatic (futures-based streams)
- Examples exist for reference

**What We Need to Do**:
1. Wrap `generate_content_stream()` in our `GeminiClient`
2. Convert `GenerationResponse` stream to `StreamChunk` stream
3. Add live model detection (models ending in `-live`)
4. Handle rate limiting for streaming requests
5. Add tests and documentation
6. Test specifically with `gemini-2.0-flash-live` to confirm rate limit benefits

**Complexity Reduced**: Since gemini_rust handles the hard parts (SSE parsing, connection management), our implementation is mostly adapting the stream format.

### Step 2: Research Gemini Live API ✅ CLARIFIED

**Critical Context from User**: 

> "The purpose of streaming in the first place is to access the live models. When on the free tier, the live models have the most permissible use limits. We want to add support for these models because we can use them more frequently."

#### Key Insights

**Primary Goal**: Access to better rate limits on free tier

**Model Naming**:
- Standard: `gemini-2.0-flash`, `gemini-2.5-flash`
- Live: `gemini-2.0-flash-live`, `gemini-2.5-flash-live` (or similar suffix)

**Requirements**:
- Live models **require streaming** - they don't work with regular generate API
- Streaming is not just a performance optimization, it's the **only way** to use live models
- Live models offer better RPM/RPD limits on free tier

#### What We Know

1. ✅ gemini_rust supports streaming via SSE
2. ✅ Standard models support streaming (tested with examples)
3. ⚠️ **Need to verify**: Do live models use the same SSE endpoint?
4. ⚠️ **Need to verify**: Do live models require different authentication?
5. ⚠️ **Need to verify**: Actual rate limit differences

#### Questions for Testing

**When implementing MVP, test with live models**:

```rust
// Test 1: Does live model work with same streaming API?
let request = GenerateRequest {
    model: Some("gemini-2.0-flash-live".to_string()),
    // ... rest of request
};
let stream = client.generate_stream(&request).await?;

// Test 2: Compare rate limits
// - Make 100 requests to gemini-2.0-flash (standard)
// - Make 100 requests to gemini-2.0-flash-live (live)
// - Measure: Which one hits rate limits first?
```

**Expected Outcome**: 
- Live models should work with existing streaming implementation
- Live models should allow more requests before rate limiting
- If they don't work, we'll get specific error messages to guide next steps

#### Implementation Impact

**Model Detection**:
```rust
impl GeminiClient {
    fn is_live_model(model_name: &str) -> bool {
        model_name.contains("-live")
    }
    
    fn parse_model_name(name: &str) -> (Model, bool) {
        let is_live = Self::is_live_model(name);
        let model = match name {
            "gemini-2.0-flash-live" => Model::Custom("models/gemini-2.0-flash-live"),
            "gemini-2.5-flash-live" => Model::Custom("models/gemini-2.5-flash-live"),
            // ... other models
            _ if name.contains("-live") => Model::Custom(format!("models/{}", name)),
            // ... standard models
        };
        (model, is_live)
    }
}
```

**Rate Limiter Configuration**:
```rust
// In tier config, potentially different limits for live models
pub struct TierConfig {
    pub name: String,
    pub rpm: Option<u32>,  // Live models might have higher values
    pub rpd: Option<u32>,  // Live models might have higher values
    // ...
}
```

#### Documentation Needed

After implementation, update GEMINI.md with:

```markdown
## Live Models and Rate Limits

Live models (e.g., `gemini-2.0-flash-live`) offer **higher rate limits** on the free tier, making them ideal for development and testing.

### Using Live Models

Live models require streaming:

\`\`\`rust
let client = GeminiClient::new()?;

let request = GenerateRequest {
    model: Some("gemini-2.0-flash-live".to_string()),
    // ... your request
};

// MUST use generate_stream() - live models don't support generate()
let mut stream = client.generate_stream(&request).await?;

while let Some(chunk) = stream.try_next().await? {
    print!("{}", chunk.text);
    if chunk.finished { break; }
}
\`\`\`

### Rate Limit Benefits

Free tier comparison (example - verify actual values):

| Model | RPM | RPD | Notes |
|-------|-----|-----|-------|
| gemini-2.0-flash | 15 | 1,500 | Standard |
| gemini-2.0-flash-live | **30** | **3,000** | Better for dev |

Use live models for:
- Development and testing
- Iterative prompt engineering  
- High-frequency API calls
- CI/CD test suites

Use standard models for:
- Production deployments
- When streaming not needed
- Batch processing
\`\`\`
```

#### Status

- ✅ Understand business case (better rate limits)
- ✅ Understand requirement (must use streaming)
- ⚠️ Need to verify live model specifics during implementation
- ⚠️ Need to document actual rate limit differences

**Action Item**: First MVP test should be with `gemini-2.0-flash-live` to confirm it works and measure rate limit benefits.

---

## Implementation Strategy

### Phase 1: Extend BoticelliDriver Trait (Foundation)

**Goal**: Add streaming capability alongside existing blocking API

#### Option A: New Trait Method (Recommended)

```rust
#[async_trait]
pub trait BoticelliDriver: Send + Sync {
    // Existing method (unchanged)
    async fn generate(&self, request: &GenerateRequest) -> BoticelliResult<GenerateResponse>;
    
    // New streaming method
    async fn generate_stream(
        &self,
        request: &GenerateRequest,
    ) -> BoticelliResult<Pin<Box<dyn Stream<Item = BoticelliResult<StreamChunk>> + Send>>>;
    
    // Optional: Check if model supports streaming
    fn supports_streaming(&self, model: &str) -> bool {
        false  // Default: no streaming
    }
}

/// Incremental response chunk from streaming API
#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub text: String,
    pub finished: bool,
    pub metadata: Option<ChunkMetadata>,
}

#[derive(Debug, Clone)]
pub struct ChunkMetadata {
    pub tokens_generated: Option<u32>,
    pub finish_reason: Option<String>,
}
```

**Pros**:
- Backward compatible (existing code unchanged)
- Clear separation of streaming vs. blocking
- Opt-in for drivers that support streaming

**Cons**:
- Drivers must implement both methods (or provide default)
- Consumers must handle two code paths

#### Option B: Callback-Based (Alternative)

```rust
#[async_trait]
pub trait BoticelliDriver: Send + Sync {
    async fn generate(&self, request: &GenerateRequest) -> BoticelliResult<GenerateResponse>;
    
    async fn generate_with_callback<F>(
        &self,
        request: &GenerateRequest,
        on_chunk: F,
    ) -> BoticelliResult<GenerateResponse>
    where
        F: Fn(StreamChunk) + Send + Sync;
}
```

**Pros**:
- Still returns complete response at end
- Allows progressive updates during generation

**Cons**:
- Less flexible than Stream
- Harder to compose with other async code

**Recommendation**: Use **Option A** (Stream-based) for maximum flexibility.

---

### Phase 2: Implement Streaming in GeminiClient

#### 2.1: Add Streaming Support to ModelClientWrapper

```rust
struct ModelClientWrapper {
    client: Gemini,
    rate_limiter: Option<RateLimiter>,
    supports_streaming: bool,  // New field
}

impl ModelClientWrapper {
    async fn generate_stream(
        &self,
        request: &GenerateRequest,
    ) -> BoticelliResult<impl Stream<Item = BoticelliResult<StreamChunk>>> {
        // Apply rate limiting
        if let Some(limiter) = &self.rate_limiter {
            limiter.acquire().await?;
        }
        
        // Call gemini_rust streaming API
        let stream = self.client.generate_content_stream(/* ... */)?;
        
        // Transform gemini_rust stream into BoticelliResult<StreamChunk>
        Ok(stream.map(|result| {
            result
                .map_err(|e| GeminiError::new(GeminiErrorKind::ApiRequest(e.to_string())).into())
                .and_then(|chunk| convert_gemini_chunk_to_boticelli(chunk))
        }))
    }
}
```

#### 2.2: Detect Streaming-Capable Models

```rust
impl GeminiClient {
    fn model_supports_streaming(model_name: &str) -> bool {
        model_name.contains("-live") || model_name.contains("-streaming")
    }
    
    fn parse_model_name(name: &str) -> (Model, bool) {
        let supports_streaming = Self::model_supports_streaming(name);
        let model = match name {
            "gemini-2.0-flash-live" => Model::Custom("models/gemini-2.0-flash-live"),
            // ... other models
        };
        (model, supports_streaming)
    }
}
```

#### 2.3: Implement BoticelliDriver::generate_stream

```rust
#[async_trait]
impl BoticelliDriver for GeminiClient {
    async fn generate_stream(
        &self,
        request: &GenerateRequest,
    ) -> BoticelliResult<Pin<Box<dyn Stream<Item = BoticelliResult<StreamChunk>> + Send>>> {
        let model_name = self.resolve_model_name(request);
        
        // Check if model supports streaming
        if !Self::model_supports_streaming(&model_name) {
            return Err(GeminiError::new(GeminiErrorKind::StreamingNotSupported(model_name)).into());
        }
        
        let wrapper = self.get_or_create_client(&model_name).await?;
        let stream = wrapper.generate_stream(request).await?;
        
        Ok(Box::pin(stream))
    }
    
    fn supports_streaming(&self, model: &str) -> bool {
        Self::model_supports_streaming(model)
    }
}
```

---

## Concrete Implementation Plan (Based on Findings)

### Quick Win: Minimal Streaming Implementation

Based on our investigation, here's a **minimal viable implementation** that could be done in 1-2 days:

#### Step 1: Add StreamChunk Type (5 minutes)

```rust
// In src/models/mod.rs or src/driver.rs

/// Incremental response chunk from streaming API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    /// Text content in this chunk
    pub text: String,
    
    /// Whether this is the final chunk
    pub finished: bool,
    
    /// Optional metadata about this chunk
    pub metadata: Option<ChunkMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    /// Tokens generated so far
    pub tokens_generated: Option<u32>,
    
    /// Why generation stopped (if finished)
    pub finish_reason: Option<String>,
}
```

#### Step 2: Extend BoticelliDriver Trait (10 minutes)

```rust
// In src/driver.rs

use futures::stream::Stream;
use std::pin::Pin;

#[async_trait]
pub trait BoticelliDriver: Send + Sync {
    // Existing method (unchanged)
    async fn generate(&self, request: &GenerateRequest) -> BoticelliResult<GenerateResponse>;
    
    // New streaming method with default implementation
    async fn generate_stream(
        &self,
        request: &GenerateRequest,
    ) -> BoticelliResult<Pin<Box<dyn Stream<Item = BoticelliResult<StreamChunk>> + Send>>> {
        // Default: not supported
        Err(BackendError::new("Streaming not supported by this driver").into())
    }
    
    // Check if driver supports streaming
    fn supports_streaming(&self) -> bool {
        false  // Default: no streaming
    }
}
```

#### Step 3: Implement in GeminiClient (30 minutes)

```rust
// In src/models/gemini.rs

use futures::{Stream, StreamExt, TryStreamExt};
use std::pin::Pin;

#[async_trait]
impl BoticelliDriver for GeminiClient {
    // ... existing generate() implementation unchanged ...
    
    async fn generate_stream(
        &self,
        request: &GenerateRequest,
    ) -> BoticelliResult<Pin<Box<dyn Stream<Item = BoticelliResult<StreamChunk>> + Send>>> {
        let model_name = self.resolve_model_name(request);
        let wrapper = self.get_or_create_client(&model_name).await?;
        
        // Apply rate limiting (count as single request)
        if let Some(limiter) = &wrapper.rate_limiter {
            limiter.acquire().await?;
        }
        
        // Build gemini_rust request (reuse existing conversion)
        let gemini_request = self.build_gemini_request(request)?;
        
        // Call gemini_rust streaming API
        let gemini_stream = wrapper.client
            .generate_content_stream(gemini_request)
            .await
            .map_err(|e| GeminiError::new(GeminiErrorKind::ApiRequest(e.to_string())))?;
        
        // Transform gemini GenerationResponse stream to our StreamChunk stream
        let chunk_stream = gemini_stream
            .map(|result| {
                result
                    .map_err(|e| GeminiError::new(GeminiErrorKind::ApiRequest(e.to_string())).into())
                    .and_then(|response| convert_to_stream_chunk(response))
            });
        
        Ok(Box::pin(chunk_stream))
    }
    
    fn supports_streaming(&self) -> bool {
        true  // Gemini supports streaming
    }
}

/// Convert gemini_rust GenerationResponse to our StreamChunk
fn convert_to_stream_chunk(response: gemini_rust::GenerationResponse) -> BoticelliResult<StreamChunk> {
    let text = response.text();  // gemini_rust helper method
    
    let finished = response
        .candidates
        .first()
        .and_then(|c| c.finish_reason.as_ref())
        .is_some();
    
    let metadata = response.usage_metadata.map(|usage| ChunkMetadata {
        tokens_generated: Some(usage.total_token_count),
        finish_reason: response.candidates
            .first()
            .and_then(|c| c.finish_reason.as_ref())
            .map(|r| format!("{:?}", r)),
    });
    
    Ok(StreamChunk {
        text,
        finished,
        metadata,
    })
}
```

#### Step 4: Add Basic Test (20 minutes)

```rust
// In tests/gemini_streaming_test.rs

#![cfg(feature = "gemini")]

use boticelli::{BoticelliDriver, GeminiClient, GenerateRequest, Message, Role, Input};
use futures::StreamExt;

#[tokio::test]
async fn test_basic_streaming() {
    let _ = dotenvy::dotenv();
    
    let client = GeminiClient::new().expect("Failed to create client");
    
    assert!(client.supports_streaming(), "Gemini should support streaming");
    
    let request = GenerateRequest {
        messages: vec![Message {
            role: Role::User,
            content: vec![Input::Text("Count from 1 to 5".to_string())],
        }],
        model: Some("gemini-2.0-flash".to_string()),
        ..Default::default()
    };
    
    let mut stream = client.generate_stream(&request).await.expect("Stream creation failed");
    
    let mut chunks = Vec::new();
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.expect("Chunk error");
        chunks.push(chunk.text.clone());
        
        if chunk.finished {
            break;
        }
    }
    
    assert!(!chunks.is_empty(), "Should receive at least one chunk");
    
    let full_text = chunks.join("");
    println!("Streaming result: {}", full_text);
    
    // Should contain numbers
    assert!(full_text.contains('1') || full_text.contains("one"), 
        "Response should contain counting");
}

#[tokio::test]
async fn test_streaming_matches_non_streaming() {
    let _ = dotenvy::dotenv();
    
    let client = GeminiClient::new().expect("Failed to create client");
    
    let request = GenerateRequest {
        messages: vec![Message {
            role: Role::User,
            content: vec![Input::Text("Say 'Hello World' exactly".to_string())],
        }],
        model: Some("gemini-2.0-flash".to_string()),
        ..Default::default()
    };
    
    // Get streaming response
    let mut stream = client.generate_stream(&request).await.expect("Stream failed");
    let mut streaming_text = String::new();
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.expect("Chunk error");
        streaming_text.push_str(&chunk.text);
        if chunk.finished {
            break;
        }
    }
    
    // Get non-streaming response  
    let response = client.generate(&request).await.expect("Generate failed");
    let non_streaming_text = response.outputs.iter()
        .filter_map(|o| match o {
            boticelli::Output::Text(t) => Some(t.clone()),
            _ => None,
        })
        .collect::<String>();
    
    // They should be similar (may have minor formatting differences)
    assert!(!streaming_text.is_empty());
    assert!(!non_streaming_text.is_empty());
    
    println!("Streaming: {}", streaming_text);
    println!("Non-streaming: {}", non_streaming_text);
}
```

#### Step 5: Update Dependencies (if needed)

Check if we need to add to `Cargo.toml`:

```toml
[dependencies]
# These might already be present for gemini_rust
futures = "0.3"
```

#### Step 6: Documentation (15 minutes)

Add to GEMINI.md:

```markdown
## Streaming Support

Gemini models support streaming responses for real-time content generation:

\`\`\`rust
use boticelli::{BoticelliDriver, GeminiClient, GenerateRequest};
use futures::StreamExt;

let client = GeminiClient::new()?;

let request = GenerateRequest {
    // ... your request
};

let mut stream = client.generate_stream(&request).await?;

while let Some(chunk_result) = stream.next().await {
    let chunk = chunk_result?;
    print!("{}", chunk.text);
    
    if chunk.finished {
        break;
    }
}
\`\`\`

### When to Use Streaming

- **Real-time UI updates**: Show content as it's generated
- **Long responses**: Display progress during generation
- **Interactive applications**: Provide faster perceived responsiveness

### Limitations

- Rate limiting counts the entire stream as one request
- Cannot partially cancel a stream (yet)
- Narratives don't support streaming (use `generate()` instead)
```

### Total Time Estimate: 2-3 hours for MVP

This gets you:
- ✅ Working streaming support in GeminiClient
- ✅ Backward compatible (no breaking changes)
- ✅ Basic tests
- ✅ Documentation

### What's NOT Included in MVP

- Streaming in narrative executor
- Advanced rate limiting (per-chunk)
- Cancellation support
- CLI streaming output
- Multiple simultaneous streams

These can be added incrementally later.

---

### Phase 3: Update Narrative Executor (Optional)

**Decision Point**: Do we need streaming in narrative execution?

#### Scenario A: No Narrative Streaming (Simpler)

- Narratives continue using blocking `generate()`
- Streaming is opt-in for custom code
- No changes needed to narrative executor

#### Scenario B: Progressive Narrative Execution (Advanced)

- Allow narratives to show progress as content generates
- Useful for long generations
- Requires executor changes

```rust
// Example: Stream-aware executor
impl<D: BoticelliDriver> NarrativeExecutor<D> {
    pub async fn execute_with_progress<N, F>(
        &self,
        narrative: &N,
        on_progress: F,
    ) -> BoticelliResult<NarrativeExecution>
    where
        N: NarrativeProvider,
        F: Fn(&str, &StreamChunk) + Send + Sync,
    {
        // For each act, if model supports streaming, use generate_stream
        // Otherwise fall back to generate()
        // ...
    }
}
```

**Recommendation**: Start with **Scenario A**. Add Scenario B later if needed.

---

### Phase 4: Testing Strategy

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    
    #[tokio::test]
    async fn test_streaming_basic() {
        let client = GeminiClient::new().unwrap();
        
        let request = GenerateRequest {
            messages: vec![Message {
                role: Role::User,
                content: vec![Input::Text("Count to 10".to_string())],
            }],
            model: Some("gemini-2.0-flash-live".to_string()),
            ..Default::default()
        };
        
        let mut stream = client.generate_stream(&request).await.unwrap();
        let mut chunks = Vec::new();
        
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.unwrap();
            chunks.push(chunk.text.clone());
            
            if chunk.finished {
                break;
            }
        }
        
        assert!(!chunks.is_empty());
        let full_text = chunks.join("");
        assert!(full_text.contains("1"));
    }
    
    #[tokio::test]
    async fn test_non_streaming_model_returns_error() {
        let client = GeminiClient::new().unwrap();
        
        let request = GenerateRequest {
            model: Some("gemini-2.0-flash".to_string()),  // Non-streaming
            ..Default::default()
        };
        
        let result = client.generate_stream(&request).await;
        assert!(result.is_err());
    }
}
```

#### Integration Tests

```rust
#[tokio::test]
async fn test_streaming_rate_limiting() {
    // Verify rate limiter works with streaming
}

#[tokio::test]
async fn test_streaming_error_handling() {
    // Verify graceful handling of mid-stream errors
}

#[tokio::test]
async fn test_streaming_cancellation() {
    // Verify stream cleanup on early termination
}
```

---

## Error Handling

### New Error Types

```rust
pub enum GeminiErrorKind {
    // Existing variants...
    
    /// Model doesn't support streaming
    StreamingNotSupported(String),
    
    /// Stream was interrupted
    StreamInterrupted(String),
    
    /// Stream exceeded timeout
    StreamTimeout,
}
```

### Error Scenarios

1. **Model doesn't support streaming**: Return error immediately
2. **Connection drops mid-stream**: Wrap in StreamInterrupted error
3. **Rate limit hit during stream**: Pause stream, resume when allowed
4. **Client cancels stream**: Clean up resources properly

---

## Rate Limiting Considerations

### Challenge

Current rate limiter is designed for discrete requests. Streaming sessions may:
- Last minutes (not milliseconds)
- Generate many tokens incrementally
- Need different RPM/TPM accounting

### Solutions

#### Option 1: Count Stream as Single Request

```rust
// Acquire rate limit token at start of stream
limiter.acquire().await?;

// Stream proceeds without further checks
// Tokens counted at end of stream
```

**Pros**: Simple  
**Cons**: Doesn't account for long-running streams

#### Option 2: Periodic Rate Limit Checks

```rust
// Check rate limit every N chunks or N seconds
let mut chunk_count = 0;
while let Some(chunk) = stream.next().await {
    chunk_count += 1;
    
    if chunk_count % 100 == 0 {
        limiter.check_and_wait().await?;
    }
    
    yield chunk;
}
```

**Pros**: Better accounting  
**Cons**: More complex, may interrupt flow

**Recommendation**: Start with **Option 1**, monitor usage, add Option 2 if needed.

---

## Migration Path

### Phase 1: Foundation (Week 1-2)
- [ ] Investigate gemini_rust streaming capabilities
- [ ] Research Gemini Live API protocol
- [ ] Design BoticelliDriver streaming extension
- [ ] Create proof-of-concept with direct HTTP if needed

### Phase 2: Core Implementation (Week 3-4)
- [ ] Extend BoticelliDriver trait
- [ ] Implement streaming in GeminiClient
- [ ] Add model detection for streaming support
- [ ] Write unit tests

### Phase 3: Integration (Week 5)
- [ ] Test with real Gemini Live models
- [ ] Add integration tests
- [ ] Document usage in GEMINI.md
- [ ] Add examples

### Phase 4: Polish (Week 6)
- [ ] Rate limiting refinements
- [ ] Error handling improvements
- [ ] Performance testing
- [ ] User guide for streaming

---

## Open Questions

1. **Does gemini_rust support streaming?**
   - If no: Do we fork it, contribute upstream, or implement direct HTTP?

2. **What protocol does Gemini Live use?**
   - WebSocket? SSE? gRPC?
   - This affects implementation significantly

3. **How do Live models authenticate?**
   - Same API key as regular models?
   - Different endpoints?

4. **Are there different rate limits for streaming?**
   - Same RPM/TPM as non-streaming?
   - Per-session limits?

5. **Do we need streaming in narratives?**
   - Or just for custom code/CLI?
   - Impacts executor changes

6. **Should we support cancellation?**
   - Allow users to stop streams mid-generation?
   - How does this interact with rate limiting?

---

## Success Criteria

### Minimum Viable Product (MVP)

**Primary Goal**: Enable use of live models for better free tier rate limits

- [ ] Can successfully connect to `gemini-2.0-flash-live` model
- [ ] Receive incremental responses as stream
- [ ] Stream completes successfully
- [ ] Errors handled gracefully
- [ ] Basic rate limiting works
- [ ] **Can make significantly more requests with live models vs. standard models** ⭐

### Validation Tests

**Rate Limit Comparison** (run during testing):
```bash
# Test 1: Standard model
time for i in {1..50}; do 
    boticelli run --model gemini-2.0-flash quick_test.toml
done

# Test 2: Live model (should succeed where standard fails)
time for i in {1..50}; do
    boticelli run --model gemini-2.0-flash-live quick_test.toml  
done
```

Expected: Live model allows more requests before hitting rate limits

### Full Implementation

- [ ] All live models work (flash-live, pro-live variants)
- [ ] Standard models also support streaming (nice-to-have)
- [ ] Rate limiting properly accounts for streaming
- [ ] Comprehensive tests (unit + integration)
- [ ] Documentation with live model usage guide
- [ ] CLI supports streaming with `--stream` flag (optional)
- [ ] Backward compatible with existing code
- [ ] **Rate limit benefits clearly documented** ⭐

---

## Resources

### Documentation
- [Gemini API Reference](https://ai.google.dev/api/generate-content)
- [Tokio Streams](https://docs.rs/tokio-stream/latest/tokio_stream/)
- [Async Streams in Rust](https://rust-lang.github.io/async-book/05_streams/01_chapter.html)

### Libraries
- `tokio-stream`: Stream utilities
- `futures`: Stream trait and combinators
- `async-stream`: Macro for creating streams
- `pin-project`: Pin projection for streams

### Similar Implementations
- OpenAI Rust SDK (streaming support)
- Anthropic Rust SDK (streaming support)
- gRPC Rust examples (bidirectional streaming)

---

## Notes

- Start with read-only investigation of gemini_rust
- Prototype with smallest possible change
- Consider backward compatibility at each step
- Document learnings as we go
- Update this document with findings

---

## Timeline Estimate

### Original Estimate (Before Investigation)

- **Investigation**: 1-2 weeks
- **Design & Prototyping**: 1-2 weeks  
- **Implementation**: 2-3 weeks
- **Testing & Documentation**: 1 week
- **Total**: 5-8 weeks for full implementation

### Revised Estimate (After Investigation)

**Major Discovery**: gemini_rust already has complete streaming support via SSE!

#### Fast Track MVP: 2-3 hours ⚡

- [ ] Add StreamChunk type (5 min)
- [ ] Extend BoticelliDriver trait (10 min)
- [ ] Implement in GeminiClient (30 min)
- [ ] Add basic tests (20 min)
- [ ] Update dependencies if needed (5 min)
- [ ] Write documentation (15 min)
- [ ] Test end-to-end (30 min)

**Result**: Working streaming for all Gemini models

#### Full Implementation: 1-2 weeks

**Week 1**:
- [ ] MVP implementation (Day 1)
- [ ] Comprehensive tests (Day 2)
- [ ] Rate limiting refinements (Day 3)
- [ ] Error handling edge cases (Day 4)
- [ ] Documentation and examples (Day 5)

**Week 2** (Optional enhancements):
- [ ] CLI streaming support
- [ ] Narrative executor streaming (if desired)
- [ ] Cancellation support
- [ ] Performance optimization
- [ ] Advanced rate limiting (per-chunk)

**Complexity**: Reduced from HIGH to MEDIUM-LOW due to gemini_rust's existing support

---

## Next Steps

### Immediate (Day 1)

1. ✅ **Investigation Complete** - gemini_rust supports streaming via SSE
2. [ ] Create feature branch: `git checkout -b feature/gemini-streaming`
3. [ ] Implement MVP (Steps 1-6 above)
4. [ ] Run tests: `cargo test --features gemini`
5. [ ] Commit: "Add streaming support to GeminiClient (MVP)"

### Short Term (Week 1)

6. [ ] Add integration tests
7. [ ] Test with different models (gemini-2.0-flash, gemini-2.5-flash, etc.)
8. [ ] Add CLI example: `boticelli run --stream narrative.toml`
9. [ ] Update GEMINI.md with streaming guide
10. [ ] PR review and merge

### Future Enhancements (Week 2+)

11. [ ] Narrative executor streaming support (if needed)
12. [ ] Advanced rate limiting for long streams
13. [ ] Streaming cancellation/timeout
14. [ ] Performance benchmarks
15. [ ] Real-time progress indicators in CLI
16. [ ] Automatic model fallback (live → standard on rate limit)

---

## Conclusion

**Status Update**: Investigation phase COMPLETE ✅

**Primary Goal**: Enable `gemini-2.0-flash-live` and other live models to access **significantly better rate limits** on the free tier, enabling more frequent API usage during development.

**Key Finding**: gemini_rust v1.5.1 has excellent streaming support via Server-Sent Events (SSE). This dramatically simplifies our implementation.

**Business Impact**: 
- Unlock higher RPM/RPD limits on free tier
- Enable more extensive testing without hitting limits
- Faster iteration cycles during development
- Better CI/CD test coverage

**Technical Approach**: 
- Implement streaming in 2-3 hours (MVP)
- Test specifically with `gemini-2.0-flash-live` to validate rate limit benefits
- Document rate limit differences for users
- Provide clear migration path from standard to live models

**Recommendation**: Proceed with MVP implementation immediately. The hard work is already done by gemini_rust - we just need to:
1. Adapt the stream format to our `StreamChunk` type
2. Add live model detection
3. Test and document rate limit benefits

**Confidence Level**: HIGH - Clear path forward with working examples and robust library support.

**Next Action**: Create feature branch and implement Steps 1-6 of MVP plan (estimated 2-3 hours).

---

## UPDATED RECOMMENDATIONS (2025-01-17)

### Critical Discovery: "Live" Models Don't Exist

After querying the Gemini v1beta API (`ListModels`), we confirmed:
- ❌ **NO models named "gemini-2.0-flash-live" exist**
- ❌ **NO models with "live" in the name exist**
- ✅ 2 models support `bidiGenerateContent` (bidirectional streaming):
  - `gemini-2.0-flash-exp`
  - `gemini-2.0-flash-exp-image-generation`

### What This Means

**Original Plan**: Implement streaming to access "live" models with better rate limits
**Reality**: The premise was incorrect - those models don't exist

### Revised Options

#### Option 1: STOP - No Action Needed (RECOMMENDED) ✅

**Rationale**:
- Original goal (better rate limits via "live" models) is unachievable
- Current implementation works with all 48 available Gemini models
- SSE streaming exists in gemini_rust but we don't need it yet
- No user-facing benefit until we need interactive features

**When to Revisit**:
- Google releases actual "live" or bidirectional streaming models
- We need real-time interactive features (voice, chat)
- We hit rate limits and need experimental model access

#### Option 2: Implement SSE Streaming Anyway (LOW PRIORITY)

**Reasons TO implement**:
- Future-proofing for when streaming models arrive
- Enable progressive UI updates (show text as it generates)
- Practice with streaming patterns

**Reasons NOT TO implement**:
- No immediate business value
- Adds complexity without user benefit
- Standard models work fine for our use case

#### Option 3: Investigate Bidirectional Streaming (NOT RECOMMENDED)

**Why NOT**:
- Requires WebSocket/gRPC (much more complex than SSE)
- Only 2 experimental models support it
- Experimental models may be unstable/deprecated
- No confirmed rate limit benefits
- High effort, unknown reward

### Final Recommendation

**CLOSE THIS WORK ITEM** and update tests:

1. Remove `test_streaming_with_live_model` test (model doesn't exist)
2. Document finding in GEMINI.md: "Note: As of 2025-01-17, no 'live' models exist"
3. Keep SSE streaming investigation in this doc for future reference
4. Revisit if Google announces live/streaming models with better limits

**Alternative**: If you want streaming UI (text appears as generated):
- Implement SSE streaming as a nice-to-have feature
- Use with existing models (2.0-flash, 2.5-flash, etc.)
- Don't expect rate limit improvements

### Test Cleanup Required

Current failing test:
```rust
#[tokio::test]
#[cfg_attr(not(feature = "api"), ignore)]
async fn test_streaming_with_live_model() {
    // ... tries to use "gemini-2.0-flash-live" which doesn't exist
}
```

**Action**: Mark as `#[ignore]` with comment: "Model doesn't exist - see GEMINI_STREAMING.md"
Or delete entirely since it's testing a non-existent model.


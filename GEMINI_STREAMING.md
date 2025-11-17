# Gemini Live API WebSocket Implementation Plan

**Date**: 2025-11-17
**Status**: Planning Complete - Ready for Implementation
**Priority**: HIGH (enables better rate limits on free tier)
**Estimated Effort**: 10-15 hours

## Executive Summary

**Goal**: Implement WebSocket client support to access Gemini Live API models which have better free-tier rate limits.

**Current Status**: REST API client works for standard models. Live API requires WebSocket protocol, which is not yet implemented.

**Business Value**:
- Better RPM/RPD limits on free tier (confirmed by user's API dashboard)
- Access to real-time bidirectional streaming capabilities
- Future-proofing for voice/multimodal interactions

---

## Technical Research Findings

### WebSocket Endpoint

**URL**: `wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1beta.GenerativeService.BidiGenerateContent`

**Authentication**: API key as query parameter
```
wss://generativelanguage.googleapis.com/ws/...BidiGenerateContent?key=YOUR_API_KEY
```

**Production Note**: For client-side connections, use ephemeral tokens instead of API keys for security.

### Available Models

Confirmed models that work with Live API:
- `models/gemini-2.0-flash-exp`

**To Verify**: Whether `gemini-2.0-flash-live` or `gemini-2.5-flash-live` exist (user mentioned these as having better rate limits).

### Message Protocol

#### Connection Handshake

1. **Client sends setup message** (immediately after WebSocket connection):
```json
{
  "setup": {
    "model": "models/gemini-2.0-flash-exp",
    "generationConfig": {
      "responseModalities": ["TEXT"],
      "temperature": 1.0,
      "maxOutputTokens": 100
    },
    "systemInstruction": {
      "parts": [{"text": "Optional system instruction"}]
    }
  }
}
```

2. **Server responds with setupComplete**:
```json
{
  "setupComplete": {}
}
```

3. **Client must wait** for `setupComplete` before sending additional messages.

#### Text Message Exchange

**Client sends**:
```json
{
  "clientContent": {
    "turns": [
      {
        "role": "user",
        "parts": [{"text": "Hello, how are you?"}]
      }
    ],
    "turnComplete": true
  }
}
```

**Server responds**:
```json
{
  "serverContent": {
    "modelTurn": {
      "parts": [{"text": "I'm doing well, thank you!"}]
    },
    "turnComplete": true
  },
  "usageMetadata": {
    "promptTokenCount": 10,
    "candidatesTokenCount": 20,
    "totalTokenCount": 30
  }
}
```

#### Realtime Audio/Video Input (Optional)

**Client sends**:
```json
{
  "realtimeInput": {
    "mediaChunks": [
      {
        "mimeType": "audio/pcm;rate=16000",
        "data": "<base64_encoded_audio>"
      }
    ]
  }
}
```

#### Server Message Types

All server messages include optional `usageMetadata` plus **exactly one** of:
- `setupComplete` - Handshake confirmation
- `serverContent` - Model-generated content
- `toolCall` - Function call request
- `toolCallCancellation` - Cancel previous tool calls
- `goAway` - Disconnect warning
- `sessionResumptionUpdate` - Session state for resumption

#### Client Message Types

All client messages contain **exactly one** of:
- `setup` - Initial configuration (first message only)
- `clientContent` - Text conversation turns
- `realtimeInput` - Audio/video streaming data
- `toolResponse` - Responses to function calls

### Generation Config Options

```rust
pub struct GenerationConfig {
    pub candidate_count: Option<i32>,
    pub max_output_tokens: Option<i32>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<i32>,
    pub presence_penalty: Option<f64>,
    pub frequency_penalty: Option<f64>,
    pub response_modalities: Option<Vec<String>>, // ["TEXT", "AUDIO"]
    // speech_config, media_resolution available for audio/video
}
```

**Note**: The following fields from standard API are **not supported** in Live API:
- `responseLogprobs`
- `responseMimeType`
- `responseSchema`
- `stopSequence`
- `routingConfig`

---

## Implementation Architecture

### Module Structure

```
src/gemini/
├── mod.rs              # Module exports
├── client.rs           # Existing REST client (GeminiClient)
├── live_client.rs      # NEW: WebSocket client (GeminiLiveClient)
├── live_protocol.rs    # NEW: Message types for Live API
├── error.rs            # Extend with Live API errors
└── rate_limit.rs       # Extend for WebSocket rate limiting
```

### Core Types

#### GeminiLiveClient

```rust
pub struct GeminiLiveClient {
    api_key: String,
}

impl GeminiLiveClient {
    pub async fn connect(&self, model: &str) -> GeminiResult<LiveSession> {
        // 1. Build WebSocket URL with API key
        // 2. Perform WebSocket handshake
        // 3. Send setup message
        // 4. Wait for setupComplete
        // 5. Return LiveSession
    }
}
```

#### LiveSession

```rust
pub struct LiveSession {
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    model: String,
    rate_limiter: Option<LiveRateLimiter>,
}

impl LiveSession {
    /// Send a text message and get complete response
    pub async fn send_text(&mut self, text: &str) -> GeminiResult<String>;

    /// Send a text message and stream responses
    pub async fn send_text_stream(&mut self, text: &str)
        -> GeminiResult<impl Stream<Item = GeminiResult<LiveChunk>>>;

    /// Close the session gracefully
    pub async fn close(self) -> GeminiResult<()>;
}
```

#### Message Protocol Types

```rust
// Client messages
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupMessage {
    pub setup: SetupConfig,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupConfig {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<SystemInstruction>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientContentMessage {
    pub client_content: ClientContent,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientContent {
    pub turns: Vec<Turn>,
    pub turn_complete: bool,
}

// Server messages
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setup_complete: Option<SetupComplete>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_content: Option<ServerContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_metadata: Option<UsageMetadata>,
    // ... other message types
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerContent {
    pub model_turn: ModelTurn,
    pub turn_complete: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelTurn {
    pub parts: Vec<Part>,
}
```

### Integration with Existing Code

Update `GeminiClient` to detect and route to Live API:

```rust
pub struct GeminiClient {
    rest_pool: ModelPool,           // Existing REST clients
    live_client: GeminiLiveClient,  // New WebSocket client
}

impl GeminiClient {
    fn is_live_model(model_name: &str) -> bool {
        // Models that require Live API (WebSocket)
        model_name.contains("-live") || model_name == "gemini-2.0-flash-exp"
    }

    pub async fn generate_stream(
        &self,
        request: &GenerateRequest
    ) -> BoticelliResult<Pin<Box<dyn Stream<Item = BoticelliResult<StreamChunk>> + Send>>> {
        let model_name = self.resolve_model_name(request);

        if Self::is_live_model(&model_name) {
            // Use WebSocket Live API
            self.generate_stream_live(request).await
        } else {
            // Use existing REST API (if we implement SSE streaming later)
            Err(BoticelliError::from(GeminiError::new(
                GeminiErrorKind::StreamingNotSupported(model_name)
            )))
        }
    }

    async fn generate_stream_live(
        &self,
        request: &GenerateRequest
    ) -> BoticelliResult<Pin<Box<dyn Stream<Item = BoticelliResult<StreamChunk>> + Send>>> {
        // 1. Connect to Live API
        // 2. Send message
        // 3. Stream responses
        // 4. Convert to StreamChunk format
    }
}
```

### Rate Limiting for WebSocket

**Challenge**: WebSocket connections are persistent, not discrete requests.

**Solution**: Track messages sent, not connections.

```rust
pub struct LiveRateLimiter {
    messages_sent: AtomicU32,
    window_start: Instant,
    max_messages_per_minute: u32,
}

impl LiveRateLimiter {
    pub async fn acquire(&self) {
        // If approaching limit, sleep until window resets
        let elapsed = self.window_start.elapsed();
        let count = self.messages_sent.load(Ordering::SeqCst);

        if count >= self.max_messages_per_minute {
            if elapsed < Duration::from_secs(60) {
                let wait = Duration::from_secs(60) - elapsed;
                tokio::time::sleep(wait).await;
            }
            // Reset window
            self.messages_sent.store(0, Ordering::SeqCst);
            self.window_start = Instant::now();
        }
    }

    pub fn record_message(&self) {
        self.messages_sent.fetch_add(1, Ordering::SeqCst);
    }
}
```

### Error Handling

New error types needed:

```rust
pub enum GeminiErrorKind {
    // Existing variants...

    /// WebSocket connection failed
    WebSocketConnection(String),

    /// WebSocket handshake failed (setup phase)
    WebSocketHandshake(String),

    /// Invalid message received from server
    InvalidServerMessage(String),

    /// Server sent goAway message
    ServerDisconnect(String),

    /// Stream was interrupted
    StreamInterrupted(String),
}
```

---

## Implementation Plan

### Dependencies to Add

```toml
[dependencies]
tokio-tungstenite = { version = "0.21", features = ["native-tls"] }
futures-util = "0.3"
```

### Sprint 1: WebSocket Foundation (3-5 hours)

**Files to Create**:
- `src/gemini/live_protocol.rs` - Message type definitions
- `src/gemini/live_client.rs` - WebSocket client implementation

**Tasks**:
- [ ] Add `tokio-tungstenite` and `futures-util` dependencies to `Cargo.toml`
- [ ] Create `live_protocol.rs` with all message types (setup, client content, server messages)
- [ ] Implement `GeminiLiveClient::connect()` - WebSocket handshake
- [ ] Implement setup message send and `setupComplete` wait
- [ ] Write unit tests for message serialization/deserialization
- [ ] Test: Verify messages serialize to correct JSON format

**Success Criteria**:
- Can establish WebSocket connection to Live API endpoint
- Can send setup message and receive setupComplete
- All message types serialize/deserialize correctly

### Sprint 2: Basic Message Exchange (2-3 hours)

**Files to Modify**:
- `src/gemini/live_client.rs`

**Tasks**:
- [ ] Implement `LiveSession::send_text()` - blocking text generation
- [ ] Implement response parsing (serverContent -> String)
- [ ] Handle `turnComplete` flag to know when response is done
- [ ] Add basic error handling (connection drops, invalid messages)
- [ ] Write integration test with `#[cfg_attr(not(feature = "api"), ignore)]`
- [ ] Test: Send "Hello" and get response from `gemini-2.0-flash-exp`

**Success Criteria**:
- Can send text message via clientContent
- Can receive complete text response via serverContent
- Integration test passes with real API

### Sprint 3: Streaming Support (2-3 hours)

**Files to Modify**:
- `src/gemini/live_client.rs`
- `src/driver.rs` (add `StreamChunk` type if not exists)

**Tasks**:
- [ ] Implement `LiveSession::send_text_stream()` - returns `Stream<StreamChunk>`
- [ ] Convert `serverContent` messages to `StreamChunk` format
- [ ] Handle incremental responses (multiple serverContent before turnComplete)
- [ ] Yield chunks as they arrive, set `finished: true` on last chunk
- [ ] Test: Verify streaming with prompt that generates long response

**Success Criteria**:
- Streaming returns incremental chunks
- Last chunk has `finished: true`
- Can display text progressively as it arrives

### Sprint 4: Integration with GeminiClient (2 hours)

**Files to Modify**:
- `src/gemini/client.rs` (GeminiClient)
- `src/gemini/mod.rs` (exports)
- `src/driver.rs` (BoticelliDriver trait)

**Tasks**:
- [ ] Add `StreamChunk` type to `src/driver.rs` (if not exists from earlier work)
- [ ] Add `generate_stream()` method to `BoticelliDriver` trait
- [ ] Add `GeminiLiveClient` field to `GeminiClient`
- [ ] Implement `is_live_model()` detection
- [ ] Route streaming requests to Live API vs REST API
- [ ] Convert Live API stream to `StreamChunk` stream
- [ ] Update exports in `src/lib.rs`

**Success Criteria**:
- `GeminiClient` automatically routes to Live API for live models
- Existing REST API code continues working
- Zero regressions in existing tests

### Sprint 5: Rate Limiting (2 hours)

**Files to Create**:
- Tests in `tests/gemini_live_rate_limit_test.rs`

**Files to Modify**:
- `src/gemini/live_client.rs`
- `src/gemini/rate_limit.rs` (or new file)

**Tasks**:
- [ ] Implement `LiveRateLimiter` struct
- [ ] Add rate limiting to `LiveSession::send_text()` and `send_text_stream()`
- [ ] Use existing rate limit configuration from `TierConfig`
- [ ] Add backoff/retry logic for rate limit errors
- [ ] Test: Verify rate limiter prevents exceeding limits

**Success Criteria**:
- Rate limiter prevents sending messages too quickly
- Properly sleeps and resets window
- Integrates with existing rate limit infrastructure

### Sprint 6: Testing & Documentation (2-3 hours)

**Files to Create**:
- `tests/gemini_live_basic_test.rs`
- `tests/gemini_live_streaming_test.rs`

**Files to Modify**:
- `GEMINI.md` - Add Live API usage section
- `README.md` - Mention Live API support

**Tasks**:
- [ ] Write comprehensive integration tests (all with `#[cfg_attr(not(feature = "api"), ignore)]`)
- [ ] Test basic text generation
- [ ] Test streaming
- [ ] Test rate limiting
- [ ] Test error handling (invalid model, connection drop)
- [ ] **Empirical rate limit comparison**: Measure actual RPM/TPD for live vs standard models
- [ ] Document findings in GEMINI.md
- [ ] Add usage examples to documentation
- [ ] Add Live API section to GEMINI.md

**Success Criteria**:
- All tests pass
- Rate limit benefits documented with actual numbers
- Clear usage examples in docs

### Sprint 7: Polish & Error Handling (1-2 hours)

**Files to Modify**:
- `src/gemini/error.rs`
- `src/gemini/live_client.rs`

**Tasks**:
- [ ] Add comprehensive error types for Live API
- [ ] Implement graceful connection cleanup
- [ ] Add logging/tracing with `tracing` crate
- [ ] Handle `goAway` server message gracefully
- [ ] Implement connection timeout handling
- [ ] Test edge cases (network interruption, server errors)

**Success Criteria**:
- All error scenarios handled gracefully
- Good tracing/logging for debugging
- Clean resource cleanup on connection close

---

## Testing Strategy

### Unit Tests (No API Calls)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_message_serialization() {
        let msg = SetupMessage {
            setup: SetupConfig {
                model: "models/gemini-2.0-flash-exp".to_string(),
                generation_config: None,
                system_instruction: None,
            }
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"model\":\"models/gemini-2.0-flash-exp\""));
    }

    #[test]
    fn test_server_message_deserialization() {
        let json = r#"{"setupComplete": {}}"#;
        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        assert!(msg.setup_complete.is_some());
    }

    #[test]
    fn test_is_live_model() {
        assert!(GeminiClient::is_live_model("gemini-2.0-flash-live"));
        assert!(GeminiClient::is_live_model("gemini-2.0-flash-exp"));
        assert!(!GeminiClient::is_live_model("gemini-2.0-flash"));
    }
}
```

### Integration Tests (With API)

```rust
// tests/gemini_live_basic_test.rs
#![cfg(feature = "gemini")]

use boticelli::{GeminiClient, BoticelliDriver, GenerateRequest, Message, Role, Input};
use futures::StreamExt;

#[tokio::test]
#[cfg_attr(not(feature = "api"), ignore)]
async fn test_live_model_basic_generation() {
    let _ = dotenvy::dotenv();

    let client = GeminiClient::new().expect("Failed to create client");

    let request = GenerateRequest {
        messages: vec![Message {
            role: Role::User,
            content: vec![Input::Text("Say 'Hello World' exactly".to_string())],
        }],
        model: Some("gemini-2.0-flash-exp".to_string()),
        max_tokens: Some(10),
        ..Default::default()
    };

    let mut stream = client.generate_stream(&request).await.expect("Stream failed");

    let mut full_text = String::new();
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.expect("Chunk error");
        full_text.push_str(&chunk.text);
        if chunk.finished {
            break;
        }
    }

    assert!(!full_text.is_empty());
    println!("Live API response: {}", full_text);
}

#[tokio::test]
#[cfg_attr(not(feature = "api"), ignore)]
async fn test_live_model_streaming() {
    let _ = dotenvy::dotenv();

    let client = GeminiClient::new().expect("Failed to create client");

    let request = GenerateRequest {
        messages: vec![Message {
            role: Role::User,
            content: vec![Input::Text("Count from 1 to 10".to_string())],
        }],
        model: Some("gemini-2.0-flash-exp".to_string()),
        max_tokens: Some(100),
        ..Default::default()
    };

    let mut stream = client.generate_stream(&request).await.expect("Stream failed");

    let mut chunks = Vec::new();
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.expect("Chunk error");
        println!("Chunk: {}", chunk.text);
        chunks.push(chunk.text.clone());

        if chunk.finished {
            break;
        }
    }

    assert!(chunks.len() > 1, "Should receive multiple chunks");

    let full_text = chunks.join("");
    assert!(full_text.contains('1') || full_text.contains("one"));
}
```

### Rate Limit Measurement Test

```rust
#[tokio::test]
#[cfg_attr(not(feature = "api"), ignore)]
#[ignore] // Only run manually to measure rate limits
async fn test_measure_live_model_rate_limits() {
    // This test measures actual rate limits by making many requests
    // Run manually with: cargo test --features gemini,api -- --ignored test_measure_live_model_rate_limits

    let _ = dotenvy::dotenv();
    let client = GeminiClient::new().unwrap();

    let mut successes = 0;
    let mut rate_limited = false;

    for i in 1..=100 {
        let request = GenerateRequest {
            messages: vec![Message {
                role: Role::User,
                content: vec![Input::Text("Hi".to_string())],
            }],
            model: Some("gemini-2.0-flash-exp".to_string()),
            max_tokens: Some(5),
            ..Default::default()
        };

        match client.generate_stream(&request).await {
            Ok(mut stream) => {
                while let Some(_) = stream.next().await {}
                successes += 1;
                println!("Request {}: Success", i);
            }
            Err(e) => {
                println!("Request {}: Failed - {}", i, e);
                rate_limited = true;
                break;
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    println!("\nLive Model Results:");
    println!("  Successful requests: {}", successes);
    println!("  Rate limited: {}", rate_limited);
}
```

---

## Open Questions & Verification Needed

### 1. Model Names

**Question**: Do `gemini-2.0-flash-live` or `gemini-2.5-flash-live` actually exist?

**Research Shows**: Only `gemini-2.0-flash-exp` confirmed in examples.

**Action**: Test with both during implementation:
- Try `gemini-2.0-flash-live` (what user mentioned)
- Try `gemini-2.0-flash-exp` (what docs show)
- Document which actually work

### 2. Rate Limits

**Question**: What are the actual rate limits for Live API models on free tier?

**Action**: Run empirical test (Sprint 6) to measure:
- RPM (requests per minute)
- TPD (tokens per day)
- Compare to standard models

### 3. Connection Lifecycle

**Question**: How long can WebSocket connections stay open?

**Unknowns**:
- Idle timeout?
- Need keepalive pings?
- Automatic reconnection?

**Action**: Test during implementation, handle `goAway` message.

### 4. API Version

**Observation**: Some examples use `v1alpha`, official docs show `v1beta`.

**Action**: Use `v1beta` (official API reference), fall back to `v1alpha` if needed.

---

## Success Criteria

### Minimum Viable Product (MVP)

- [ ] Can connect to Live API via WebSocket
- [ ] Can send text message and receive response
- [ ] Streaming works (incremental chunks)
- [ ] Basic error handling (connection failures)
- [ ] At least one integration test passes
- [ ] Can use from existing `GeminiClient` API

### Full Implementation

- [ ] All Live API message types supported (text, audio, tools)
- [ ] Rate limiting prevents quota exhaustion
- [ ] Comprehensive error handling
- [ ] All integration tests pass
- [ ] Documentation complete with examples
- [ ] **Rate limit benefits measured and documented**
- [ ] Zero regressions in existing tests

---

## Timeline

**Total Estimate**: 10-15 hours

**Sprint Breakdown**:
1. WebSocket Foundation: 3-5 hours
2. Basic Message Exchange: 2-3 hours
3. Streaming Support: 2-3 hours
4. Integration with GeminiClient: 2 hours
5. Rate Limiting: 2 hours
6. Testing & Documentation: 2-3 hours
7. Polish & Error Handling: 1-2 hours

**Recommended Schedule**:
- Week 1: Sprints 1-4 (foundation + integration)
- Week 2: Sprints 5-7 (rate limiting + polish)

---

## Next Steps

1. **Create feature branch**: `git checkout -b feature/gemini-live-api`
2. **Start Sprint 1**: Add dependencies, create `live_protocol.rs`, implement basic WebSocket connection
3. **Test early**: Get a working WebSocket connection as soon as possible
4. **Iterate**: Build incrementally, test each sprint before moving to next

---

## References

- [Live API WebSocket Reference](https://ai.google.dev/api/live)
- [Live API Documentation](https://ai.google.dev/gemini-api/docs/live)
- [Gemini 2.0 WebSocket Examples](https://gist.github.com/quartzjer/9636066e96b4f904162df706210770e4)
- [Live API Web Console (React Example)](https://github.com/google-gemini/live-api-web-console)
- [Tokio Tungstenite Docs](https://docs.rs/tokio-tungstenite/latest/tokio_tungstenite/)
- [WebSocket Protocol RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)

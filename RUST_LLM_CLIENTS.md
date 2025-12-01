# Rust LLM Client Library Research

**Date:** 2025-12-01  
**Purpose:** Identify best Rust crates for integrating with LLM APIs beyond Gemini

---

## Top Candidates

### 1. async-openai ⭐ **RECOMMENDED FOR OPENAI**
- **Version:** 0.31.1
- **Downloads:** 1.8M+
- **Repository:** https://github.com/64bit/async-openai
- **Description:** Official-quality Rust library for OpenAI
- **License:** MIT

**Features:**
- Full OpenAI API coverage (GPT-4, GPT-3.5, embeddings, etc.)
- Async/await with tokio
- Streaming support
- Well-maintained and actively developed
- Type-safe API

**Pros:**
- Most popular and mature OpenAI client
- Battle-tested in production
- Good documentation
- Similar async patterns to gemini-rust

**Cons:**
- OpenAI only (no Anthropic/Claude support)
- API changes when OpenAI updates

**Use Case:** Direct replacement for gemini-rust when using OpenAI models

---

### 2. rig-core
- **Version:** 0.24.0
- **Downloads:** 156K+
- **Repository:** Check crates.io page
- **Description:** Rust library for building LLM-powered applications

**Features:**
- Multi-provider support (OpenAI, Anthropic, Cohere, etc.)
- High-level abstractions
- Agent/chain patterns
- Tool calling support

**Pros:**
- Unified API across multiple providers
- Higher-level than raw API clients
- Built for agent workflows

**Cons:**
- Smaller community than async-openai
- More opinionated architecture
- May not expose all provider-specific features

**Use Case:** When building agents that need to switch between providers

---

### 3. genai
- **Version:** 0.5.0-alpha.3 (Alpha!)
- **Downloads:** 90K+
- **Repository:** Check crates.io page
- **Description:** General AI library for Rust

**Features:**
- Multi-provider abstraction
- Streaming support
- Simple, unified API

**Pros:**
- Clean, simple API
- Provider-agnostic code

**Cons:**
- **Alpha status** - not production-ready
- Breaking changes likely
- Limited documentation

**Use Case:** Experimental/prototype projects only

---

### 4. langchain-rust
- **Version:** 4.6.0
- **Downloads:** 118K+
- **Repository:** Check crates.io page
- **Description:** LangChain port to Rust

**Features:**
- Chain patterns from LangChain
- Multiple LLM providers
- Document loaders and vector stores

**Pros:**
- Familiar patterns if you know LangChain
- Full framework for LLM apps

**Cons:**
- Heavy framework (may be overkill)
- Python LangChain patterns don't always map well to Rust
- More complex than needed for simple API calls

**Use Case:** When you need the full LangChain ecosystem in Rust

---

## Provider-Specific Recommendations

### OpenAI (GPT-4, GPT-3.5, etc.)
**Primary:** `async-openai`  
**Alternative:** `rig-core` (if need multi-provider)

### Anthropic (Claude)
**Primary:** `rig-core` (has Anthropic support)  
**Alternative:** Custom implementation with `reqwest` (Anthropic doesn't have official Rust SDK)

### Local Models (Ollama, etc.)
**Primary:** `rig-core` (has Ollama support)  
**Alternative:** Direct HTTP calls to Ollama API

---

## Integration Strategy for Botticelli

### Current State
- Using `gemini-rust` crate for Google Gemini
- Clean trait-based architecture (`Driver`, `Executor`)
- Rate limiting and budget multipliers

### Recommended Approach

#### Option 1: Provider-Specific Crates (Recommended)
Keep the current trait-based design and add:
```toml
[dependencies]
gemini-rust = "1.5"           # Current
async-openai = "0.31"         # Add for OpenAI
# No good Anthropic crate - use reqwest directly
```

**Pros:**
- Each crate is optimized for its provider
- No unnecessary abstractions
- Easy to adopt provider-specific features
- Matches current architecture

**Cons:**
- Different APIs per provider
- More code to maintain

#### Option 2: Unified Framework (rig-core)
Replace direct provider clients with rig-core:
```toml
[dependencies]
rig-core = "0.24"  # Handles all providers
```

**Pros:**
- Single API for all providers
- Easier to switch providers
- Less code duplication

**Cons:**
- Another abstraction layer
- May not support all provider features
- Less control over rate limiting
- Smaller community

---

## Recommendation

**For Botticelli:** Use **Option 1** (Provider-Specific Crates)

**Rationale:**
1. Your trait-based architecture already abstracts providers
2. `async-openai` is mature and widely used (1.8M downloads)
3. Direct control over rate limiting and budget multipliers
4. Can add Anthropic support with `reqwest` (same as gemini-rust uses internally)
5. Matches current patterns - minimal refactoring

**Implementation Plan:**
1. Add `async-openai` dependency with `openai` feature flag
2. Create `OpenAIClient` implementing your `Driver` trait
3. Reuse existing rate limiter and budget system
4. Test with GPT-4 on simple narratives
5. Document in GEMINI.md → LLM_PROVIDERS.md

---

## Code Example: Adding OpenAI Support

```rust
// crates/botticelli_models/src/openai/client.rs
use async_openai::{Client, types::*};
use crate::Driver;

pub struct OpenAIClient {
    client: Client,
    rate_limiter: RateLimiter,
}

impl Driver for OpenAIClient {
    async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse> {
        // Same pattern as GeminiClient
        self.rate_limiter.wait().await;
        
        let req = CreateChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: convert_messages(request.messages),
            // ...
        };
        
        let response = self.client.chat().create(req).await?;
        Ok(convert_response(response))
    }
}
```

---

## Research Sources
- crates.io API queries
- GitHub repository stats
- Community discussions
- Download statistics (as of 2025-12-01)

---

*Last Updated: 2025-12-01*
*Researched for: Botticelli v0.2.0+*

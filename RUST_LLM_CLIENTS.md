# Rust LLM Client Library Research

**Date:** 2025-12-01  
**Updated:** 2025-12-01 (Focus on single-provider crates)
**Purpose:** Identify best Rust crates for integrating with LLM APIs beyond Gemini

**Scope:** Single-provider crates only (Botticelli handles multi-provider abstraction)

---

## Single-Provider Crates (Recommended)

### 1. async-openai ⭐ **RECOMMENDED FOR OPENAI**
- **Version:** 0.31.1
- **Downloads:** 1.8M+ | **GitHub Stars:** 1,680 ⭐
- **Repository:** https://github.com/64bit/async-openai
- **Last Updated:** 2025-11-28 (Active!)
- **License:** MIT

**Features:**
- Full OpenAI API coverage (GPT-4, GPT-3.5, GPT-o1, embeddings, etc.)
- Async/await with tokio
- Streaming support
- Assistant API support
- Audio, batch, and administration APIs
- Type-safe API with comprehensive feature flags

**Pros:**
- Most popular and mature OpenAI client in Rust
- Battle-tested in production (1.8M downloads)
- Actively maintained (updated 2 days ago)
- Excellent documentation
- Similar async patterns to gemini-rust
- Comprehensive feature coverage

**Cons:**
- OpenAI only (not multi-provider)
- Large dependency tree

**Free Tier:** No - OpenAI charges per token  
**Use Case:** Production OpenAI integration (GPT-4, GPT-3.5)

---

### 2. openai_dive (Alternative OpenAI Client)
- **Version:** 1.3.3
- **Downloads:** 120K+
- **Repository:** https://github.com/tjardoo/openai-client
- **License:** MIT

**Features:**
- Async OpenAI API access
- Streaming support
- Realtime API support
- Simpler feature set than async-openai
- rustls-tls option

**Pros:**
- Lighter weight than async-openai
- Fewer features = simpler to use
- Good for basic use cases

**Cons:**
- Less comprehensive than async-openai
- Smaller community
- Less battle-tested

**Free Tier:** No - OpenAI charges per token  
**Use Case:** When you don't need full OpenAI API coverage

---

### 3. ollama-rs ⭐ **RECOMMENDED FOR OLLAMA**
- **Version:** 0.3.3
- **Downloads:** 179K+ | **GitHub Stars:** 943 ⭐
- **Repository:** https://github.com/pepperoni21/ollama-rs
- **Last Updated:** 2025-11-30 (Today!) ✅
- **License:** MIT

**Features:**
- Full Ollama API support
- Async/await with tokio
- Streaming support
- Model management (pull, push, delete)
- Chat completions
- Embeddings generation
- Local model execution

**Pros:**
- **Actively maintained** (updated today!)
- Good community support (943 stars)
- Comprehensive Ollama API coverage
- Great for local/self-hosted models
- No API costs

**Cons:**
- Requires Ollama running locally
- Performance depends on local hardware
- Limited to Ollama-compatible models

**Free Tier:** Yes! ✅ Completely free (local execution)  
**Use Case:** Self-hosted models, development, testing, privacy-focused deployments

**Supported Models:** Llama 2, Mistral, CodeLlama, DeepSeek-Coder, and all Ollama models

---

### 4. anthropic-sdk
- **Version:** 0.1.5
- **Downloads:** 53K+ | **GitHub Stars:** 34
- **Repository:** https://github.com/mixpeal/anthropic-sdk
- **Last Updated:** 2024-07-23 (6 months ago) ⚠️
- **License:** Check repository

**Features:**
- Claude API access
- Async/await support
- Basic API coverage

**Pros:**
- Only dedicated Anthropic Rust SDK found
- Cleaner than raw HTTP calls

**Cons:**
- **Not actively maintained** (last update 6 months ago)
- Small community (34 stars)
- Limited features
- May not support latest Claude APIs (Claude 3.5 Sonnet, etc.)

**Free Tier:** No - Anthropic charges per token  
**Use Case:** Anthropic/Claude integration (but consider raw HTTP with `reqwest` instead)

**Alternative:** Use `reqwest` directly with Anthropic's HTTP API - may be more reliable given SDK staleness

---

---

## Summary Table

| Provider | Crate | Free Tier | Stars | Downloads | Updated | Recommendation |
|----------|-------|-----------|-------|-----------|---------|----------------|
| **Ollama** | `ollama-rs` | ✅ Yes (local) | 943 | 179K | Today | ⭐ **START HERE** |
| **OpenAI** | `async-openai` | ❌ No | 1,680 | 1.8M | 2 days ago | ⭐ Production |
| OpenAI Alt | `openai_dive` | ❌ No | - | 120K | - | Alternative |
| **Groq** | Use `async-openai` | ✅ Yes (limited) | - | - | - | Free tier option |
| **Anthropic** | `anthropic-sdk` | ❌ No | 34 | 53K | 6 mo ago | ⚠️ Stale, use reqwest |
| **Gemini** | `gemini-rust` | ✅ Yes (15 RPM) | - | - | Current | Currently using |
| Hugging Face | Raw HTTP | ✅ Yes (limited) | - | - | - | Use reqwest |

---

## Free Tier Providers & Their Rust Crates

### ✅ Completely Free (Self-Hosted)

#### Ollama (Local Execution)
- **Crate:** `ollama-rs` ⭐ **HIGHLY RECOMMENDED**
- **Cost:** Free (uses local compute)
- **Models:** Llama 2, Mistral, CodeLlama, DeepSeek-Coder, Phi, etc.
- **Setup:** Install Ollama, pull models, run locally
- **Pros:** No API keys, no rate limits, privacy, offline capable
- **Cons:** Requires local GPU/CPU, slower than cloud APIs

### ⚠️ Free Tier (Limited)

#### Groq (Fast Inference)
- **Crate:** None found - Use OpenAI-compatible API with `async-openai`
- **Cost:** Free tier available (limited requests/day)
- **Models:** Llama 3, Mixtral, Gemma
- **Note:** Groq API is OpenAI-compatible, so `async-openai` works with base URL override

#### Hugging Face Inference API
- **Crate:** No dedicated client - Use `reqwest` with HTTP API
- **Cost:** Free tier (rate limited)
- **Models:** Thousands of open models
- **Note:** Simple HTTP API, no official Rust SDK needed

### 💰 Paid (No Free Tier)

#### OpenAI
- **Crate:** `async-openai` ⭐ or `openai_dive`
- **Cost:** Pay per token (no free tier)
- **Models:** GPT-4, GPT-3.5, GPT-o1

#### Anthropic (Claude)
- **Crate:** `anthropic-sdk` (stale) or raw `reqwest`
- **Cost:** Pay per token (no free tier)
- **Models:** Claude 3.5 Sonnet, Claude 3 Opus, Claude 3 Haiku

#### Google Gemini
- **Crate:** `gemini-rust` ⭐ (currently using)
- **Cost:** Free tier available (15 RPM, 1M TPM, 1500 RPD)
- **Models:** Gemini 2.0 Flash, Gemini 1.5 Pro/Flash

---

---

## Recommendations by Priority

### 1. **Ollama (Free, Local) - HIGHEST PRIORITY** ⭐
- **Crate:** `ollama-rs` (179K downloads, 943 stars, updated today)
- **Why:** Completely free, no API costs, privacy, actively maintained
- **Models:** Llama 2, Mistral, CodeLlama, DeepSeek-Coder, Phi-2
- **Use Case:** Development, testing, cost-sensitive production
- **Implementation:** Add as optional feature, minimal changes to current architecture

### 2. **OpenAI (Paid, Comprehensive)**
- **Crate:** `async-openai` (1.8M downloads, 1,680 stars)
- **Why:** Most mature Rust LLM client, production-ready
- **Models:** GPT-4, GPT-3.5, GPT-o1
- **Use Case:** When you need cutting-edge performance and can pay
- **Implementation:** Straightforward integration with existing Driver trait

### 3. **Groq (Free Tier, Fast)**
- **Crate:** Use `async-openai` with base URL override
- **Why:** OpenAI-compatible API, free tier, very fast inference
- **Models:** Llama 3, Mixtral, Gemma
- **Use Case:** Free tier for testing, fast inference
- **Implementation:** Same as OpenAI with different endpoint

### 4. **Anthropic (Paid, Claude)**
- **Crate:** Raw `reqwest` (anthropic-sdk is stale)
- **Why:** Claude is excellent for reasoning tasks
- **Models:** Claude 3.5 Sonnet, Claude 3 Opus
- **Use Case:** When you need Claude specifically
- **Implementation:** Direct HTTP calls, similar to how gemini-rust works internally

## Implementation Strategy

### Phase 1: Add Ollama Support (Free, Easy)
```toml
[dependencies]
ollama-rs = { version = "0.3", optional = true }

[features]
ollama = ["ollama-rs"]
```

**Benefits:**
- Zero API costs
- Test entire narrative system locally
- No rate limits
- Privacy (no data leaves your machine)
- Can run on development machines

### Phase 2: Add OpenAI (When Budget Allows)
```toml
[dependencies]
async-openai = { version = "0.31", optional = true }

[features]
openai = ["async-openai"]
```

**Benefits:**
- Best-in-class models (GPT-4)
- Proven Rust client
- Similar patterns to gemini-rust

### Phase 3: Consider Groq (Free Tier)
Use `async-openai` with Groq's OpenAI-compatible endpoint:
```rust
let config = OpenAIConfig::new()
    .with_api_base("https://api.groq.com/openai/v1");
```

**Benefits:**
- Free tier
- Very fast inference
- Reuse OpenAI client code

---

## Code Examples

### Adding Ollama Support (Recommended First)

```rust
// crates/botticelli_models/src/ollama/client.rs
#[cfg(feature = "ollama")]
use ollama_rs::{Ollama, generation::completion::request::GenerationRequest};
use crate::Driver;

pub struct OllamaClient {
    client: Ollama,
    model_name: String,
    rate_limiter: Option<RateLimiter>, // Optional since it's local
}

impl OllamaClient {
    pub fn new(model: &str) -> Result<Self> {
        Ok(Self {
            client: Ollama::default(), // Connects to localhost:11434
            model_name: model.to_string(),
            rate_limiter: None, // No rate limiting for local
        })
    }
}

impl Driver for OllamaClient {
    async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse> {
        let req = GenerationRequest::new(
            self.model_name.clone(),
            convert_prompt(request.messages)
        );
        
        let response = self.client.generate(req).await?;
        Ok(convert_response(response))
    }
}
```

### Adding OpenAI Support

```rust
// crates/botticelli_models/src/openai/client.rs
#[cfg(feature = "openai")]
use async_openai::{Client, types::*};
use crate::Driver;

pub struct OpenAIClient {
    client: Client<OpenAIConfig>,
    rate_limiter: RateLimiter,
    model: String,
}

impl OpenAIClient {
    pub fn new_with_tier(model: &str, tier: TierConfig) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            rate_limiter: RateLimiter::from_tier(tier),
            model: model.to_string(),
        })
    }
    
    // For Groq: override base URL
    pub fn new_groq(model: &str, tier: TierConfig) -> Result<Self> {
        let config = OpenAIConfig::new()
            .with_api_base("https://api.groq.com/openai/v1");
        Ok(Self {
            client: Client::with_config(config),
            rate_limiter: RateLimiter::from_tier(tier),
            model: model.to_string(),
        })
    }
}

impl Driver for OpenAIClient {
    async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse> {
        self.rate_limiter.wait().await;
        
        let req = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(convert_messages(request.messages))
            .build()?;
        
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

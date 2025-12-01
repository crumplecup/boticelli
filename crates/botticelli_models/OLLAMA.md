# Ollama Integration Guide

**Status:** ✅ Implemented  
**Version:** 0.2.0  
**Last Updated:** 2025-12-01

---

## Overview

Botticelli now supports **Ollama** for local LLM execution, enabling:
- **Zero API costs** - no tokens, no rate limits
- **Complete privacy** - data never leaves your machine
- **Offline capability** - works without internet
- **Free development** - test narratives without burning API credits

---

## Installation

### 1. Install Ollama

**Linux/Mac:**
```bash
curl https://ollama.ai/install.sh | sh
```

**Windows:**
Download from https://ollama.ai/download

**Verify installation:**
```bash
ollama --version
```

### 2. Pull Models

Download models you want to use:

```bash
# Recommended for general use
ollama pull llama2          # Meta's Llama 2 (7B)
ollama pull mistral         # Mistral 7B (fast, good quality)

# For coding tasks
ollama pull codellama       # Meta's Code Llama
ollama pull deepseek-coder  # DeepSeek Coder (excellent for code)

# Lightweight/fast
ollama pull phi             # Microsoft Phi-2 (2.7B, very fast)
```

**List installed models:**
```bash
ollama list
```

### 3. Start Ollama Server

Ollama runs as a background service by default. To start manually:

```bash
ollama serve
```

By default, Ollama listens on `http://localhost:11434`.

---

## Usage

### Basic Example

```rust
use botticelli_models::OllamaClient;
use botticelli_interface::BotticelliDriver;
use botticelli_core::{GenerateRequest, Message, Role, Input};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client (assumes Ollama running on localhost:11434)
    let client = OllamaClient::new("llama2")?;
    
    // Optional: Validate server and model are available
    client.validate().await?;
    
    // Create a message
    let message = Message::builder()
        .role(Role::User)
        .content(vec![Input::Text("Hello! How are you?".to_string())])
        .build()?;
    
    // Generate response
    let request = GenerateRequest::builder()
        .messages(vec![message])
        .build()?;
    
    let response = client.generate(&request).await?;
    
    println!("Response: {:?}", response.outputs());
    
    Ok(())
}
```

### Custom Server URL

```rust
let client = OllamaClient::new_with_url("llama2", "http://192.168.1.100:11434")?;
```

### Auto-Pull Missing Models

```rust
// Downloads model if not available locally
client.ensure_model().await?;
```

---

## Configuration

Ollama configuration is in `botticelli.toml`:

```toml
[providers.ollama]
default_tier = "local"

[providers.ollama.tiers.local]
name = "Local"
rpm = null  # No rate limits
tpm = null
rpd = null
max_concurrent = 4  # Hardware-based limit
cost_per_million_input_tokens = 0.0
cost_per_million_output_tokens = 0.0

# Model-specific concurrency
[providers.ollama.tiers.local.models.llama2]
max_concurrent = 2  # Large model
```

**Adjust `max_concurrent` based on your hardware:**
- **High-end GPU (RTX 4090, A100):** 6-8
- **Mid-range GPU (RTX 3070, 4070):** 4
- **Low-end GPU / CPU only:** 2

---

## Narrative Integration

### Narrative TOML

```toml
[narrative]
name = "test_ollama"
description = "Test narrative using Ollama"

[narrative.llm]
provider = "ollama"  # Use Ollama instead of Gemini
model = "llama2"     # Model name

[narrative.steps.generate]
processor = "ContentGenerationProcessor"
# ... rest of config
```

### Switching Providers

Change between Ollama and Gemini by editing the narrative TOML:

```toml
# Use Ollama (local, free)
[narrative.llm]
provider = "ollama"
model = "mistral"

# Use Gemini (cloud, API cost)
[narrative.llm]
provider = "gemini"
model = "gemini-2.0-flash"
```

---

## Supported Models

### Recommended Models

| Model | Size | Speed | Quality | Use Case |
|-------|------|-------|---------|----------|
| **mistral** | 7B | Fast | Good | General purpose |
| **llama2** | 7B | Medium | Good | Chat, Q&A |
| **phi** | 2.7B | Very Fast | Decent | Quick tasks, testing |
| **codellama** | 7B | Medium | Excellent | Code generation |
| **deepseek-coder** | 6.7B | Fast | Excellent | Code, debugging |

### Full Model List

Browse all available models:
```bash
ollama list --available
```

Or visit: https://ollama.ai/library

---

## Troubleshooting

### Server Not Running

**Error:** `Ollama server not running at http://localhost:11434`

**Solution:**
```bash
# Start Ollama service
ollama serve

# Or check if running
ps aux | grep ollama
```

### Model Not Found

**Error:** `Model not found: llama2`

**Solution:**
```bash
# Pull the model
ollama pull llama2

# List installed models
ollama list
```

### Out of Memory

**Error:** Model fails to load or crashes

**Solutions:**
1. Use a smaller model (try `phi` or `mistral:7b`)
2. Reduce `max_concurrent` in config
3. Close other applications
4. Use quantized models (e.g., `llama2:7b-q4_0`)

### Slow Performance

**CPU-only execution** is significantly slower than GPU.

**Solutions:**
1. Use smaller models (`phi`, `mistral:7b`)
2. Reduce concurrent requests
3. Consider using Gemini for production
4. Upgrade hardware (add GPU)

---

## Performance Comparison

### Speed (Approximate)

| Setup | Tokens/sec | Relative Speed |
|-------|------------|----------------|
| **Ollama (RTX 4090)** | 100-150 | 🚀🚀🚀 Fast |
| **Ollama (RTX 3070)** | 50-80 | 🚀🚀 Medium |
| **Ollama (CPU only)** | 5-15 | 🐌 Slow |
| **Gemini API** | Varies | 🚀🚀 Fast (network dependent) |

### Cost Comparison

| Provider | Cost | Use Case |
|----------|------|----------|
| **Ollama** | $0 | ✅ Development, testing, privacy-sensitive |
| **Gemini Free** | $0 (limited) | ✅ Low-volume production |
| **Gemini Paid** | $0.075/M tokens | Production, high-volume |

---

## Feature Comparison: Ollama vs Gemini

| Feature | Ollama | Gemini |
|---------|--------|--------|
| **Cost** | Free | Free tier + paid |
| **Speed** | Hardware-dependent | Fast (network-dependent) |
| **Privacy** | Complete | Data sent to Google |
| **Rate Limits** | None | Yes (15 RPM free tier) |
| **Offline** | ✅ Yes | ❌ No |
| **Model Selection** | 100+ open models | Google models only |
| **Setup** | Manual install | API key only |
| **Hardware Requirements** | GPU recommended | None |

---

## Best Practices

### Development Workflow

1. **Use Ollama for development**
   - Fast iteration
   - No API costs
   - Test narratives freely

2. **Switch to Gemini for production**
   - Consistent performance
   - No hardware requirements
   - Cloud scalability

### Model Selection

- **Prototyping:** `phi` (fastest)
- **Development:** `mistral` (good balance)
- **Code tasks:** `deepseek-coder`
- **Production:** Switch to Gemini

### Resource Management

```toml
# Development (laptop)
[providers.ollama.tiers.local]
max_concurrent = 2

# Production (server with GPU)
[providers.ollama.tiers.local]
max_concurrent = 8
```

---

## Testing

### Run Tests

```bash
# Ensure Ollama is running and llama2 is installed
ollama pull llama2
ollama serve

# Run tests
cargo test --package botticelli_models --features ollama -- --ignored
```

### Example Test

```rust
#[tokio::test]
#[ignore] // Requires Ollama
async fn test_generation() {
    let client = OllamaClient::new("llama2").unwrap();
    client.validate().await.unwrap();
    
    let message = Message::builder()
        .role(Role::User)
        .content(vec![Input::Text("Hello".into())])
        .build().unwrap();
    
    let request = GenerateRequest::builder()
        .messages(vec![message])
        .build().unwrap();
    
    let response = client.generate(&request).await.unwrap();
    assert!(!response.outputs().is_empty());
}
```

---

## FAQ

### Q: Can I use Ollama and Gemini in the same project?

**A:** Yes! Enable both features and switch via narrative TOML:

```toml
[dependencies]
botticelli_models = { version = "0.2", features = ["gemini", "ollama"] }
```

### Q: Which models work best?

**A:** For general use: `mistral` or `llama2`. For code: `deepseek-coder`.

### Q: Do I need a GPU?

**A:** No, but CPU-only is 10-20x slower. GPU highly recommended for production.

### Q: Can I use custom fine-tuned models?

**A:** Yes! Import to Ollama and reference by name:
```bash
ollama create my-model -f Modelfile
```

### Q: How much disk space do models use?

**A:** 3-15 GB per model. Plan accordingly.

---

## Resources

- **Ollama Website:** https://ollama.ai
- **Model Library:** https://ollama.ai/library
- **Ollama Docs:** https://github.com/ollama/ollama
- **Botticelli Issues:** https://github.com/crumplecup/botticelli/issues

---

*Last Updated: 2025-12-01*  
*Next: Anthropic (Claude) integration*

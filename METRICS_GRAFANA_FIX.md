# Metrics & Grafana Implementation Issue

**Status:** In Progress  
**Priority:** High  
**Created:** 2025-11-30

## The OpenTelemetry v0.24 → v0.31 Breaking Changes Problem

### Why This Document Exists

**WE KEEP REPEATING THE SAME MISTAKE.** This document exists to break that cycle.

### The Root Cause: Training Data vs. Reality

**Critical Context:** AI training data is predominantly based on OpenTelemetry Rust **v0.24**. The current version is **v0.31**. As a pre-1.0 crate, OpenTelemetry has had **significant breaking changes** across these versions.

### The Pattern We Keep Repeating

1. AI implements metrics using v0.24 patterns (e.g., `opentelemetry-prometheus` crate)
2. Cargo check fails with compilation errors
3. Human says: "Read the v0.31 docs, this approach is deprecated/changed"
4. AI promises to check docs and fixes immediate issue
5. AI implements partial fix, but other v0.24 patterns remain
6. **Next implementation: AI falls back to v0.24 patterns from training data**
7. **Repeat from step 1**

### Why This Keeps Happening

**Probabilistic Pattern Matching:** AI systems operate on statistical patterns from training data, not deterministic rules. When OpenTelemetry v0.24 examples dominate the training corpus, these patterns have higher probability weights than:
- Project documentation
- Explicit instructions
- Recent corrections in conversation history

**The training data bias is stronger than corrective instructions.**

### The "Autopilot" Effect

Think of it like driving the same route to work every day - you might take the wrong turn on autopilot even when you consciously know it's the weekend and you're going somewhere else. The AI exhibits the same "muscle memory":

- **Training data (v0.24) creates the strongest learned patterns**
- Project documentation shifts probability distributions but doesn't override them
- In the flow of code generation, familiar patterns activate automatically
- Immediate context and statistical associations dominate over abstract rules

**Human vigilance is essential:**
- Search for `opentelemetry-prometheus` in any metrics changes
- Verify OTLP exporter usage, not deprecated Prometheus exporter
- Check Cargo.toml additions match v0.31 API
- Run `cargo check` immediately after metrics code generation
- **Expect the AI to reach for v0.24 patterns repeatedly - this is normal behavior**

### Major Breaking Changes v0.24 → v0.31

1. **`opentelemetry-prometheus` crate removed** (v0.25+)
   - Old: Direct Prometheus exporter
   - New: OTLP exporter → Collector → Prometheus

2. **Metrics API redesign** (v0.20+)
   - Old: `Meter::u64_counter()`, `Meter::f64_histogram()`
   - New: Builder pattern with `.init()` calls

3. **SDK initialization changes**
   - Old: `MeterProvider::builder().with_reader()`
   - New: Different builder patterns per exporter type

4. **Resource API changes**
   - Old: `Resource::new()`
   - New: `Resource::default()` with merge patterns

**Critical Fact:** Code examples from v0.24 will **not compile** in v0.31. Always verify against current crate documentation.

## The Correct Approach for v0.31

### Architecture Overview

```
Rust App (OpenTelemetry SDK v0.31)
    ↓ (OTLP/HTTP)
OpenTelemetry Collector
    ↓ (Prometheus Remote Write or expose /metrics)
Prometheus
    ↓ (PromQL queries)
Grafana Dashboards
```

### Key Dependencies (v0.31)

```toml
[dependencies]
opentelemetry = "0.31"
opentelemetry_sdk = { version = "0.31", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.31", features = ["metrics", "http-proto"] }
```

**NOT THESE (deprecated/removed):**
```toml
# ❌ WRONG - This crate was removed
opentelemetry-prometheus = "0.x"

# ❌ WRONG - This exporter was removed  
opentelemetry-prometheus-exporter = "0.x"
```

### Implementation Pattern

```rust
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_otlp::MetricsExporterBuilder;

// Create OTLP exporter
let exporter = opentelemetry_otlp::MetricsExporterBuilder::default()
    .with_http()
    .with_endpoint("http://localhost:4318/v1/metrics")
    .build()?;

// Create meter provider with periodic reader
let reader = PeriodicReader::builder(exporter, runtime::Tokio)
    .with_interval(Duration::from_secs(30))
    .build();

let provider = SdkMeterProvider::builder()
    .with_reader(reader)
    .build();
```

## Current Status

### What We Have

1. ✅ Tracing working (Jaeger via OTLP)
2. ✅ Bot server running in container
3. ✅ Prometheus running at `localhost:9090`
4. ✅ Grafana running at `localhost:3000`
5. ❌ No metrics flowing from app → Prometheus

### What We Need

1. Initialize metrics SDK with OTLP exporter in bot server
2. Add instrumentation counters/histograms for:
   - LLM API request success/failure rates
   - JSON parsing success/failure rates  
   - Request latencies
3. Configure OpenTelemetry Collector (or direct OTLP → Prometheus)
4. Verify metrics in Prometheus
5. Create Grafana dashboards

## Diagnostic Steps Completed

### Step 1: Check App Metrics Initialization ✅

- Located metrics setup in `crates/botticelli_server/src/observability.rs`
- Uses placeholder "stdout" exporter
- **Problem:** Never initializes real metrics provider

### Step 2: Attempted Fix ❌

- Tried to add `opentelemetry-prometheus` dependency
- **Failed:** Crate doesn't exist in v0.31
- **Root cause:** Using outdated v0.24 patterns

## Next Steps

### Phase 1: Metrics SDK Setup (OTLP Approach)

1. Add OTLP metrics exporter to `botticelli_server/Cargo.toml`
2. Update `observability.rs` to initialize metrics with OTLP exporter
3. Point exporter at OpenTelemetry Collector or Prometheus OTLP receiver

### Phase 2: Instrumentation

Add metrics in `crates/botticelli_ai/src/gemini/mod.rs`:
```rust
let meter = global::meter("botticelli.llm");
let request_counter = meter
    .u64_counter("llm.requests")
    .with_description("LLM API requests")
    .build();

let error_counter = meter
    .u64_counter("llm.errors")
    .with_description("LLM API errors")
    .build();
```

### Phase 3: Infrastructure

**Option A: Direct OTLP to Prometheus**
- Enable Prometheus OTLP receiver
- Configure Prometheus to scrape itself

**Option B: Use OpenTelemetry Collector** (Recommended)
- Collector receives OTLP from app
- Collector exports to Prometheus
- Better for multi-backend scenarios

### Phase 4: Verification

1. Check app logs for metrics initialization
2. Query Prometheus: `up{job="botticelli"}`
3. Query custom metrics: `llm_requests_total`
4. Import dashboards to Grafana

## References

- [OpenTelemetry Rust v0.31 docs](https://docs.rs/opentelemetry/0.31.0)
- [OpenTelemetry OTLP docs](https://docs.rs/opentelemetry-otlp/0.31.0)
- [Prometheus OTLP receiver](https://prometheus.io/docs/prometheus/latest/feature_flags/#otlp-receiver)

## Lessons Learned

1. **Always check actual crate versions** - Don't assume API patterns from training data
2. **Document deprecations explicitly** - Future sessions need this context
3. **The Prometheus exporter was removed for good reasons** - OTLP is the universal protocol
4. **AI training data bias is real** - Explicit documentation helps but doesn't eliminate old patterns

---

**TO FUTURE AI SESSIONS:** If you see this document and are tempted to use `opentelemetry-prometheus`, **STOP**. Read this section again. Use OTLP. The Prometheus exporter doesn't exist anymore.

---

## Implementation Status (2025-11-30)

### ✅ Completed

1. **Dependencies configured** (`Cargo.toml`):
   - `opentelemetry = "0.31"`
   - `opentelemetry-otlp = { version = "0.31", features = ["tokio", "metrics"] }`
   - `opentelemetry_sdk = { version = "0.31", features = ["rt-tokio"] }`
   - `opentelemetry-stdout = { version = "0.31", features = ["trace", "metrics"] }`

2. **OTLP metrics exporter** (`crates/botticelli/src/observability.rs`):
   - Metrics initialization via OTLP in `init_metrics()` function
   - Supports both stdout (dev) and OTLP (production) exporters
   - Configured via `OTEL_EXPORTER` and `OTEL_EXPORTER_OTLP_ENDPOINT` env vars

3. **LLM metrics instrumentation** (`crates/botticelli_models/src/`):
   - `metrics.rs` module with `LlmMetrics` struct
   - Tracks: requests, errors, duration, tokens (prompt/completion/total)
   - Labels: provider, model, error_type
   - Already integrated into `gemini/client.rs` generate methods

4. **Code compiles**: `cargo check` passes without errors

### 🔍 Next Steps

**Verify the metrics pipeline is working:**

1. Start the actor-server with OTLP exporter:
   ```bash
   OTEL_EXPORTER=otlp OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 \
   cargo run --bin actor-server --features discord
   ```

2. Check if metrics are being exported:
   - View OTLP endpoint logs (if using collector)
   - Or check stdout if using stdout exporter

3. Verify Prometheus is scraping (if configured):
   ```bash
   curl http://localhost:9090/api/v1/label/__name__/values | grep llm
   ```

4. Check Grafana dashboards can query the metrics

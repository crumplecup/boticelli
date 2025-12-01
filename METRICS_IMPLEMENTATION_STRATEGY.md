# Metrics Implementation Strategy (v0.31 Correct Approach)

**Date:** 2025-11-30  
**Status:** Strategy Document - Ready for Implementation

## Problem Statement

Grafana dashboards show no metrics data. After multiple failed attempts using outdated v0.24 patterns, we need to implement the CORRECT v0.31 approach based on actual OpenTelemetry Rust source code.

## Root Cause of Previous Failures

**Training Data Bias:** AI training data features v0.24 patterns. When generating code, muscle memory reaches for:
- ❌ `opentelemetry-prometheus` crate (removed in v0.25)
- ❌ Direct Prometheus exporters
- ❌ Old metrics API patterns

**Solution:** Read ACTUAL v0.31 source code from `/tmp/otel-rust/opentelemetry-otlp/examples/basic-otlp-http/src/main.rs`

---

## The CORRECT v0.31 Architecture

### Source of Truth
**File:** `/tmp/otel-rust/opentelemetry-otlp/examples/basic-otlp-http/src/main.rs` (lines 55-66)

```rust
fn init_metrics() -> SdkMeterProvider {
    let exporter = MetricExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary) 
        .build()
        .expect("Failed to create metric exporter");

    SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(get_resource())
        .build()
}
```

### Key Observations

1. **No Prometheus-specific code** - just OTLP MetricExporter
2. **`.with_http()`** - Uses HTTP transport (not gRPC)
3. **`Protocol::HttpBinary`** - Binary protobuf format
4. **`.with_periodic_exporter(exporter)`** - Automatic export every 30s
5. **`.with_resource()`** - Service name and attributes
6. **No endpoint in code** - Uses environment variable `OTEL_EXPORTER_OTLP_ENDPOINT`

---

## Implementation Plan

### Phase 1: Fix `botticelli_core/src/observability.rs`

**Current Problem:** Code uses wrong patterns, tries to configure endpoints manually

**Correct Approach:**

```rust
use opentelemetry_otlp::{MetricExporter, Protocol};
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::Resource;

#[instrument]
fn init_metrics() -> Result<SdkMeterProvider, ObservabilityError> {
    info!("Initializing OpenTelemetry metrics (OTLP)");
    
    // Use environment variable OTEL_EXPORTER_OTLP_ENDPOINT
    // Default: http://localhost:4318 (OTLP/HTTP standard port)
    let exporter = MetricExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .build()
        .map_err(|e| {
            error!(?e, "Failed to create OTLP metric exporter");
            ObservabilityError::Metrics(format!("OTLP exporter: {}", e))
        })?;
    
    debug!("OTLP metric exporter created");
    
    let resource = Resource::builder()
        .with_service_name("botticelli-actor-server")
        .build();
    
    let meter_provider = SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(resource)
        .build();
    
    global::set_meter_provider(meter_provider.clone());
    
    info!("✓ Metrics initialized (OTLP/HTTP)");
    
    // Test metric
    let meter = global::meter("botticelli");
    let counter = meter.u64_counter("test_startup").build();
    counter.add(1, &[]);
    debug!("Test startup metric recorded");
    
    Ok(meter_provider)
}
```

### Phase 2: Environment Variables

**Set in `Containerfile`:**

```dockerfile
# OTLP endpoint for metrics
# Note: Must point to OTLP-compatible collector, not Prometheus directly
ENV OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4318
```

**Why not Prometheus directly?**
- Prometheus OTLP receiver is at `/api/v1/otlp/v1/metrics`
- OpenTelemetry OTLP exporter expects standard OTLP endpoint structure
- **Solution:** Use OTEL Collector as intermediary

### Phase 3: Add OpenTelemetry Collector

**Why needed:**
- App exports OTLP → Collector
- Collector translates → Prometheus format
- Prometheus scrapes Collector's `/metrics` endpoint

**`docker-compose.observability.yml`:**

```yaml
otel-collector:
  image: otel/opentelemetry-collector-contrib:latest
  container_name: botticelli-otel-collector
  command: ["--config=/etc/otel-collector-config.yaml"]
  volumes:
    - ./otel-collector-config.yaml:/etc/otel-collector-config.yaml
  ports:
    - "4318:4318"   # OTLP HTTP receiver
    - "8889:8889"   # Prometheus exporter for scraping
  networks:
    - botticelli

prometheus:
  image: prom/prometheus:latest
  container_name: botticelli-prometheus
  command:
    - '--config.file=/etc/prometheus/prometheus.yml'
  volumes:
    - ./prometheus.yml:/etc/prometheus/prometheus.yml
  ports:
    - "9090:9090"
  networks:
    - botticelli
```

**`otel-collector-config.yaml`:**

```yaml
receivers:
  otlp:
    protocols:
      http:
        endpoint: 0.0.0.0:4318

exporters:
  prometheus:
    endpoint: "0.0.0.0:8889"

service:
  pipelines:
    metrics:
      receivers: [otlp]
      exporters: [prometheus]
```

**`prometheus.yml`:**

```yaml
scrape_configs:
  - job_name: 'otel-collector'
    static_configs:
      - targets: ['otel-collector:8889']
```

### Phase 4: Update Dependencies

**`crates/botticelli_core/Cargo.toml`:**

```toml
[dependencies]
opentelemetry = "0.31"
opentelemetry_sdk = { version = "0.31", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.31", features = ["metrics", "http-proto"] }

# NOT THESE (removed in v0.25):
# opentelemetry-prometheus = "0.x"  ❌
```

### Phase 5: Testing Strategy

**Step 1: Verify OTLP Collector**
```bash
# Should return 405 (endpoint exists, needs POST)
curl -v http://localhost:4318/v1/metrics
```

**Step 2: Rebuild and restart**
```bash
just bot-build
just bot-restart
```

**Step 3: Check logs**
```bash
podman logs botticelli-actor-server 2>&1 | grep -i metric
```

Expected:
```
INFO Initializing OpenTelemetry metrics (OTLP)
DEBUG OTLP metric exporter created  
INFO ✓ Metrics initialized (OTLP/HTTP)
DEBUG Test startup metric recorded
```

**Step 4: Verify Collector receives metrics**
```bash
podman logs botticelli-otel-collector 2>&1 | tail -50
```

**Step 5: Check Prometheus**
```bash
curl 'http://localhost:9090/api/v1/query?query=test_startup'
```

Should return data with `test_startup` metric.

**Step 6: Grafana dashboards**
http://localhost:3000 - should show metrics

---

## Success Criteria

- [ ] Code matches v0.31 example exactly (no custom endpoint logic)
- [ ] Uses `OTEL_EXPORTER_OTLP_ENDPOINT` environment variable
- [ ] OTEL Collector receives OTLP metrics from app
- [ ] Prometheus scrapes Collector's Prometheus exporter
- [ ] Grafana queries show data
- [ ] No compilation errors about `opentelemetry-prometheus`
- [ ] Logs show "✓ Metrics initialized (OTLP/HTTP)"

---

## Anti-Patterns to Avoid

### ❌ WRONG (v0.24 patterns)
```rust
// DO NOT USE THESE:
use opentelemetry_prometheus::*;  // ❌ Crate removed
.with_endpoint("http://prometheus:9090/...")  // ❌ Manual endpoint
Protocol::HttpBinary // ❌ in wrong context
```

### ✅ CORRECT (v0.31 from actual examples)
```rust
use opentelemetry_otlp::{MetricExporter, Protocol};

let exporter = MetricExporter::builder()
    .with_http()
    .with_protocol(Protocol::HttpBinary)
    .build()?;  // Uses OTEL_EXPORTER_OTLP_ENDPOINT env var
```

---

## Why This Will Work

1. **Code directly from v0.31 examples** - not AI inference
2. **Standard OTLP architecture** - app → collector → prometheus
3. **No deprecated crates** - only current v0.31 APIs
4. **Environment-driven config** - follows OpenTelemetry standards
5. **Incremental testing** - verify each layer independently

---

## Implementation Checklist

- [ ] Clone `/tmp/otel-rust` to reference examples
- [ ] Copy `init_metrics()` pattern from `basic-otlp-http/src/main.rs`
- [ ] Update `botticelli_core/src/observability.rs` 
- [ ] Create `otel-collector-config.yaml`
- [ ] Update `docker-compose.observability.yml` with collector
- [ ] Update `prometheus.yml` to scrape collector
- [ ] Set `OTEL_EXPORTER_OTLP_ENDPOINT` in Containerfile
- [ ] Remove any `opentelemetry-prometheus` references
- [ ] Run `cargo check` - zero warnings
- [ ] Run `just bot-build`
- [ ] Test OTLP endpoint: `curl http://localhost:4318/v1/metrics`
- [ ] Restart: `just bot-restart`
- [ ] Verify logs show metrics initialization
- [ ] Query Prometheus for `test_startup` metric
- [ ] Check Grafana dashboards

---

## References

- **Primary Source:** `/tmp/otel-rust/opentelemetry-otlp/examples/basic-otlp-http/src/main.rs`
- [OpenTelemetry OTLP v0.31](https://docs.rs/opentelemetry-otlp/0.31.0/)
- [OTEL Collector Docs](https://opentelemetry.io/docs/collector/)
- [Prometheus OTLP](https://prometheus.io/docs/prometheus/latest/feature_flags/#otlp-receiver)

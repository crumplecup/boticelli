# Botticelli Observability Dashboards

Complete guide to monitoring Botticelli with traces, metrics, and dashboards.

## Quick Start

### 1. Start the Full Observability Stack

```bash
# With Podman
podman-compose -f docker-compose.observability.yml up -d

# With Docker
docker-compose -f docker-compose.observability.yml up -d
```

This starts:
- **Jaeger** (traces): http://localhost:16686
- **Prometheus** (metrics): http://localhost:9090
- **Grafana** (dashboards): http://localhost:3000
- **PostgreSQL** (database): localhost:5433

### 2. Access Grafana

1. Open http://localhost:3000
2. Login: `admin` / `admin` (change on first login)
3. Navigate to **Dashboards** → **Botticelli Overview**

### 3. Run Your Bot with Tracing

```bash
# Bot server
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 \
cargo run --release -p botticelli_server --bin bot-server --features otel-otlp

# Actor server
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 \
cargo run --release -p botticelli_actor --bin actor-server --features otel-otlp,discord
```

Traces will flow to Jaeger, and you'll see them in Grafana!

## What You Get Out of the Box

### Trace Visualization (Jaeger)
- Request traces with timing breakdowns
- Service dependency maps
- Error traces highlighted

### Basic Metrics (Jaeger-derived)
- Trace ingestion rate
- Error rate (from trace spans)
- Service health

### Starter Dashboard (Grafana)
- Trace error rate gauge
- Traces received rate graph
- Recent traces panel

## Adding Application Metrics

The starter setup uses **trace-derived metrics** from Jaeger. For the specific metrics you want (LLM API failures, JSON parsing failures), you need to add **custom metrics** to your code.

### Step 1: Add OpenTelemetry Metrics Dependency

```toml
# Cargo.toml
[dependencies]
opentelemetry = { version = "0.28", features = ["metrics"] }
opentelemetry-otlp = { version = "0.31", features = ["metrics"], optional = true }
opentelemetry_sdk = { version = "0.28", features = ["metrics"] }
```

### Step 2: Initialize Metrics in Your Code

Add to `crates/botticelli_narrative/src/lib.rs`:

```rust
use opentelemetry::metrics::{Counter, Histogram, Meter};
use opentelemetry::{global, KeyValue};
use std::sync::OnceLock;

static METRICS: OnceLock<BotMetrics> = OnceLock::new();

pub struct BotMetrics {
    pub llm_requests: Counter<u64>,
    pub llm_errors: Counter<u64>,
    pub json_parse_attempts: Counter<u64>,
    pub json_parse_failures: Counter<u64>,
    pub narrative_duration: Histogram<f64>,
}

impl BotMetrics {
    fn init() -> Self {
        let meter = global::meter("botticelli");
        
        Self {
            llm_requests: meter
                .u64_counter("llm_requests_total")
                .with_description("Total LLM API requests")
                .init(),
            
            llm_errors: meter
                .u64_counter("llm_errors_total")
                .with_description("Failed LLM API requests")
                .init(),
            
            json_parse_attempts: meter
                .u64_counter("json_parse_attempts_total")
                .with_description("JSON parse attempts")
                .init(),
            
            json_parse_failures: meter
                .u64_counter("json_parse_failures_total")
                .with_description("JSON parse failures")
                .init(),
            
            narrative_duration: meter
                .f64_histogram("narrative_duration_seconds")
                .with_description("Narrative execution duration")
                .init(),
        }
    }
    
    pub fn get() -> &'static Self {
        METRICS.get_or_init(Self::init)
    }
}
```

### Step 3: Instrument Your Code

**LLM API Calls** (in `botticelli_models`):

```rust
// In generate() method
let metrics = BotMetrics::get();

metrics.llm_requests.add(1, &[
    KeyValue::new("model", model.to_string()),
]);

match result {
    Ok(response) => response,
    Err(e) => {
        metrics.llm_errors.add(1, &[
            KeyValue::new("model", model.to_string()),
            KeyValue::new("error_type", error_type(&e)),
        ]);
        return Err(e);
    }
}
```

**JSON Parsing** (in `botticelli_narrative/src/extraction.rs`):

```rust
pub fn parse_json<T: DeserializeOwned>(json_str: &str) -> Result<T, Error> {
    let metrics = BotMetrics::get();
    
    metrics.json_parse_attempts.add(1, &[]);
    
    match serde_json::from_str(json_str) {
        Ok(value) => Ok(value),
        Err(e) => {
            metrics.json_parse_failures.add(1, &[
                KeyValue::new("error", "parse_error"),
            ]);
            Err(Error::from(e))
        }
    }
}
```

**Narrative Execution** (in `botticelli_narrative/src/executor.rs`):

```rust
pub async fn execute<N>(&self, narrative: &N) -> Result<NarrativeExecution>
where N: NarrativeProvider
{
    let start = std::time::Instant::now();
    let metrics = BotMetrics::get();
    
    let result = self.execute_impl(narrative).await;
    
    let duration = start.elapsed().as_secs_f64();
    metrics.narrative_duration.record(duration, &[
        KeyValue::new("narrative", narrative.name().to_string()),
        KeyValue::new("success", result.is_ok().to_string()),
    ]);
    
    result
}
```

### Step 4: Export Metrics

Add metrics export to your observability initialization (where you init tracing):

```rust
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::metrics::SdkMeterProvider;

pub fn init_observability() -> Result<()> {
    // Existing trace setup...
    
    // Add metrics
    let metrics_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint("http://localhost:4317");
    
    let meter_provider = opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_exporter(metrics_exporter)
        .with_period(std::time::Duration::from_secs(10))
        .build()?;
    
    global::set_meter_provider(meter_provider);
    
    Ok(())
}
```

### Step 5: Create Advanced Dashboards

Once metrics are flowing, create dashboards in Grafana for:

**LLM API Health Dashboard**:
```promql
# Error rate
rate(llm_errors_total[5m]) / rate(llm_requests_total[5m]) * 100

# Request rate by model
sum(rate(llm_requests_total[1m])) by (model)

# P95 latency (from spans)
histogram_quantile(0.95, rate(narrative_duration_seconds_bucket[5m]))
```

**JSON Parsing Dashboard**:
```promql
# Failure rate
rate(json_parse_failures_total[5m]) / rate(json_parse_attempts_total[5m]) * 100

# Failure count by error type
sum(rate(json_parse_failures_total[1m])) by (error)
```

**Narrative Performance Dashboard**:
```promql
# Execution time by narrative
histogram_quantile(0.95, sum(rate(narrative_duration_seconds_bucket[5m])) by (le, narrative))

# Success rate
sum(rate(narrative_duration_seconds_count{success="true"}[5m])) / sum(rate(narrative_duration_seconds_count[5m]))
```

## Dashboard Examples

### Creating a Custom Dashboard

1. Go to http://localhost:3000
2. Click **Dashboards** → **New** → **New Dashboard**
3. Click **Add visualization**
4. Select **Prometheus** data source
5. Enter PromQL query (see examples above)
6. Configure visualization type (Graph, Gauge, Stat, etc.)
7. Click **Save**

### Example: LLM Error Rate Panel

```json
{
  "title": "LLM API Error Rate",
  "targets": [
    {
      "expr": "(rate(llm_errors_total[5m]) / rate(llm_requests_total[5m])) * 100",
      "legendFormat": "{{model}}"
    }
  ],
  "type": "timeseries",
  "fieldConfig": {
    "defaults": {
      "unit": "percent",
      "color": { "mode": "thresholds" },
      "thresholds": {
        "steps": [
          { "value": 0, "color": "green" },
          { "value": 5, "color": "yellow" },
          { "value": 10, "color": "red" }
        ]
      }
    }
  }
}
```

## Alerting

### Set Up Alerts in Grafana

1. Create alert rules based on metrics
2. Example: Alert when LLM error rate > 10%

```yaml
# Alert condition
expr: (rate(llm_errors_total[5m]) / rate(llm_requests_total[5m])) * 100 > 10

# For: 5m (sustained for 5 minutes)
```

3. Configure notification channels (email, Slack, Discord)

## Current Limitations

**Without custom metrics** (what you have now):
- ✅ Individual trace inspection
- ✅ Basic trace volume/error stats
- ❌ No aggregated metrics per operation type
- ❌ No custom business metrics
- ❌ Limited historical analysis

**With custom metrics** (after instrumentation):
- ✅ All the above
- ✅ Real-time dashboards
- ✅ Alerting on thresholds
- ✅ Historical trend analysis
- ✅ Service-level objectives (SLOs)

## Next Steps

1. **Now**: Use the starter setup to explore Jaeger traces in Grafana
2. **Phase 1**: Add basic metrics counters (5-10 key metrics)
3. **Phase 2**: Build dashboards for those metrics
4. **Phase 3**: Set up alerting rules
5. **Phase 4**: Add advanced metrics (histograms, exemplars)

## Troubleshooting

### No traces showing up in Grafana
- Check OTEL_EXPORTER_OTLP_ENDPOINT is set correctly
- Verify Jaeger is running: `podman logs botticelli-jaeger`
- Check network: `curl http://localhost:4317`

### No metrics in Prometheus
- Metrics export requires custom instrumentation (see Step 2-4 above)
- Check Prometheus targets: http://localhost:9090/targets
- Verify Jaeger metrics endpoint: `curl http://localhost:14269/metrics`

### Grafana can't connect to datasources
- Check containers are on same network
- Verify datasource URLs use container names (`http://prometheus:9090`)
- Check health: `podman-compose ps`

## Resources

- [OpenTelemetry Rust SDK](https://docs.rs/opentelemetry/)
- [Prometheus PromQL](https://prometheus.io/docs/prometheus/latest/querying/basics/)
- [Grafana Dashboards](https://grafana.com/docs/grafana/latest/dashboards/)
- [Jaeger Documentation](https://www.jaegertracing.io/docs/)

---

**Pro Tip**: Start with trace-only observability (what you have), then gradually add metrics as you identify bottlenecks. Don't try to instrument everything at once!

# Trace-Based Observability Strategy

**Status:** Planning
**Date:** 2025-11-30
**Context:** Metrics pipeline blocked, but Jaeger traces working. Pivot to trace-based observability.

## Problem Statement

We've spent significant time trying to get Prometheus metrics working with OpenTelemetry v0.31, hitting repeated API compatibility issues. Meanwhile, **Jaeger traces are working perfectly** and already instrument our actor and narrative execution.

**Key insight:** Traces contain rich temporal and contextual data that can provide similar observability to metrics, especially for:
- Actor execution patterns
- Narrative success/failure rates
- LLM API call durations and errors
- JSON parsing failures
- Bottleneck identification

## Strategy: Leverage Existing Traces

Instead of fighting metrics, use what's already working - our comprehensive tracing instrumentation.

### What Traces Already Capture

Our existing `#[instrument]` attributes capture:

1. **Actor execution:**
   - Span: `actor::execute`
   - Fields: `actor_name`, `skill`, error conditions
   - Duration: Built-in span timing

2. **Narrative execution:**
   - Span: `narrative::execute_impl`
   - Fields: `narrative_name`, `act_count`, composition details
   - Duration: End-to-end narrative timing

3. **LLM API calls:**
   - Span: `gemini::generate_content`
   - Fields: `model`, token counts, errors
   - Duration: API latency

4. **JSON parsing:**
   - Span: `parse_json` (if instrumented)
   - Fields: success/failure, error messages
   - Duration: Parsing time

### Phase 1: Jaeger Query Dashboard

**Goal:** Create a simple web interface to query common patterns from Jaeger.

**Approach:** Use Jaeger's HTTP API to query traces and build dashboards.

**Implementation:**
```rust
// Simple dashboard queries
- "Show all actor executions in last hour"
- "Show failed narrative executions"
- "Show LLM API calls > 5 seconds"
- "Show JSON parse errors"
```

**Benefits:**
- Uses existing working infrastructure
- No new dependencies
- Jaeger UI already has powerful filtering

### Phase 2: Span Metrics from Traces

**Goal:** Generate metrics FROM traces using span data.

**Approach:** 
- Query Jaeger traces programmatically
- Aggregate span data (counts, durations, errors)
- Display in simple dashboard

**Metrics to derive:**
```
- actor_execution_count (by actor_name, status)
- narrative_execution_duration (p50, p95, p99)
- llm_api_call_duration (by model)
- llm_api_error_rate
- json_parse_error_rate
```

**Implementation:**
```rust
// Pseudocode
let traces = jaeger_client.query_traces(
    service: "actor-server",
    operation: "actor::execute",
    lookback: "1h"
)?;

let success_count = traces.iter()
    .filter(|t| !t.has_error())
    .count();

let error_count = traces.len() - success_count;
let error_rate = error_count as f64 / traces.len() as f64;
```

### Phase 3: Trace Analytics Dashboard

**Goal:** Rich analytics dashboard powered by trace data.

**Features:**
- Timeline view of actor executions
- Heatmap of narrative duration by type
- Error rate trends over time
- Bottleneck identification (slowest spans)
- Dependency graph (which actors call which narratives)

**Technology options:**
- Simple HTML/JS dashboard querying Jaeger API
- Use existing Jaeger UI with saved queries
- Grafana with Jaeger data source (already configured!)

## Recommendation: Start with Grafana + Jaeger

**Why:** We already have both running in our observability stack!

**Implementation:**

1. **Grafana already has Jaeger data source configured**
   - Check: http://localhost:3000/connections/datasources
   - Should see Jaeger at http://jaeger:16686

2. **Create Trace Query Dashboard in Grafana:**
   - Use "Explore" tab with Jaeger data source
   - Build queries for common patterns
   - Save as dashboard panels

3. **Example queries in Grafana:**
   ```
   Service: actor-server
   Operation: actor::execute
   Tags: actor_name=Content Generator
   ```

4. **Dashboard panels:**
   - Table: Recent actor executions (last 100)
   - Graph: Actor execution duration over time
   - Table: Failed executions with error messages
   - Graph: Narrative execution counts by type

### Immediate Next Steps

1. **Verify Grafana-Jaeger connection:**
   ```bash
   curl http://localhost:3000/api/datasources
   ```

2. **Create first dashboard in Grafana UI:**
   - Navigate to Explore
   - Select Jaeger data source
   - Query: service="actor-server"
   - Save useful queries as dashboard

3. **Add trace links to logs:**
   - Ensure trace IDs in structured logs
   - Clickable links to Jaeger UI

## Success Criteria

- [ ] View actor execution history in Grafana
- [ ] Identify slow narrative executions
- [ ] Track error rates from trace data
- [ ] One-click jump from Grafana to detailed trace in Jaeger
- [ ] Dashboard updates in real-time as actors run

## Why This Works Better Than Metrics

1. **Already implemented:** Traces exist and work
2. **Richer context:** Traces show causation, not just correlation
3. **No new dependencies:** Uses existing stack
4. **Better for debugging:** Click through to full trace detail
5. **Async-friendly:** Traces naturally handle async operations

## Metrics Still Useful For

- High-cardinality aggregations (1000s of metrics)
- Real-time alerting (Prometheus AlertManager)
- Resource monitoring (CPU, memory, disk)

**Verdict:** Traces can provide 80% of what we need for actor/narrative observability. Defer metrics until we actually need them.

## References

- [Jaeger HTTP API](https://www.jaegertracing.io/docs/1.21/apis/#http-json-internal)
- [Grafana Jaeger Data Source](https://grafana.com/docs/grafana/latest/datasources/jaeger/)
- Existing: `OBSERVABILITY_SETUP.md`
- Related: `METRICS_GRAFANA_FIX.md` (blocked on OpenTelemetry v0.31 issues)

# Observability Dashboards Guide

## Overview

This guide shows how to use Jaeger's UI to monitor actors and narratives in the Botticelli system. Since we use trace-based observability, all our insights come from spans with rich attributes.

## Accessing Jaeger

1. Ensure containers are running: `just bot-up`
2. Open Jaeger UI: http://localhost:16686
3. Select service: `actor-server`

## Dashboard Queries

### Actor Performance

**Query: All Actor Executions**
- Service: `actor-server`
- Operation: `execute`
- Tags: `actor_name=*`

**Query: Specific Actor**
- Service: `actor-server`
- Operation: `execute`
- Tags: `actor_name="Content Generator"`

**Key Metrics to Watch:**
- Duration: How long each actor execution takes
- Error status: Look for `error=true` tag
- Frequency: How often actors execute

### Narrative Execution

**Query: All Narrative Executions**
- Service: `actor-server`
- Operation: `execute_impl_with_multi`
- Tags: `narrative_name=*`

**Query: Specific Narrative**
- Service: `actor-server`
- Operation: `execute_impl_with_multi`
- Tags: `narrative_name="batch_generate"`

**Key Span Attributes:**
- `narrative_name`: Which narrative ran
- `act_count`: How many acts in the composition
- `current_act`: Which act is executing
- `act_name`: Name of the current act

### Act-Level Detail

**Query: Individual Acts**
- Service: `actor-server`
- Operation: `execute_impl`
- Tags: `act_name=*`

**Key Attributes:**
- `act_name`: The specific act
- `act_type`: single_turn, multi_turn, etc.
- `context_size`: Token count in context
- `max_tokens`: Generation limit

### Error Analysis

**Query: Failed Executions**
- Service: `actor-server`
- Tags: `error=true`

**Common Error Patterns:**
- Actor execution failures (skill errors)
- Narrative parsing issues
- LLM API errors
- Rate limit hits

## Understanding Trace Structure

### Typical Trace Hierarchy

```
actor.execute (actor_name="Content Generator")
  ├─ execute_skill_with_retry (skill="narrative_execution")
  │   └─ narrative_execution.execute
  │       └─ executor.execute_impl_with_multi (narrative_name="batch_generate")
  │           ├─ executor.execute_impl (act_name="feature")
  │           │   └─ gemini.generate
  │           ├─ executor.execute_impl (act_name="tutorial")
  │           │   └─ gemini.generate
  │           └─ executor.execute_impl (act_name="social")
  │               └─ gemini.generate
```

### Span Attributes Reference

**Actor Spans:**
- `actor_name`: Name from config
- `skill`: Which skill executed
- `recoverable`: Can retry on failure
- `attempt`: Retry attempt number

**Narrative Spans:**
- `narrative_name`: Narrative identifier
- `narrative_path`: TOML file path
- `act_count`: Number of acts
- `current_act`: Current position in sequence

**Act Spans:**
- `act_name`: Act identifier
- `act_type`: Execution pattern
- `context_size`: Input token count
- `max_tokens`: Output limit
- `temperature`: Sampling parameter

**LLM Spans:**
- `model`: Model identifier
- `provider`: gemini/anthropic
- `response_tokens`: Generated token count
- `finish_reason`: Why generation stopped

## Common Monitoring Scenarios

### 1. Actor Health Check

**Goal:** Verify all actors are executing regularly

**Steps:**
1. Set time range to last hour
2. Search for operation: `execute`
3. Group by `actor_name` tag
4. Look for:
   - Regular execution intervals
   - No persistent errors
   - Reasonable durations

### 2. Narrative Bottleneck Detection

**Goal:** Find slow acts in narratives

**Steps:**
1. Find a narrative execution trace
2. Expand the full trace tree
3. Compare act durations
4. Identify outliers
5. Check `context_size` and `response_tokens` for large values

### 3. Error Root Cause Analysis

**Goal:** Understand why an execution failed

**Steps:**
1. Query for `error=true`
2. Open failed trace
3. Find the deepest span with error
4. Check span logs for error messages
5. Review parent spans for context

### 4. Rate Limit Monitoring

**Goal:** Track API rate limiting

**Steps:**
1. Search for operation: `generate`
2. Look for error spans
3. Check logs for "rate limit" messages
4. Correlate with actor execution patterns

## Tips and Best Practices

### Effective Time Ranges

- **Real-time monitoring:** Last 15 minutes
- **Debugging:** Last hour
- **Pattern analysis:** Last 24 hours
- **Historical review:** Custom range

### Using Trace Comparison

1. Find a successful trace
2. Find a failed trace for same operation
3. Use "Compare" feature
4. Look for differences in:
   - Duration
   - Child span count
   - Attribute values

### Search Tips

- Use wildcards: `actor_name=*Generator*`
- Combine tags: `error=true narrative_name=batch_generate`
- Sort by duration to find slowest traces
- Use "Find Similar Traces" for patterns

### Performance Baselines

Establish normal ranges:
- Actor execution: 30-120 seconds typical
- Single act: 5-30 seconds typical
- LLM generation: 2-15 seconds typical

Flag when:
- Duration > 2x baseline
- Error rate > 5%
- Missing expected executions

## Integration with Development

### Adding New Instrumentation

When adding spans, include:
- Operation name (consistent naming)
- Key identifying attributes
- Context size when relevant
- Error status and messages

### Debugging Workflow

1. Reproduce issue locally with `just bot-up`
2. Check Jaeger for traces around issue time
3. Identify failing span
4. Check span attributes and logs
5. Add more instrumentation if needed
6. Verify fix in Jaeger

### Documentation Workflow

When adding features:
1. Add appropriate instrumentation
2. Test that spans appear in Jaeger
3. Document expected span structure
4. Add to this guide if monitoring pattern changes

## Limitations

**What Jaeger Doesn't Show:**
- Aggregate metrics over time (use Prometheus/Grafana when metrics re-enabled)
- Long-term trends
- Percentiles across traces
- Custom visualizations

**Workarounds:**
- Export trace data for analysis
- Use Jaeger API for programmatic queries
- Sample traces for representative patterns

## Future Enhancements

When metrics are re-enabled:
1. Grafana dashboards for aggregates
2. Alert rules for anomalies
3. SLO tracking
4. Correlation with traces

## Support

For issues:
1. Check span instrumentation in code
2. Verify OTEL_EXPORTER_OTLP_ENDPOINT
3. Check container logs: `podman logs botticelli-actor-server`
4. Verify Jaeger is receiving traces: Check Jaeger UI health


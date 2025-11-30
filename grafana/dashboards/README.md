# Botticelli Grafana Dashboards

This directory contains pre-built Grafana dashboards for monitoring Botticelli bot operations.

## Available Dashboards

### 1. LLM API Health (`llm-api-health.json`)

**Purpose:** Monitor LLM API performance and reliability

**Key Metrics:**
- **Error Rate Gauge** - Overall API error rate (green < 5%, yellow < 10%, red > 10%)
- **Request Rate by Model** - Requests/second per model (track which models are most used)
- **P95 Latency Gauge** - 95th percentile response time (green < 1s, yellow < 2s, red > 2s)
- **Latency Percentiles** - P50, P95, P99 trends over time
- **Errors by Type** - Breakdown of error types (rate_limit, auth, network, timeout, etc.)
- **Token Usage Rate** - Token consumption per model (cost tracking)

**Use Cases:**
- Detect API rate limiting issues
- Identify slow models
- Track cost via token usage
- Diagnose auth/network problems

**Alerts to Consider:**
- Error rate > 10%
- P95 latency > 2s
- Rate limit errors > 1/min

---

### 2. Narrative Performance (`narrative-performance.json`)

**Purpose:** Monitor narrative execution and JSON parsing reliability

**Key Metrics:**
- **Success Rate Gauge** - Narrative execution success rate (green > 95%, yellow > 90%, red < 90%)
- **JSON Parse Failure Rate** - Percentage of failed JSON extractions
- **Execution Duration P95** - Time to complete narratives and acts
- **JSON Parsing Results** - Success/failure counts per narrative
- **Execution Rate** - Narratives executed per second

**Use Cases:**
- Identify problematic narratives
- Track JSON parsing reliability (schema issues)
- Monitor execution performance
- Detect narrative bottlenecks

**Alerts to Consider:**
- Success rate < 90%
- JSON failure rate > 10%
- Execution duration spike > 2x baseline

---

### 3. Bot Health (`bot-health.json`)

**Purpose:** Monitor bot execution and content pipeline health

**Key Metrics:**
- **Failure Rate Gauge** - Bot execution failure rate (green < 5%, yellow < 10%, red > 10%)
- **Queue Depth Gauge** - Pending content items (green < 10, yellow < 50, red > 50)
- **Time Since Last Success** - Seconds since last successful execution (green < 5m, yellow < 10m, red > 10m)
- **Execution Rate by Type** - Bot executions per second by type
- **Execution Duration P95** - Bot execution time
- **Content Pipeline Throughput** - Generated/curated/published content rates

**Use Cases:**
- Detect stuck bots (time since success)
- Monitor queue backlog
- Track content production rates
- Identify slow bot types

**Alerts to Consider:**
- Time since success > 10 minutes
- Queue depth > 50
- Failure rate > 10%
- Pipeline throughput = 0 for > 5 minutes

---

## Dashboard Access

After starting the observability stack:

```bash
podman-compose -f docker-compose.observability.yml up -d
```

**Grafana URL:** http://localhost:3000  
**Default Credentials:** admin / admin

Dashboards will be automatically loaded under the **Botticelli** folder.

---

## Customization

All dashboards are editable. Common customizations:

### Change Time Range
Default: Last 15 minutes (5s refresh)
- Click time picker (top right)
- Select custom range or relative time

### Add Panels
1. Click "Add panel" button
2. Use Prometheus data source
3. Write PromQL query (see examples below)
4. Configure visualization type

### Save Changes
Changes are saved to `/var/lib/grafana` in the container (persistent volume).

---

## PromQL Query Examples

### LLM Metrics

```promql
# Error rate by model
rate(llm_errors[5m]) / rate(llm_requests[5m]) * 100

# Requests per second
sum(rate(llm_requests[1m])) by (model)

# P95 latency
histogram_quantile(0.95, rate(llm_duration_bucket[5m]))

# Total errors by type
sum(increase(llm_errors[1h])) by (error_type)

# Token usage per model
sum(rate(llm_tokens[1m])) by (model)
```

### Narrative Metrics

```promql
# Success rate
sum(rate(narrative_executions{success="true"}[5m])) / sum(rate(narrative_executions[5m])) * 100

# JSON parsing failure rate
rate(narrative_json_failures[5m]) / (rate(narrative_json_success[5m]) + rate(narrative_json_failures[5m])) * 100

# Slowest narratives
topk(5, histogram_quantile(0.95, rate(narrative_duration_bucket[5m])))

# JSON failures by narrative
sum(rate(narrative_json_failures[1m])) by (narrative_name)
```

### Bot Metrics

```promql
# Bot failure rate
rate(bot_failures[5m]) / rate(bot_executions[5m]) * 100

# Queue depth (current value)
bot_queue_depth

# Time since last success (current value)
bot_time_since_success

# Pipeline efficiency (curated / generated)
rate(pipeline_curated[5m]) / rate(pipeline_generated[5m]) * 100
```

---

## Troubleshooting

### Dashboards Not Loading

1. Check Grafana logs:
   ```bash
   podman logs botticelli-grafana
   ```

2. Verify dashboard files are mounted:
   ```bash
   podman exec botticelli-grafana ls -la /var/lib/grafana/dashboards
   ```

3. Check provisioning config:
   ```bash
   podman exec botticelli-grafana cat /etc/grafana/provisioning/dashboards/dashboards.yml
   ```

### No Data in Panels

1. Check Prometheus targets are UP:
   - Visit http://localhost:9090/targets
   - All targets should show "UP" status

2. Verify metrics are being exported:
   ```bash
   curl http://localhost:9090/api/v1/label/__name__/values | grep llm
   ```

3. Check bot is running with OTLP export:
   ```bash
   OTEL_EXPORTER=otlp cargo run --features otel-otlp
   ```

### Dashboard UID Conflicts

If you see "Dashboard with UID already exists" errors:
1. Edit the JSON file
2. Change the `"uid"` field to a unique value
3. Restart Grafana

---

## Dashboard Development

To create new dashboards:

1. Build dashboard in Grafana UI
2. Export JSON: Settings → JSON Model → Copy JSON
3. Save to `grafana/dashboards/new-dashboard.json`
4. Update this README with description
5. Restart Grafana to auto-load

**Tip:** Set `"id": null` in JSON to let Grafana assign IDs on import.

---

## Alerting (Future Enhancement)

To add alerts to these dashboards:

1. Configure alert rules in Grafana UI
2. Set up notification channels (email, Slack, Discord)
3. Define thresholds and conditions
4. Export alert rules to `grafana/provisioning/alerting/`

See [Grafana Alerting Documentation](https://grafana.com/docs/grafana/latest/alerting/) for details.

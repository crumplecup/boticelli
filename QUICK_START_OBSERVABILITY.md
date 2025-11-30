# Quick Start: Full Observability Stack

## What You Get

- **Jaeger**: Distributed tracing (spans, traces)
- **Prometheus**: Metrics collection (counters, histograms, gauges)
- **Grafana**: Dashboards for both traces and metrics

## Start the Stack

### Using Podman (Recommended)

```bash
# Start observability stack
podman-compose -f docker-compose.jaeger-only.yml up -d

# Verify all containers are running
podman ps

# Should see:
# - botticelli-jaeger
# - botticelli-prometheus
# - botticelli-grafana
```

### Using Docker

```bash
# Start observability stack
docker-compose -f docker-compose.jaeger-only.yml up -d

# Verify
docker ps
```

## Start Your Bot Server

### Configure Environment (.env file)

Create a `.env` file in the project root with these required variables:

```bash
# Discord (required)
DISCORD_TOKEN=your_bot_token_here

# LLM Provider (at least one required)
GEMINI_API_KEY=your_gemini_key_here
# or
ANTHROPIC_API_KEY=your_anthropic_key_here

# Observability (required for metrics/traces)
OTEL_EXPORTER=otlp
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
PROMETHEUS_ENDPOINT=0.0.0.0:9464

# Database (optional - has deployment-aware defaults)
# Local dev default: postgresql://postgres:postgres@localhost:5432/botticelli
# Container default: postgresql://botticelli:botticelli@postgres:5432/botticelli
# DATABASE_URL=postgresql://custom:custom@host:5432/db

# Deployment environment (optional - set automatically in containers)
# DEPLOYMENT_ENV=container  # Uses container database defaults
# DEPLOYMENT_ENV=local      # Uses local dev database defaults (default)
```

**Note on Database Configuration:**
- Local runs default to `postgresql://postgres:postgres@localhost:5432/botticelli`
- Container runs default to `postgresql://botticelli:botticelli@postgres:5432/botticelli`
- Container uses PostgreSQL from observability stack (`docker-compose.observability.yml`)
- Explicitly set `DATABASE_URL` to override defaults
- See [DATABASE_SYNC_STRATEGY.md](DATABASE_SYNC_STRATEGY.md) for syncing between environments
- See [DEPLOYMENT_CONFIG.md](DEPLOYMENT_CONFIG.md) for configuration details

### Run the Bot Server

The bot server automatically loads `.env` on startup.

**Using just with Podman (recommended):**
```bash
just bot-build    # Build container image (includes migrations)
just bot-up       # Start bot server container
just bot-logs     # View bot server logs
```

**Using just locally (without container):**
```bash
just run-actor-server
```

**Using cargo directly:**
```bash
cargo run --bin actor-server --release --features discord
```

**Note:** Container deployment is recommended because:
- Includes all runtime dependencies (TOML configs, narratives, migrations)
- Connects to observability stack via shared `botticelli` network
- Automatically runs database migrations on startup
- Production-like environment

## Access the UIs

### Jaeger (Traces)
- URL: http://localhost:16686
- What to see:
  - Service: Select "botticelli-actor-server"
  - Operations: LLM API calls, narrative executions
  - Traces: Click on any trace to see span details

### Prometheus (Metrics)
- URL: http://localhost:9090
- What to check:
  - Status > Targets: Verify "actor-server" is UP
  - Graph tab, try queries:
    - `bot_executions_total` - Total bot executions by type
    - `bot_failures_total` - Failed executions by bot type
    - `rate(bot_executions_total[5m])` - Execution rate per 5 minutes
    - `bot_duration_seconds` - Execution duration histogram

### Grafana (Dashboards)
- URL: http://localhost:3000
- Login: `admin` / `admin`
- Pre-configured dashboards:
  - **Botticelli Overview**: Bot execution counts, durations, failure rates
  - **Bot Health**: Overall system health (Phase 1: bot-level metrics only)
  
**Note:** Dashboard panels for LLM and narrative metrics will show "No data" until Phase 2/3 implementation. Currently available:
  - Bot execution counts by type
  - Bot execution duration histograms
  - Bot failure rates
  - Queue depth gauges

## Verify Everything Works

### 1. Check Metrics Endpoint

```bash
curl http://localhost:9464/metrics | grep -E "^bot_"
```

Should see output like (after actors execute at least once):
```
# HELP bot_executions_total Total bot executions
# TYPE bot_executions_total counter
bot_executions_total{bot_type="Content Generator"} 5
bot_executions_total{bot_type="Content Curator"} 3
# HELP bot_duration_seconds Bot execution duration
# TYPE bot_duration_seconds histogram
bot_duration_seconds_bucket{bot_type="Content Generator",le="0.5"} 2
bot_duration_seconds_sum{bot_type="Content Generator"} 4.23
bot_duration_seconds_count{bot_type="Content Generator"} 5
```

**Note:** Currently only bot-level metrics are implemented (Phase 1). LLM and narrative metrics coming in future phases. See [METRICS_GRAFANA_FIX.md](METRICS_GRAFANA_FIX.md) for implementation status.

### 2. Check Prometheus Scraping

1. Open http://localhost:9090/targets
2. Find "actor-server" job
3. Status should be "UP" (green)
4. Last scrape should be recent (< 30s ago)

### 3. Check Grafana Data Sources

1. Open http://localhost:3000
2. Go to Configuration > Data Sources
3. Should see:
   - **Prometheus** (default) - Status: OK
   - **Jaeger** - Status: OK

### 4. View a Dashboard

1. In Grafana, go to Dashboards
2. Open "LLM API Health"
3. Select time range: Last 15 minutes
4. You should see:
   - Request counts
   - Error rates (% failed)
   - Response time graphs
   - Token usage

If no data appears:
- Ensure bot server is running
- Trigger some activity (run a command)
- Wait 15-30 seconds for metrics to be scraped
- Refresh dashboard

## Troubleshooting

### No Metrics in Grafana

**Check 1: Is Prometheus scraping the bot server?**
```bash
# From Prometheus UI (localhost:9090/targets)
# actor-server should show State: UP
```

If DOWN:
```bash
# Test metrics endpoint directly
curl http://localhost:9464/metrics

# If this works but Prometheus can't reach it:
# - Podman users: Verify host.containers.internal resolves
# - Docker users: Edit prometheus.yml, change to host.docker.internal
```

**Check 2: Is bot server exposing metrics?**
```bash
# Check bot server logs
# Should see on startup:
# INFO: Prometheus metrics server listening on http://0.0.0.0:9464/metrics
```

If not:
```bash
# Verify environment variable is set
echo $PROMETHEUS_ENDPOINT
# Should output: 0.0.0.0:9464
```

**Check 3: Is Grafana connected to Prometheus?**
```bash
# Grafana UI > Configuration > Data Sources > Prometheus
# Click "Test" button
# Should show: "Data source is working"
```

### Container Networking

**If bot-server is containerized** (using `just bot-up`) - **RECOMMENDED**:
- Bot server and observability stack share the `botticelli` network
- Bot connects to PostgreSQL at `postgres:5432` (container name)
- Bot connects to Jaeger at `jaeger:4317` (container name)
- Prometheus scrapes from `bot-server:9464` (container name)
- No networking issues - everything just works!

**If bot-server runs locally** (using `just run-actor-server` or `cargo run`):

1. Check if `host.containers.internal` works:
   ```bash
   podman exec botticelli-prometheus ping host.containers.internal
   ```

2. If ping fails, get your host IP:
   ```bash
   ip addr show | grep 'inet ' | grep -v 127.0.0.1
   ```

3. Update `prometheus.yml`:
   ```yaml
   - job_name: 'actor-server'
     static_configs:
       - targets: ['YOUR_HOST_IP:9464']  # Use IP from step 2
   ```

4. Restart Prometheus:
   ```bash
   podman-compose -f docker-compose.jaeger-only.yml restart prometheus
   ```

### Dashboard Shows "No Data"

**Fix 1: Adjust time range**
- In Grafana dashboard, top-right corner
- Change from "Last 1h" to "Last 5 minutes"
- Click refresh

**Fix 2: Trigger activity**
```bash
# Generate some metrics by triggering bot activity
# Wait 15-30 seconds
# Refresh dashboard
```

**Fix 3: Check metric names**
- Grafana dashboard queries might use old metric names
- Go to Prometheus UI (localhost:9090)
- Click "Graph"
- Start typing `llm_` - autocomplete shows available metrics
- Verify metric names match what dashboard queries use

## Metrics Reference

### LLM API Metrics

All metrics include labels for `provider` and `model`.

| Metric | Type | Description | Labels |
|--------|------|-------------|--------|
| `llm_requests_total` | Counter | Total API requests | provider, model |
| `llm_errors_total` | Counter | Failed requests | provider, model, error_type |
| `llm_duration_seconds` | Histogram | API call latency | provider, model |
| `llm_tokens_total` | Counter | Total tokens | model |
| `llm_tokens_prompt_total` | Counter | Prompt tokens | model |
| `llm_tokens_completion_total` | Counter | Completion tokens | model |

### Useful Queries

**Error rate (percentage)**:
```promql
100 * (
  rate(llm_errors_total[5m]) 
  / 
  rate(llm_requests_total[5m])
)
```

**Average response time**:
```promql
rate(llm_duration_seconds_sum[5m]) 
/ 
rate(llm_duration_seconds_count[5m])
```

**P95 latency** (requires histogram):
```promql
histogram_quantile(0.95, rate(llm_duration_seconds_bucket[5m]))
```

**Tokens per minute by model**:
```promql
sum by (model) (rate(llm_tokens_total[1m]) * 60)
```

**Requests by provider**:
```promql
sum by (provider) (rate(llm_requests_total[5m]))
```

## Next Steps

1. ✅ Run bot server with metrics enabled
2. ✅ Verify Prometheus is scraping (`/targets` page)
3. ✅ Open Grafana dashboards
4. ✅ Trigger some bot activity
5. ✅ Watch metrics populate in real-time

## Stopping the Stack

```bash
# Podman
podman-compose -f docker-compose.jaeger-only.yml down

# Keep data volumes:
podman-compose -f docker-compose.jaeger-only.yml down

# Delete data volumes too:
podman-compose -f docker-compose.jaeger-only.yml down -v
```

## Architecture Summary

### Containerized Setup (Recommended)

```
┌───────────────────────────────────────────────────────┐
│              botticelli network                       │
│                                                       │
│  ┌──────────────┐                                    │
│  │ bot-server   │                                    │
│  │ (container)  │                                    │
│  │              │─────OTLP 4317────▶┌──────────┐    │
│  │  Reads:      │                    │  Jaeger  │    │
│  │  - .env      │                    │  :16686  │    │
│  │  - *.toml    │                    └──────────┘    │
│  │  - narratives│                                    │
│  │              │                                    │
│  │  Runs:       │                    ┌──────────┐    │
│  │  - migrations│────connects to─────│PostgreSQL│    │
│  │  on startup  │    postgres:5432   │  :5432   │    │
│  │              │                    └──────────┘    │
│  │  Exposes:    │                                    │
│  │  - :9464     │                    ┌──────────┐    │
│  └──────────────┘                    │Prometheus│    │
│        │                             │  :9090   │    │
│        └────HTTP :9464──scrapes─────▶│          │    │
│                                      └────┬─────┘    │
│                                           │          │
│                                           ▼          │
│                                    ┌──────────┐      │
│                                    │ Grafana  │      │
│                                    │  :3000   │      │
│                                    └──────────┘      │
└───────────────────────────────────────────────────────┘
```

### Local Setup (Development)

```
┌─────────────┐                ┌─────────────────┐
│ bot-server  │                │ Docker/Podman   │
│  (local)    │                │   Containers    │
│             │                │                 │
│ Emits       │──OTLP:4317────▶│   Jaeger        │
│ traces      │                │                 │
│             │                │   Prometheus    │
│ Exposes     │──HTTP:9464────▶│   (scrapes via  │
│ metrics     │                │    host.        │
│             │                │    containers.  │
└─────────────┘                │    internal)    │
                               │                 │
                               │   Grafana       │
                               └─────────────────┘
```

## Documentation

- **OBSERVABILITY_SETUP.md**: Detailed setup guide
- **OBSERVABILITY_METRICS_JAEGER_ISSUE.md**: Why Jaeger alone isn't enough
- **grafana/dashboards/README.md**: Dashboard documentation
- **OBSERVABILITY_DASHBOARDS.md**: Dashboard design patterns

## Support

If you see errors or unexpected behavior:

1. Check logs:
   ```bash
   # Bot server logs (in terminal where it's running)
   
   # Container logs
   podman logs botticelli-prometheus
   podman logs botticelli-grafana
   podman logs botticelli-jaeger
   ```

2. Verify network connectivity:
   ```bash
   # From inside prometheus container
   podman exec botticelli-prometheus wget -O- http://host.containers.internal:9464/metrics
   ```

3. Check configuration:
   ```bash
   # Verify prometheus config
   podman exec botticelli-prometheus cat /etc/prometheus/prometheus.yml
   ```

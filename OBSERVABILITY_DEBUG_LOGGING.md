# Observability Debug Logging

## Overview

Comprehensive debug logging has been added throughout the metrics pipeline to diagnose issues with metrics collection and export.

## Logging Locations

### 1. Metrics Initialization (`botticelli/src/observability.rs`)

**Logs emitted:**
- `INFO: "Initializing metrics provider"` - Start of metrics setup
- `DEBUG: "Metrics exporter configuration"` - Shows configured exporter backend
- For Stdout exporter:
  - `INFO: "Using stdout metrics exporter (development mode)"`
  - `DEBUG: "Creating stdout metric exporter"`
  - `DEBUG: "Creating periodic reader for stdout exporter"`
  - `DEBUG: "Building meter provider with stdout reader"`
  - `DEBUG: "Setting global meter provider"`
  - `INFO: "Stdout metrics provider initialized successfully"`
- For OTLP exporter:
  - `INFO: "Using OTLP metrics exporter"` (with endpoint)
  - `INFO: "Metrics will be sent via OTLP HTTP binary protocol"` (with metrics_endpoint)
  - `DEBUG: "Building OTLP metric exporter"`
  - `DEBUG: "OTLP metric exporter built successfully"`
  - `DEBUG: "Building meter provider with periodic exporter"`
  - `DEBUG: "Setting global meter provider"`
  - `INFO: "OTLP metrics provider initialized successfully"`
  - `ERROR: "Failed to build OTLP metric exporter"` (on error)
- `INFO: "Metrics initialization complete"` - End of metrics setup

### 2. Instrument Creation (`botticelli_server/src/metrics.rs`)

**BotMetrics::new():**
- `DEBUG: "Getting global meter for botticelli_bots"`
- `DEBUG: "Building bot metrics instruments"`
- `DEBUG: "Created bot.executions counter"`
- `DEBUG: "Created bot.failures counter"`
- `DEBUG: "Created bot.duration histogram"`
- `DEBUG: "Created bot.queue_depth gauge"`
- `DEBUG: "Created bot.time_since_success gauge"`
- `DEBUG: "BotMetrics instruments created successfully"`

**ServerMetrics::new():**
- `INFO: "Initializing ServerMetrics"`
- `DEBUG: "Creating BotMetrics"`
- `DEBUG: "Creating NarrativeMetrics"`
- `DEBUG: "Creating PipelineMetrics"`
- `INFO: "ServerMetrics initialized successfully"`

### 3. Metrics Recording (`botticelli_server/src/metrics.rs`)

**BotMetrics::record_execution():**
- `DEBUG: "Recording bot execution metrics"` (with bot_type and duration_secs)
- `DEBUG: "Bot execution metrics recorded"`

**BotMetrics::record_failure():**
- `DEBUG: "Recording bot failure metric"` (with bot_type)
- `DEBUG: "Bot failure metric recorded"`

## Usage

### Enable Debug Logging

Set the `RUST_LOG` environment variable to see debug messages:

```bash
# See all debug messages
export RUST_LOG=debug

# See only metrics-related debug messages
export RUST_LOG=botticelli=debug,botticelli_server=debug

# See only observability initialization
export RUST_LOG=botticelli::observability=debug
```

### Typical Log Flow

For a successful metrics setup and execution, you should see:

```
INFO botticelli::observability: Initializing metrics provider
DEBUG botticelli::observability: Metrics exporter configuration exporter=Otlp { endpoint: "http://localhost:4317" }
INFO botticelli::observability: Using OTLP metrics exporter endpoint="http://localhost:4317"
INFO botticelli::observability: Metrics will be sent via OTLP HTTP binary protocol metrics_endpoint="http://localhost:4317/api/v1/otlp/v1/metrics"
DEBUG botticelli::observability: Building OTLP metric exporter
DEBUG botticelli::observability: OTLP metric exporter built successfully
DEBUG botticelli::observability: Building meter provider with periodic exporter
DEBUG botticelli::observability: Setting global meter provider
INFO botticelli::observability: OTLP metrics provider initialized successfully
INFO botticelli::observability: Metrics initialization complete
INFO botticelli_server::metrics: Initializing ServerMetrics
DEBUG botticelli_server::metrics: Creating BotMetrics
DEBUG botticelli_server::metrics: Getting global meter for botticelli_bots
DEBUG botticelli_server::metrics: Building bot metrics instruments
DEBUG botticelli_server::metrics: Created bot.executions counter
DEBUG botticelli_server::metrics: Created bot.failures counter
DEBUG botticelli_server::metrics: Created bot.duration histogram
DEBUG botticelli_server::metrics: Created bot.queue_depth gauge
DEBUG botticelli_server::metrics: Created bot.time_since_success gauge
DEBUG botticelli_server::metrics: BotMetrics instruments created successfully
DEBUG botticelli_server::metrics: Creating NarrativeMetrics
... (similar for other metrics)
INFO botticelli_server::metrics: ServerMetrics initialized successfully
DEBUG botticelli_server::metrics: Recording bot execution metrics bot_type="Content Generator" duration_secs=5.2
DEBUG botticelli_server::metrics: Bot execution metrics recorded
```

## Troubleshooting

### No Metrics Logs Appearing

If you don't see ANY metrics-related logs:

1. Check `RUST_LOG` is set to at least `info`
2. Verify observability feature is enabled: `cargo build --features observability`
3. Check that metrics are enabled in config: `ObservabilityConfig::new(...).with_metrics(true)`

### Metrics Initialized But Not Exported

If you see initialization logs but no data in Prometheus/Grafana:

1. Check for OTLP exporter errors in logs
2. Verify OTLP endpoint is accessible: `curl http://prometheus:9090/api/v1/otlp/v1/metrics`
3. Confirm Prometheus has `--web.enable-otlp-receiver` flag
4. Look for "Recording bot execution metrics" logs - if missing, bots aren't running
5. Check meter provider is set: Look for "Setting global meter provider" log

### Instruments Created But No Data Recorded

If you see "Created bot.executions counter" but never "Recording bot execution metrics":

1. Bot execution may be failing before metrics recording
2. Check actor-server logs for execution errors
3. Verify actors are scheduled and running (check actor execution logs)

## Next Steps

If metrics still don't work after checking logs:

1. Capture full logs with `RUST_LOG=trace`
2. Share relevant log excerpts showing where the flow breaks
3. Test with stdout exporter first: `OTEL_EXPORTER=stdout`
4. Verify basic OpenTelemetry setup with a minimal test

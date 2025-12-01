# Podman Containerization Strategy

## Overview

Strategy for containerizing the bot-server binary using Podman while maintaining connectivity with the observability stack (Prometheus, Grafana, Jaeger).

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ Podman Network: botticelli-net                              │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ bot-server   │  │ Prometheus   │  │  Grafana     │     │
│  │ :9090/metrics│→ │ :9090        │→ │  :3000       │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│         ↓                                                    │
│  ┌──────────────┐                                           │
│  │   Jaeger     │                                           │
│  │ :4317 (OTLP) │                                           │
│  │ :16686 (UI)  │                                           │
│  └──────────────┘                                           │
└─────────────────────────────────────────────────────────────┘
```

## Current State Issues

1. **Network Isolation**: Bot-server runs on host, observability stack in containers
2. **Prometheus Target**: Configured to scrape `host.containers.internal:9090` but bot-server on host uses `localhost:9090`
3. **No Container Image**: No Containerfile/Dockerfile for bot-server yet

## Solution: Multi-Stage Build with Podman

### Phase 1: Create Containerfile

```dockerfile
# Containerfile
FROM rust:1.83-slim as builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build release binary
RUN cargo build --release --bin actor-server --features discord

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /build/target/release/actor-server /app/actor-server

# Copy configuration files
COPY actor_server.toml /app/
COPY crates/botticelli_server/actors/*.toml /app/actors/
COPY crates/botticelli_narrative/narratives /app/narratives

# Expose metrics port
EXPOSE 9090

# Run as non-root user
RUN useradd -m -u 1000 botticelli
USER botticelli

CMD ["/app/actor-server"]
```

### Phase 2: Podman Compose Integration

Update `docker-compose.observability.yml`:

```yaml
services:
  bot-server:
    build:
      context: .
      dockerfile: Containerfile
    container_name: botticelli-bot-server
    networks:
      - botticelli-net
    ports:
      - "9090:9090"
    environment:
      - DISCORD_TOKEN=${DISCORD_TOKEN}
      - DATABASE_URL=${DATABASE_URL}
      - RUST_LOG=${RUST_LOG:-info}
      - OTEL_EXPORTER=${OTEL_EXPORTER:-otlp}
      - OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4317
    volumes:
      - ./.narrative_state:/app/.narrative_state
      - ./.actor_server_state.json:/app/.actor_server_state.json
    depends_on:
      - jaeger
      - prometheus
    restart: unless-stopped

  prometheus:
    # ... existing config, update targets:
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus

  # ... rest of services
```

Update `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'botticelli-bot-server'
    static_configs:
      - targets: ['bot-server:9090']  # Use container service name
```

### Phase 3: Just Recipes

```justfile
# Container Management
# ===================

# Build bot-server container image
build-container:
    @echo "🐋 Building bot-server container..."
    podman build -t botticelli-bot-server:latest -f Containerfile .

# Start full observability stack with bot-server
start-observability:
    @echo "🚀 Starting observability stack with bot-server..."
    podman-compose -f docker-compose.observability.yml up -d
    @echo "✅ Stack running:"
    @echo "   📊 Grafana: http://localhost:3000"
    @echo "   📈 Prometheus: http://localhost:9091"
    @echo "   🔍 Jaeger: http://localhost:16686"
    @echo "   🤖 Bot metrics: http://localhost:9090/metrics"

# Stop observability stack
stop-observability:
    @echo "🛑 Stopping observability stack..."
    podman-compose -f docker-compose.observability.yml down

# View bot-server logs
logs-bot-server:
    podman logs -f botticelli-bot-server

# Rebuild and restart bot-server only
restart-bot-server:
    @echo "🔄 Rebuilding and restarting bot-server..."
    just build-container
    podman-compose -f docker-compose.observability.yml up -d bot-server

# Shell into bot-server container
shell-bot-server:
    podman exec -it botticelli-bot-server /bin/bash

# Check bot-server metrics endpoint
check-metrics:
    @echo "📊 Checking metrics endpoint..."
    curl -s http://localhost:9090/metrics | head -20
```

## Development Workflow

### Initial Setup

```bash
# 1. Create .env with required variables
cp .env.example .env
# Edit .env with your DISCORD_TOKEN, etc.

# 2. Build container image
just build-container

# 3. Start full stack
just start-observability

# 4. Verify connectivity
just check-metrics
```

### Development Cycle

```bash
# Make code changes...

# Rebuild and restart bot-server
just restart-bot-server

# Watch logs
just logs-bot-server

# Check metrics are flowing
just check-metrics
```

## Network Troubleshooting

### Check Network Connectivity

```bash
# List networks
podman network ls

# Inspect network
podman network inspect botticelli-net

# Test connectivity between containers
podman exec botticelli-prometheus curl bot-server:9090/metrics
```

### Common Issues

**Issue**: Prometheus can't scrape bot-server
**Solution**: Verify network connectivity:
```bash
podman exec botticelli-prometheus ping bot-server
```

**Issue**: Bot-server can't reach Jaeger
**Solution**: Check OTLP endpoint:
```bash
podman exec botticelli-bot-server curl -v http://jaeger:4317
```

**Issue**: Container can't resolve DNS
**Solution**: Restart podman network:
```bash
podman-compose down
podman network rm botticelli-net
podman-compose up -d
```

## Benefits

1. **Network Isolation**: All services on same network
2. **Service Discovery**: Use container names as hostnames
3. **Reproducible**: Same environment everywhere
4. **Resource Limits**: Can set CPU/memory constraints
5. **Easy Cleanup**: `podman-compose down` removes everything

## Migration Path

1. Create Containerfile (Phase 1)
2. Update docker-compose.observability.yml (Phase 2)
3. Add just recipes (Phase 3)
4. Test connectivity
5. Update QUICK_START_OBSERVABILITY.md with new workflow

## Next Steps

- [ ] Create Containerfile
- [ ] Update docker-compose.observability.yml
- [ ] Add just recipes to justfile
- [ ] Test full stack startup
- [ ] Verify metrics flow: bot-server → Prometheus → Grafana
- [ ] Verify traces flow: bot-server → Jaeger
- [ ] Update quickstart documentation
- [ ] Add troubleshooting guide

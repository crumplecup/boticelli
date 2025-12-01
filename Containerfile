# Multi-stage build for Botticelli Actor Server with cargo-chef
# Uses Fedora as base to match host environment

# Base image with Rust and build tools
FROM registry.fedoraproject.org/fedora:latest AS base

# Install build dependencies
RUN dnf install -y \
    gcc \
    gcc-c++ \
    make \
    cmake \
    openssl-devel \
    pkg-config \
    postgresql-devel \
    && dnf clean all

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install cargo-chef for dependency caching
RUN cargo install cargo-chef

WORKDIR /app

# Planner stage - analyze dependencies
FROM base AS planner
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo chef prepare --recipe-path recipe.json

# Cacher stage - build dependencies only (cached layer)
FROM base AS cacher
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --bin actor-server --features discord,observability,otel-otlp

# Builder stage - build application code
FROM base AS builder

# Install diesel_cli for migrations
RUN cargo install diesel_cli --no-default-features --features postgres

# Copy pre-built dependencies from cacher
COPY --from=cacher /app/target target
COPY --from=cacher /root/.cargo /root/.cargo

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY migrations ./migrations
COPY diesel.toml ./

# Copy config files needed by include_str! macros
COPY botticelli.toml actor_server.toml bot_server.toml actor.toml ./

# Build release binary (dependencies already built)
RUN cargo build --release --bin actor-server --features discord,observability,otel-otlp

# Runtime stage
FROM registry.fedoraproject.org/fedora:latest

# Install runtime dependencies
RUN dnf install -y \
    openssl \
    ca-certificates \
    postgresql \
    postgresql-libs \
    && dnf clean all

# Create non-root user
RUN useradd -m -u 1000 botticelli

# Create app directory
WORKDIR /app

# Copy binary and diesel_cli from builder
COPY --from=builder /app/target/release/actor-server /usr/local/bin/actor-server
COPY --from=builder /root/.cargo/bin/diesel /usr/local/bin/diesel

# Copy configuration files
COPY actor_server.toml ./
COPY crates/botticelli_server/configs ./crates/botticelli_server/configs
COPY crates/botticelli_narrative/narratives ./crates/botticelli_narrative/narratives

# Copy migrations and diesel config
COPY migrations ./migrations
COPY diesel.toml ./

# Copy entrypoint script
COPY docker-entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

# Create state directory
RUN mkdir -p /app/state && chown botticelli:botticelli /app/state

# Switch to non-root user
USER botticelli

# Environment variables (overridden at runtime)
ENV RUST_LOG=info
# Traces: OTLP to Jaeger (working)
ENV OTEL_EXPORTER=otlp
ENV OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:4318

# Metrics: Disabled (stdout only) - See METRICS_GRAFANA_FIX.md
# ENV OTEL_EXPORTER_OTLP_METRICS_ENDPOINT=http://prometheus:9090/api/v1/otlp/v1/metrics

# Expose metrics port
EXPOSE 9464

# Run the entrypoint script
ENTRYPOINT ["docker-entrypoint.sh"]

# Deployment-Aware Configuration

## Overview

Botticelli uses environment-aware configuration defaults to seamlessly work across local development and containerized deployments without manual configuration changes.

## How It Works

### Database Configuration

The `DatabaseConfig` in `botticelli_server` provides smart defaults based on the `DEPLOYMENT_ENV` variable:

```rust
use botticelli_server::DatabaseConfig;

// Automatically selects correct defaults
let config = DatabaseConfig::from_env()?;
```

### Environment Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `DEPLOYMENT_ENV` | Set deployment context | `local` |
| `DATABASE_URL` | Override database URL | Environment-specific |

### Default Database URLs

| Environment | Default URL |
|-------------|-------------|
| Local dev (`DEPLOYMENT_ENV` not set or `local`) | `postgresql://postgres:postgres@localhost:5432/botticelli` |
| Container (`DEPLOYMENT_ENV=container`) | `postgresql://botticelli:botticelli@postgres:5432/botticelli` |
| Custom (set `DATABASE_URL`) | Uses provided URL |

## Usage

### Local Development

No configuration needed - just works:

```bash
# .env file (DATABASE_URL optional)
DISCORD_TOKEN=...
GEMINI_API_KEY=...

# Run locally
cargo run --bin actor-server --features discord
```

Uses: `postgresql://postgres:postgres@localhost:5432/botticelli`

### Container Deployment

Automatically uses container defaults:

```bash
# Containerfile sets DEPLOYMENT_ENV=container
just bot-build
just bot-up
```

Uses: `postgresql://botticelli:botticelli@postgres:5432/botticelli`

Note: Container uses `postgres` hostname (container name in `botticelli` network)

### Custom Configuration

Override with explicit `DATABASE_URL`:

```bash
# .env
DATABASE_URL=postgresql://prod:secret@prod-db:5432/botticelli

# Works in both local and container environments
```

## Implementation

### Server Code

```rust
use botticelli_server::DatabaseConfig;

// In your server initialization
let db_config = DatabaseConfig::from_env()?;
let mut conn = PgConnection::establish(db_config.url())?;
```

### Container Setup

The `docker-entrypoint.sh` automatically sets:

```bash
export DEPLOYMENT_ENV=container
```

This ensures container runs use the correct defaults without requiring `.env` changes.

## Benefits

1. **Zero Config**: Works out of the box in both environments
2. **Explicit Override**: Can still set `DATABASE_URL` for custom setups
3. **No Duplication**: Single source of truth in code
4. **Type Safe**: Rust ensures correct configuration at compile time
5. **Self-Documenting**: Environment variables show intent

## Database Synchronization

See [DATABASE_SYNC_STRATEGY.md](DATABASE_SYNC_STRATEGY.md) for:
- Syncing between local and container databases
- Exporting/importing data
- Schema migration strategies
- Collaboration workflows

## Configuration Crate

The `config` crate is available for more complex configuration needs:
- TOML/YAML/JSON config files
- Hierarchical configuration
- Complex validation rules
- Multiple environment profiles

However, for simple environment-based switching, the built-in approach is sufficient and preferred.

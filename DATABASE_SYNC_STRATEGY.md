# Database Synchronization Strategy

## Overview

Botticelli uses PostgreSQL for state persistence. This document covers how to manage databases across different environments:

1. **Local PostgreSQL** - Your development database on the host machine (port 5432)
2. **Container PostgreSQL** - Isolated database inside the bot-server container
3. **Docker Compose PostgreSQL** - Shared database for observability stack (port 5433)

## Current Architecture

### Three Database Options

**1. Host Local Database (Development)**
- Port: 5432 (default)
- Location: Host machine
- Use case: Development, testing, direct access
- Access: `psql -U botticelli -h localhost -d botticelli`

**2. Docker Compose Database (Observability)**
- Port: 5433 (mapped from container 5432)
- Location: Docker/Podman container
- Use case: Observability stack integration
- Access: `psql -U botticelli -h localhost -p 5433 -d botticelli`

**3. Bot-Server Container Database (Production-like)**
- Port: Internal to container
- Location: Inside bot-server container
- Use case: Production simulation, isolation testing
- Access: Through container exec or network bridge

## Synchronization Strategies

### Strategy 1: Shared Host Database (Recommended for Development)

**What:** All services connect to your local PostgreSQL on port 5432

**How:**
```bash
# 1. Ensure local PostgreSQL is running
systemctl status postgresql

# 2. Create database and user (if not exists)
sudo -u postgres psql -c "CREATE DATABASE botticelli;"
sudo -u postgres psql -c "CREATE USER botticelli WITH PASSWORD 'renaissance';"

# 3. Run migrations
diesel migration run

# 4. Configure bot-server container to use host database
# In .env:
DATABASE_HOST=host.containers.internal  # Docker
# OR
DATABASE_HOST=host.gateway.internal     # Podman

# 5. Build and run bot-server with host network access
just bot-run-shared-db
```

**Pros:**
- Single source of truth
- No sync needed
- Easy direct access with psql
- All data in one place

**Cons:**
- Container needs host network access
- Less production-like isolation

### Strategy 2: Container Database with Sync Tools

**What:** Bot-server runs its own database, synced bidirectionally with host

**How:**

#### A. Export/Import (Simple, One-Time)

```bash
# Export from host to file
pg_dump -U botticelli -h localhost -d botticelli -f /tmp/botticelli_backup.sql

# Import to container
podman exec -i botticelli-bot-server psql -U botticelli -d botticelli < /tmp/botticelli_backup.sql
```

#### B. Continuous Sync (Advanced)

Use `pg_dump` and `pg_restore` in watch mode:

```bash
# Justfile recipe for sync
just db-sync-to-container    # Host → Container
just db-sync-from-container  # Container → Host
```

#### C. Volume Mount (Best for Development)

Mount host PostgreSQL data directory into container:

```bash
# In Containerfile or podman run:
-v /var/lib/postgres/data:/var/lib/postgresql/data:Z
```

**Warning:** Only do this if PostgreSQL versions match exactly!

### Strategy 3: Docker Compose Shared Database

**What:** Both host and container connect to Docker Compose PostgreSQL (port 5433)

**How:**
```bash
# 1. Start observability stack
just obs-start

# 2. Configure host tools to use port 5433
export DATABASE_PORT=5433

# 3. Configure bot-server container to use compose database
# In docker-compose or podman-compose:
environment:
  - DATABASE_HOST=postgres
  - DATABASE_PORT=5432  # Internal port
networks:
  - botticelli

# 4. Run migrations against compose database
DATABASE_URL="postgres://botticelli:botticelli@localhost:5433/botticelli" diesel migration run
```

**Pros:**
- Single shared database
- Easy container networking
- Observability stack integration

**Cons:**
- Must keep Docker Compose running
- Different port from standard

## Recommended Setup

### For Development (Recommended)

**Use Strategy 1: Shared Host Database**

```bash
# .env configuration
DATABASE_HOST=host.containers.internal  # or host.gateway.internal for podman
DATABASE_PORT=5432
DATABASE_USER=botticelli
DATABASE_PASSWORD=renaissance
DATABASE_NAME=botticelli
```

```bash
# Run bot-server with host network
just bot-run-shared-db
```

### For Production-like Testing

**Use Strategy 2: Isolated Container Database**

```bash
# Bot-server has its own PostgreSQL
# Sync snapshots as needed for debugging
just db-snapshot-container > backup.sql
```

### For Observability Integration

**Use Strategy 3: Docker Compose Shared Database**

```bash
# Start full stack
just obs-start

# Run bot-server in compose network
just bot-run-compose
```

## Sync Tools (To Be Implemented)

Add these recipes to `justfile`:

```just
# Export host database to SQL file
db-export output="botticelli_backup.sql":
    pg_dump -U botticelli -h localhost -d botticelli -f {{output}}

# Import SQL file to host database
db-import input="botticelli_backup.sql":
    psql -U botticelli -h localhost -d botticelli -f {{input}}

# Snapshot container database
db-snapshot-container:
    podman exec botticelli-bot-server pg_dump -U botticelli -d botticelli

# Restore snapshot to container
db-restore-container input="botticelli_backup.sql":
    cat {{input}} | podman exec -i botticelli-bot-server psql -U botticelli -d botticelli

# Sync host → container
db-sync-to-container:
    @echo "Syncing host database to container..."
    pg_dump -U botticelli -h localhost -d botticelli | \
        podman exec -i botticelli-bot-server psql -U botticelli -d botticelli

# Sync container → host
db-sync-from-container:
    @echo "Syncing container database to host..."
    podman exec botticelli-bot-server pg_dump -U botticelli -d botticelli | \
        psql -U botticelli -h localhost -d botticelli

# Compare row counts (sanity check)
db-compare:
    @echo "Host database:"
    @psql -U botticelli -h localhost -d botticelli -c "SELECT COUNT(*) FROM narrative_executions;"
    @echo "Container database:"
    @podman exec botticelli-bot-server psql -U botticelli -d botticelli -c "SELECT COUNT(*) FROM narrative_executions;"
```

## Troubleshooting

### Container Can't Reach Host PostgreSQL

**Problem:** `host.containers.internal` not resolving

**Solution (Docker):**
```bash
docker run --add-host=host.docker.internal:host-gateway ...
```

**Solution (Podman):**
```bash
podman run --add-host=host.containers.internal:host-gateway ...
```

### Port Conflicts

**Problem:** Port 5432 already in use

**Solutions:**
1. Use Docker Compose database on port 5433
2. Stop local PostgreSQL during container testing
3. Map container to different port: `-p 5434:5432`

### Data Loss Prevention

**Always backup before sync:**
```bash
# Backup both databases first
just db-export host_backup.sql
just db-snapshot-container > container_backup.sql

# Then sync
just db-sync-to-container
```

### Migration Conflicts

If migrations are out of sync:

```bash
# Check migration status on host
diesel migration list

# Check migration status in container
podman exec botticelli-bot-server diesel migration list

# Revert and re-run if needed
diesel migration revert
diesel migration run
```

## Security Considerations

1. **Never expose PostgreSQL ports publicly**
2. **Use strong passwords in production**
3. **Restrict database access to localhost or container network**
4. **Encrypt backups containing sensitive data**
5. **Use read-only users for observability queries**

## Next Steps

1. Choose synchronization strategy based on your workflow
2. Add sync justfile recipes
3. Test backup/restore process
4. Document team workflow
5. Consider automated backup cron jobs for production

## References

- [PostgreSQL Backup Guide](https://www.postgresql.org/docs/current/backup.html)
- [Docker Networking](https://docs.docker.com/network/)
- [Podman Networking](https://docs.podman.io/en/latest/markdown/podman-network.1.html)

#!/bin/bash
set -e

# Set deployment environment for config defaults
export DEPLOYMENT_ENV=container

# Wait for postgres to be ready
echo "Waiting for PostgreSQL to be ready..."
until pg_isready -h "${DATABASE_HOST:-postgres}" -p "${DATABASE_PORT:-5432}" -U "${DATABASE_USER:-botticelli}"; do
  echo "PostgreSQL is unavailable - sleeping"
  sleep 2
done

echo "PostgreSQL is ready"

# Run migrations if DATABASE_URL is set
if [ -n "$DATABASE_URL" ]; then
  echo "Running database migrations..."
  # Create diesel schema migrations table manually to avoid empty query issue
  psql "$DATABASE_URL" -c "CREATE TABLE IF NOT EXISTS __diesel_schema_migrations (version VARCHAR(50) PRIMARY KEY NOT NULL, run_on TIMESTAMP NOT NULL DEFAULT NOW());" 2>/dev/null || true
  # Run migrations (this will skip the initial setup since table exists)
  diesel migration run --database-url "$DATABASE_URL" || {
    echo "WARNING: Migrations failed, but continuing startup"
  }
  echo "Migrations complete"
else
  echo "WARNING: DATABASE_URL not set, skipping migrations"
fi

# Start the server
echo "Starting actor-server..."
exec actor-server

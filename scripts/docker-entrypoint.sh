#!/usr/bin/env bash
set -euo pipefail

if [[ -n "${DATABASE_URL:-}" ]]; then
  echo "Waiting for database..."
  # Use pg_isready with host 'db' since we're in Docker network
  until pg_isready -h db -p 5432 > /dev/null 2>&1; do
    sleep 1
  done

  echo "Running migrations..."
  shopt -s nullglob
  # Try /app/migrations first (Docker), then /workspace/migrations (dev)
  MIGRATIONS_DIR="/app/migrations"
  if [[ ! -d "$MIGRATIONS_DIR" ]]; then
    MIGRATIONS_DIR="/workspace/migrations"
  fi

  if [[ -d "$MIGRATIONS_DIR" ]]; then
    for migration in "$MIGRATIONS_DIR"/*.sql; do
      echo "Applying: $migration"
      psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f "$migration" || true
    done
  else
    echo "No migrations directory found"
  fi
fi

exec "$@"

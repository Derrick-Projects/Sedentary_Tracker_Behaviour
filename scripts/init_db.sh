#!/usr/bin/env bash
set -x
set -eo pipefail

# Check if psql is installed
if ! [ -x "$(command -v psql)" ]; then
  echo >&2 "Error: psql is not installed."
  echo >&2 "On Kali/Debian: sudo apt-get install postgresql-client"
  exit 1
fi

# Check if sqlx-cli is installed
if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "Error: sqlx is not installed."
  echo >&2 "Use:"
  echo >&2 "    cargo install --version='~0.7' sqlx-cli --no-default-features --features rustls,postgres"
  echo >&2 "to install it."
  exit 1
fi

# Set default values for environment variables if not set
DB_USER="${POSTGRES_USER}"
DB_PASSWORD="${POSTGRES_PASSWORD}"
DB_NAME="${POSTGRES_DB}"
DB_PORT="${POSTGRES_PORT}"
DB_HOST="${POSTGRES_HOST}"

# Launch Postgres using Docker
# Allow to skip Docker if a dockerized Postgres database is already running
if [[ -z "${SKIP_DOCKER}" ]]
then
  # Check if container already exists
  RUNNING_CONTAINER=$(docker ps --filter 'name=sedentary_tracker_db' --format '{{.ID}}')
  if [[ -n $RUNNING_CONTAINER ]]; then
    echo >&2 "Postgres container already running, skipping Docker launch."
  else
    # Remove stopped container if exists
    docker rm -f sedentary_tracker_db 2>/dev/null || true
    
    docker run \
        --name sedentary_tracker_db \
        -e POSTGRES_USER=${DB_USER} \
        -e POSTGRES_PASSWORD=${DB_PASSWORD} \
        -e POSTGRES_DB=${DB_NAME} \
        -p "${DB_PORT}:5432" \
        -d postgres:15 \
        postgres -N 1000
        # ^ Increased maximum connections for testing purposes
  fi
fi

# Keep pinging Postgres until it's ready to accept commands
export PGPASSWORD="${DB_PASSWORD}"
until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
  >&2 echo "Postgres is still unavailable - sleeping"
  sleep 1
done

>&2 echo "Postgres is up and running on port ${DB_PORT}!"

# Create the database and run migrations
DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
export DATABASE_URL

sqlx database create
sqlx migrate run

>&2 echo "Database '${DB_NAME}' created and migrations applied!"
>&2 echo "DATABASE_URL=${DATABASE_URL}"

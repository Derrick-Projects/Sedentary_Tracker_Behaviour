#!/bin/bash
set -e

echo "=================================="
echo "Sedentary Tracker - Demo Setup"
echo "=================================="

# Clean up any existing containers to avoid name conflicts
echo "Cleaning up existing containers..."
docker rm -f sedentary_redis sedentary_db sedentary_backend sedentary_replay_trigger 2>/dev/null || true
docker compose -f docker-compose.yml down --remove-orphans 2>/dev/null || true

# Generate .env from Codespaces secrets (injected as env vars) with defaults
cat > .env << EOF
POSTGRES_USER=${POSTGRES_USER}
POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
POSTGRES_DB=${POSTGRES_DB}
JWT_SECRET=${JWT_SECRET}
REDIS_PORT=${REDIS_PORT:-6379}
REDIS_URL=${REDIS_URL:-redis://redis:${REDIS_PORT:-6379}/}
SERIAL_PORT=${SERIAL_PORT:-/dev/null}
BAUD_RATE=${BAUD_RATE:-115200}
SERVER_PORT=${SERVER_PORT:-8000}
RUST_LOG=${RUST_LOG:-info}
FALLBACK_TIMEOUT_SECONDS=${FALLBACK_TIMEOUT_SECONDS:-5}
REPLAY_LOG_PATH=${REPLAY_LOG_PATH:-/app/arduino_data.log}
REPLAY_SPEED_MS=${REPLAY_SPEED_MS:-5}
DISABLE_FALLBACK=${DISABLE_FALLBACK:-true}
SKIP_HISTORY=${SKIP_HISTORY:-true}
ARDUINO_LOG_FILE=${ARDUINO_LOG_FILE:-./arduino_data.log}
EOF

echo "Starting demo services..."
docker compose -f docker-compose.yml up -d --build

echo "Waiting for database..."
for i in {1..30}; do
    if docker compose -f docker-compose.yml exec -T db pg_isready -U ${POSTGRES_USER} > /dev/null 2>&1; then
        echo "Database ready!"
        break
    fi
    echo "Waiting... ($i/30)"
    sleep 2
done

echo "Seeding demo data..."
docker compose -f docker-compose.yml exec -T db psql -U ${POSTGRES_USER} -d ${POSTGRES_DB} -f /dev/stdin < scripts/seed-demo-data.sql

echo "Waiting for backend..."
for i in {1..30}; do
    if curl -s http://localhost:8000/health > /dev/null 2>&1; then
        echo "Backend ready!"
        break
    fi
    echo "Waiting... ($i/30)"
    sleep 2
done

echo ""
echo "=================================="
echo "Demo running at http://localhost:8000"
echo "=================================="
echo ""
echo "Live sensor data replay started from arduino_data.log"
echo ""
echo "Commands:"
echo "  docker compose -f docker-compose.yml logs -f backend"
echo "  docker compose -f docker-compose.yml down"
echo ""

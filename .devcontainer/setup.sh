#!/bin/bash
set -e

echo "============================================"
echo "Sedentary Tracker - Codespaces Setup"
echo "============================================"

echo "Installing system dependencies..."
sudo apt-get update
sudo apt-get install -y libudev-dev pkg-config postgresql-client

echo "Generating .env from Codespaces secrets..."

# Create .env file from Codespaces secrets (these are injected as env vars)
cat > .env << EOF
# Auto-generated from Codespaces secrets
# Do not commit this file

# Database Configuration
DATABASE_URL=${DATABASE_URL}
POSTGRES_USER=${POSTGRES_USER}
POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
POSTGRES_DB=${POSTGRES_DB}

# Redis Configuration
REDIS_URL=${REDIS_URL}

# Authentication
JWT_SECRET=${JWT_SECRET}

# Hardware Configuration
SERIAL_PORT=${SERIAL_PORT:-/dev/null}
BAUD_RATE=${BAUD_RATE:-115200}

# Server Configuration
SERVER_PORT=8000
SERVER_ADDRESS=0.0.0.0:8000
FRONTEND_DIR=/app/frontend

# Activity Thresholds
THRESH_FIDGET=0.020
THRESH_ACTIVE=0.040
ALERT_LIMIT_SECONDS=1200

# Fallback Configuration (for cloud/no-hardware mode)
FALLBACK_TIMEOUT_SECONDS=5
FALLBACK_BATCH_SIZE=500
FALLBACK_REPLAY_INTERVAL_MS=100

# Logging
RUST_LOG=info
PYTHONUNBUFFERED=1
EOF

echo ".env file created successfully"

echo "Starting services with docker compose..."
docker compose up -d --build

echo "Waiting for PostgreSQL to be ready..."
for i in {1..60}; do
    if docker compose exec -T db pg_isready -U ${POSTGRES_USER:-postgres} > /dev/null 2>&1; then
        echo "PostgreSQL is ready!"
        break
    fi
    echo "Waiting for database... ($i/60)"
    sleep 2
done

echo "Waiting for backend to be ready..."
for i in {1..60}; do
    if curl -s http://localhost:8000/health > /dev/null 2>&1; then
        echo "Backend is ready!"
        break
    fi
    echo "Waiting for backend... ($i/60)"
    sleep 2
done

echo ""
echo "============================================"
echo "Setup complete! All services are running."
echo "============================================"
echo ""
echo "The web app should open automatically."
echo "If not, click the 'Ports' tab and open port 8000."
echo ""
echo "Services:"
echo "  - Web App:    http://localhost:8000"
echo "  - PostgreSQL: localhost:5432 (internal)"
echo "  - Redis:      localhost:6379 (internal)"
echo ""
echo "Commands:"
echo "  docker compose logs -f backend  # View backend logs"
echo "  docker compose restart backend  # Restart backend"
echo "  docker compose down             # Stop all services"
echo "  docker compose up -d --build    # Rebuild and start"
echo ""
echo "Note: Running in fallback mode - replaying historical data"
echo "      (Arduino hardware not available in cloud environment)"
echo ""

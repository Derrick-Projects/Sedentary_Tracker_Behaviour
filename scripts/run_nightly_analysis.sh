#!/bin/bash
# Nightly ML Analysis Runner
# This script runs at 2 AM daily to generate activity summaries

set -e

echo "========================================="
echo "Starting Nightly ML Analysis"
echo "Time: $(date)"
echo "========================================="

# Wait for database to be ready
until pg_isready -h db -p 5432 -U postgres > /dev/null 2>&1; do
    echo "Waiting for database..."
    sleep 2
done

echo "Database ready"

# Run the enhanced analytics
python /app/enhanced_analytics.py

echo "========================================="
echo "Nightly Analysis Completed"
echo "Time: $(date)"
echo "========================================="

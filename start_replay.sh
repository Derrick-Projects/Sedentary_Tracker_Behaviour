#!/bin/bash
# Trigger replay of arduino_data.log through the server
# The frontend will receive processed data and update charts

echo "Starting replay of arduino_data.log..."
curl -s http://localhost:8000/api/replay
echo ""
echo "Replay started! Open http://localhost:8000 to see the charts."

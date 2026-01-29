FROM python:3.11-slim

WORKDIR /app

# Install system dependencies including cron for scheduling
RUN apt-get update && apt-get install -y --no-install-recommends \
    postgresql-client \
    cron \
    && rm -rf /var/lib/apt/lists/*

# Install Python dependencies (binary wheels preferred)
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application files
COPY enhanced_analytics.py .
COPY model_classification.py .

# Make scripts executable
RUN chmod +x enhanced_analytics.py

# Default command (can be overridden)
CMD ["python", "enhanced_analytics.py"]

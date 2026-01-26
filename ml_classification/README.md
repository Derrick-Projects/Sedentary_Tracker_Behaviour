# ML-Powered Sedentary Behavior Analytics

## Overview

This ML analytics system provides comprehensive, user-centered sedentary behavior statistics with LOINC-compliant FHIR observations. It runs nightly to generate daily, weekly, and monthly activity summaries using KMeans clustering for pattern detection.

## LOINC Compliance

**LOINC Code:** `87705-0` - Sedentary activity 24 hour
**Property:** NRat (Number Rate)
**Time:** 24H
**System:** ^Patient
**Scale:** Quantitative

### Reference Information

Sedentary activity is defined as lying down, sitting, or expending energy between 1.0-1.5 metabolic equivalents (METs). This system measures sedentary behavior patterns to help identify health risks associated with prolonged inactivity.

## Features

### 1. User-Specific Analytics
- **Multi-user support** with UUID-based tracking
- **Single-user mode** for personal deployments
- **Privacy-preserving** data isolation

### 2. Temporal Aggregations
- **Daily summaries**: 24-hour activity profiles
- **Weekly summaries**: 7-day rolling averages (calculated on Sundays)
- **Monthly summaries**: Full month statistics (calculated on last day)

### 3. Comprehensive Metrics

#### Time-Based Metrics
- Sedentary minutes
- Fidget minutes (micro-movements)
- Active minutes
- Total activity time

#### Performance Indicators
- Activity Score (0-100, higher is better)
- Sedentary percentage
- Active percentage
- Dominant state (ACTIVE/SEDENTARY)

#### Alert System
- 20-minute sedentary alert count
- Longest continuous sedentary period
- Real-time health warnings

### 4. Machine Learning

#### KMeans Clustering
- **3-cluster model**: Sedentary, Fidget, Active
- **Pattern detection**: Identifies recurring behavior profiles
- **Adaptive thresholds**: Suggests personalized activity thresholds
- **Distribution analysis**: Cluster population statistics

#### Signal Processing
- 10-sample moving average smoothing
- Acceleration delta magnitude calculation
- PIR sensor fusion for improved accuracy

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Nightly ML Analytics Pipeline                              │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  1. Data Collection (PostgreSQL sedentary_log)              │
│     └─ Last 24 hours of sensor data                         │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  2. Statistical Analysis                                     │
│     ├─ Basic stats calculation                              │
│     ├─ KMeans clustering (n=3)                              │
│     └─ Threshold optimization                               │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  3. Persistence (activity_summary table)                    │
│     ├─ Daily summary → CURRENT_DATE                         │
│     ├─ Weekly summary → Sunday (if applicable)              │
│     └─ Monthly summary → Last day of month                  │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  4. FHIR Observation Export                                 │
│     └─ LOINC 87705-0 compliant observations                 │
└─────────────────────────────────────────────────────────────┘
```

## Database Schema

### activity_summary Table

```sql
CREATE TABLE activity_summary (
    id SERIAL PRIMARY KEY,
    user_id UUID REFERENCES users(user_id),
    date DATE NOT NULL,
    period_type VARCHAR(10) CHECK (period_type IN ('daily', 'weekly', 'monthly')),

    -- Time metrics (minutes)
    sedentary_minutes REAL,
    fidget_minutes REAL,
    active_minutes REAL,
    total_minutes REAL,

    -- Percentages
    sedentary_percentage REAL,
    active_percentage REAL,

    -- State tracking
    dominant_state VARCHAR(20),

    -- Scores (0-100)
    activity_score INTEGER,

    -- Alert metrics
    alert_count INTEGER,
    longest_sedentary_period INTEGER, -- seconds

    -- ML analysis
    detected_patterns JSONB,
    suggested_fidget_threshold REAL,
    suggested_active_threshold REAL,

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(user_id, date, period_type)
);
```

## API Endpoints

### 1. User Analytics (FHIR Bundle)

```http
GET /api/fhir/analytics/user/{user_id}?period=daily&limit=30
```

**Response:** FHIR Bundle with Observation resources

**Query Parameters:**
- `period`: `daily` | `weekly` | `monthly` (default: `daily`)
- `limit`: Number of records (default: 30)

**Example:**
```bash
curl http://localhost:8000/api/fhir/analytics/user/123e4567-e89b-12d3-a456-426614174000?period=daily&limit=7
```

**Response Structure:**
```json
{
  "resourceType": "Bundle",
  "type": "searchset",
  "total": 7,
  "entry": [
    {
      "resource": {
        "resourceType": "Observation",
        "id": "activity-summary-123",
        "status": "final",
        "code": {
          "coding": [{
            "system": "http://loinc.org",
            "code": "87705-0",
            "display": "Sedentary activity 24 hour"
          }],
          "text": "Sedentary activity 24 hour"
        },
        "subject": {
          "reference": "Patient/123e4567-e89b-12d3-a456-426614174000"
        },
        "effectiveDateTime": "2026-01-22T02:00:00Z",
        "valueQuantity": {
          "value": 16.5,
          "unit": "h/(24.h)",
          "system": "http://unitsofmeasure.org",
          "code": "h/(24.h)"
        },
        "component": [
          {
            "code": {
              "coding": [{
                "system": "http://loinc.org",
                "code": "CUSTOM-ACTIVITY-SCORE",
                "display": "Activity Score"
              }]
            },
            "valueInteger": 72
          },
          {
            "code": {
              "coding": [{
                "system": "http://loinc.org",
                "code": "CUSTOM-DOMINANT-STATE",
                "display": "Dominant Activity State"
              }]
            },
            "valueString": "ACTIVE"
          },
          {
            "code": {
              "coding": [{
                "system": "http://loinc.org",
                "code": "CUSTOM-ALERT-COUNT",
                "display": "Sedentary Alert Count"
              }]
            },
            "valueInteger": 3
          },
          {
            "code": {
              "coding": [{
                "system": "http://loinc.org",
                "code": "CUSTOM-ACTIVE-MINUTES",
                "display": "Active Minutes"
              }]
            },
            "valueQuantity": {
              "value": 425.5,
              "unit": "min",
              "system": "http://unitsofmeasure.org",
              "code": "min"
            }
          }
        ]
      }
    }
  ]
}
```

### 2. Latest Analytics (All Users)

```http
GET /api/fhir/analytics/latest?period=daily&limit=10
```

**Response:** JSON array with summary statistics

**Example:**
```json
[
  {
    "userId": "123e4567-e89b-12d3-a456-426614174000",
    "date": "2026-01-22",
    "activityScore": 72,
    "dominantState": "ACTIVE",
    "sedentaryHours24h": 7.5,
    "loincCode": "87705-0"
  }
]
```

## Scheduling

### Nightly Execution

The ML analytics service runs automatically at **2:00 AM daily** via cron:

```
0 2 * * * /usr/local/bin/python /app/enhanced_analytics.py
```

### Manual Execution

Run immediately via Docker:

```bash
# Execute in running container
docker exec sedentary_ml_analytics python enhanced_analytics.py

# Or start a one-off container
docker-compose run --rm ml_analytics python enhanced_analytics.py
```

### Immediate Mode (Testing)

For development, uncomment this line in `docker-compose.yml`:

```yaml
ml_analytics:
  # ...
  command: ["python", "enhanced_analytics.py"]  # Run immediately on startup
```

## Installation & Setup

### 1. Run Database Migration

```bash
# The migration runs automatically on server startup
# Or manually apply with sqlx-cli:
sqlx migrate run
```

### 2. Build and Start Services

```bash
# Build all services including ML analytics
docker-compose up -d --build

# Check ML service logs
docker logs sedentary_ml_analytics -f
```

### 3. Verify Installation

```bash
# Check if cron is running
docker exec sedentary_ml_analytics crontab -l

# Manually trigger analysis
docker exec sedentary_ml_analytics python enhanced_analytics.py
```

### 4. Query Results

```bash
# Check daily summaries
docker exec sedentary_db psql -U postgres -d sedentary_data -c "SELECT * FROM activity_summary WHERE period_type='daily' ORDER BY date DESC LIMIT 5;"

# Check FHIR API
curl http://localhost:8000/api/fhir/analytics/latest
```

## Configuration

### Environment Variables

```bash
DATABASE_URL=postgres://<user>:<password>@<host>:<port>/<database>
PYTHONUNBUFFERED=1  # For real-time logging
```

### Thresholds (Configurable in enhanced_analytics.py)

```python
THRESH_FIDGET = 0.020  # Fidget threshold (acceleration delta)
THRESH_ACTIVE = 0.040  # Active threshold
```

## Output Examples

### Console Output

```
============================================================
NIGHTLY SEDENTARY BEHAVIOR ANALYSIS
   Timestamp: 2026-01-22T02:00:00
   LOINC Code: 87705-0 (Sedentary activity 24 hour)
============================================================
 
Connected to database

Processing user 123e4567-e89b-12d3-a456-426614174000...
Loaded 86400 data points for user 123e4567-e89b-12d3-a456-426614174000
Detected cluster centers: ['0.0123', '0.0287', '0.0521']
Suggested New Fidget Threshold: 0.0205
Daily summary saved for user ,,,, on 2026-01-22
   Activity Score: 72/100
   Sedentary: 450.5 min (31.2%)
   Active: 989.5 min (68.8%)

============================================================
 NIGHTLY ANALYSIS COMPLETED SUCCESSFULLY
============================================================
```

## Healthcare Integration

### EHR System Integration

The FHIR API enables seamless integration with Electronic Health Record systems:

1. **Epic/Cerner**: Import via FHIR Observation resources
2. **Health apps**: Apple Health, Google Fit compatibility
3. **Research platforms**: Export to REDCap, OMOP CDM

### Clinical Decision Support

Activity scores and alerts can trigger:
- Patient outreach for prolonged sedentary behavior
- Rehabilitation progress tracking
- Cardiovascular risk assessment

## Troubleshooting

### No Data Generated

```bash
# Check if cron is running
docker exec sedentary_ml_analytics ps aux | grep crond

# Check logs
docker logs sedentary_ml_analytics

# Manually run to see errors
docker exec sedentary_ml_analytics python enhanced_analytics.py
```

### Database Connection Errors

```bash
# Verify database is healthy
docker ps | grep sedentary_db

# Test connection
docker exec sedentary_ml_analytics psql -h db -U postgres -d sedentary_data -c "SELECT 1;"
```

### Missing Dependencies

```bash
# Rebuild ML service
docker-compose build ml_analytics
docker-compose up -d ml_analytics
```

## Performance Considerations

- **Data volume**: Optimized for 86,400 samples/day (10Hz × 24hrs)
- **Processing time**: ~2-5 seconds for daily analysis
- **Storage**: ~500 bytes per daily summary
- **Scalability**: Handles 1000+ users with proper indexing

## Future Enhancements

- [ ] Predictive modeling for sedentary episodes
- [ ] Personalized activity recommendations
- [ ] Integration with wearable devices
- [ ] Multi-variate analysis (heart rate, sleep, etc.)
- [ ] Automated health reports via email/SMS

## References

- LOINC: https://loinc.org/87705-0/
- FHIR Observation: http://hl7.org/fhir/observation.html
- Sedentary Behavior Research: PMC:2996155




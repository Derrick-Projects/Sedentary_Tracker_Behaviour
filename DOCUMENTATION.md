# Sedentary Activity Tracker - Comprehensive Documentation

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [Hardware Integration](#2-hardware-integration)
3. [Backend Architecture](#3-backend-architecture)
4. [Real-Time Data Pipeline](#4-real-time-data-pipeline)
5. [Signal Processing](#5-signal-processing)
6. [FHIR Healthcare Integration](#6-fhir-healthcare-integration)
7. [Authentication & Security](#7-authentication--security)
8. [Fallback System](#8-fallback-system)
9. [ML Analytics](#9-ml-analytics)
10. [Frontend Dashboard](#10-frontend-dashboard)
11. [API Reference](#11-api-reference)
12. [Configuration](#12-configuration)
13. [Deployment](#13-deployment)

---

## 1. Project Overview

### 1.1 Problem Statement

Prolonged sedentary behavior is a growing health concern linked to cardiovascular disease, metabolic syndrome, obesity, and increased mortality risk. The **Sedentary Activity Tracker** provides real-time monitoring and automated detection of extended sitting/inactivity periods, empowering users to take timely breaks and improve their health outcomes.

### 1.2 Target Users & Use Cases

| User Type | Use Case |
|-----------|----------|
| **Individual Health Trackers** | Monitor daily activity patterns and receive sedentary alerts |
| **Office Workers** | Get reminded to take breaks during long work sessions |
| **Healthcare Providers** | Integrate patient activity data with EHRs via FHIR |
| **Wellness Researchers** | Analyze behavioral patterns using ML-generated insights |
| **Rehabilitation Centers** | Monitor post-surgery patient activity during recovery |

### 1.3 Key Features

- **Real-time Monitoring**: Sub-second latency from sensor to dashboard
- **Activity Classification**: Automatic detection of ACTIVE, FIDGET, and SEDENTARY states
- **Sedentary Alerts**: Configurable alerts when inactivity exceeds threshold (default: 20 minutes)
- **Healthcare Compliance**: FHIR R4 compatible API with LOINC coding
- **Machine Learning**: Nightly KMeans clustering for adaptive threshold suggestions
- **Secure Authentication**: JWT tokens with Argon2id password hashing
- **Fault Tolerance**: Automatic fallback to historical data replay when hardware unavailable
- **Multi-Platform**: Works with physical Arduino or in cloud environments (GitHub Codespaces)

### 1.4 System Architecture

```
┌─────────────────┐         ┌─────────────────────────────────────────────┐
│   Arduino       │  Serial │              Rust Server                    │
│  (Sensor Hub)   │────────▶│  @ 8000                                     │
│                 │  JSON   │  ┌─────────────┐    ┌──────────────────┐   │
│  PIR + MPU6050  │  10Hz   │  │ serial.rs   │───▶│ broadcast::Hub   │   │
│  RTC (DS3231)   │         │  │ + Logic     │    │                  │   │
└─────────────────┘         │  │ Processing  │    │   ┌──────────┐   │   │
                            │  └─────────────┘    │   │WebSocket │───┼───┼──▶ Browser (D3.js)
                            │                     │   │SSE       │   │   │
                            │                     │   └──────────┘   │   │
                            │                     │   ┌──────────┐   │   │
                            │                     │   │DB Worker │───┼───┼──▶ PostgreSQL
                            │                     │   └──────────┘   │   │
                            │  ┌─────────────┐    │   ┌──────────┐   │   │
                            │  │ fhir.rs     │◀───┼──▶│Redis     │   │   │
                            │  │ (FHIR API)  │    │   │Cache     │   │   │
                            │  └─────────────┘    │   └──────────┘   │   │
                            └─────────────────────────────────────────────┘
                                                          │
                            ┌─────────────────────────────────────────┐
                            │    Python ML Service (Nightly @2AM)    │
                            │    KMeans Clustering & Analytics       │
                            └─────────────────────────────────────────┘
```

### 1.5 Technology Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| **Hardware** | Arduino + MPU6050 + PIR + DS3231 | Sensor data collection |
| **Backend** | Rust + Axum + Tokio | High-performance async web server |
| **Database** | PostgreSQL 15 | Persistent storage |
| **Cache** | Redis 7 | Real-time data caching & rate limiting |
| **ML Service** | Python + scikit-learn | Nightly analytics & clustering |
| **Frontend** | D3.js + Vanilla JS | Real-time visualization |
| **Deployment** | Docker + Docker Compose | Containerized deployment |

---

## 2. Hardware Integration

### 2.1 Component List

| Component | Model | Purpose | Interface |
|-----------|-------|---------|-----------|
| **Microcontroller** | Arduino Uno/Nano | Main processor | USB Serial |
| **Accelerometer** | MPU6050 (GY-521) | 3-axis motion detection | I2C (0x69) |
| **Motion Sensor** | HC-SR501 PIR | Large body movement | Digital (D7) |
| **Real-Time Clock** | DS3231 RTC | Accurate timestamps | I2C (0x68) |

### 2.2 Wiring Diagram

```
                    Arduino Uno
                   ┌───────────┐
                   │           │
    MPU6050 ──────▶│ A4 (SDA)  │
    DS3231  ──────▶│ A5 (SCL)  │
                   │           │
    PIR OUT ──────▶│ D7        │
                   │           │
    GND ──────────▶│ GND       │
    5V  ──────────▶│ 5V        │
                   │           │
                   │ USB ──────┼──▶ Computer (Serial)
                   └───────────┘
```

### 2.3 Serial Communication Protocol

**Configuration:**
- **Baud Rate**: 115,200
- **Format**: JSON (one object per line)
- **Sampling Rate**: 10Hz (100ms interval)

**Data Format (Arduino → Server):**
```json
{
  "ts": "14:30:25",
  "pir": 0,
  "acc": 0.045
}
```

| Field | Type | Description |
|-------|------|-------------|
| `ts` | string | Timestamp from RTC (HH:MM:SS) |
| `pir` | integer | PIR sensor state (0=no movement, 1=movement) |
| `acc` | float | Acceleration delta magnitude (g-force) |

### 2.4 Arduino Code Behavior

The Arduino acts as a "dumb streamer" - it only collects and transmits raw sensor data:

1. Read 3-axis acceleration from MPU6050
2. Calculate acceleration magnitude delta
3. Read PIR digital state
4. Read timestamp from DS3231 RTC
5. Format as JSON and send over serial
6. Repeat every 100ms

All classification logic (ACTIVE/FIDGET/SEDENTARY) happens server-side.

---

## 3. Backend Architecture

### 3.1 Rust Workspace Structure

```
sedentary_tracker/
├── Cargo.toml              # Workspace definition
├── server/                 # Main HTTP/WebSocket server
│   ├── src/
│   │   ├── main.rs         # Entry point, router setup
│   │   ├── serial.rs       # Serial port listener
│   │   ├── auth.rs         # JWT authentication
│   │   ├── login.rs        # Login handler + rate limiting
│   │   ├── signup.rs       # User registration
│   │   ├── websocket.rs    # WebSocket handler
│   │   ├── sse.rs          # Server-Sent Events handler
│   │   ├── fhir.rs         # FHIR data structures
│   │   ├── fhir_analytics.rs # FHIR API endpoints
│   │   ├── db_worker.rs    # Async database writer
│   │   ├── fallback.rs     # Hardware unavailability handling
│   │   ├── replay.rs       # Log file replay
│   │   ├── models.rs       # Data structures
│   │   └── state.rs        # Application state
│   └── Cargo.toml
├── db/                     # Database utilities
│   ├── src/lib.rs          # PgPool, Observation struct
│   └── tests/              # Integration tests
├── logic/                  # Signal processing
│   ├── src/lib.rs          # Hjorth parameters, stationarity
│   └── tests/              # Unit tests
├── errors/                 # Math utilities
│   ├── src/lib.rs          # Overflow-safe arithmetic
│   └── tests/              # Unit tests
├── frontend/               # Static web files
│   ├── index.html          # Dashboard
│   ├── app.js              # D3.js visualization
│   └── styles.css          # Styling
├── ml_classification/      # Python ML service
│   └── enhanced_analytics.py
└── migrations/             # SQL migrations
```

### 3.2 Crate Responsibilities

| Crate | Purpose | Key Exports |
|-------|---------|-------------|
| **server** | HTTP server, WebSocket, SSE, authentication | `main()`, route handlers |
| **db** | Database connection pool and models | `create_pool()`, `Observation` |
| **logic** | Signal processing algorithms | `calculate_hjorth_params()`, `check_stationarity()` |
| **errors** | Overflow-safe math operations | `add()`, `checked_sub()`, `checked_mul()`, `checked_div()` |

### 3.3 Database Schema

#### Table: `sedentary_log`
Primary storage for real-time sensor readings.

```sql
CREATE TABLE sedentary_log (
    id SERIAL PRIMARY KEY,
    state VARCHAR(20) NOT NULL,        -- 'ACTIVE', 'FIDGET', 'SEDENTARY'
    timer_seconds INTEGER,              -- Seconds of inactivity
    acceleration_val REAL,              -- Smoothed acceleration value
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

#### Table: `users`
User accounts for multi-user support.

```sql
CREATE TABLE users (
    user_id UUID PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,        -- Argon2id PHC format
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);
```

#### Table: `sensor_data`
User-level sensor readings (mirrors sedentary_log with user association).

```sql
CREATE TABLE sensor_data (
    id SERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    state VARCHAR(20) NOT NULL,
    timer_seconds INTEGER NOT NULL DEFAULT 0,
    acceleration_val REAL NOT NULL DEFAULT 0.0,
    alert_triggered BOOLEAN NOT NULL DEFAULT FALSE,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sensor_data_user_created ON sensor_data(user_id, created_at DESC);
CREATE INDEX idx_sensor_data_user_state ON sensor_data(user_id, state);
```

#### Table: `activity_summary`
ML-generated daily/weekly/monthly statistics.

```sql
CREATE TABLE activity_summary (
    id SERIAL PRIMARY KEY,
    user_id UUID REFERENCES users(user_id) ON DELETE CASCADE,
    date DATE NOT NULL,
    period_type VARCHAR(10) NOT NULL CHECK (period_type IN ('daily', 'weekly', 'monthly')),

    -- Time metrics (minutes)
    sedentary_minutes REAL NOT NULL DEFAULT 0,
    fidget_minutes REAL NOT NULL DEFAULT 0,
    active_minutes REAL NOT NULL DEFAULT 0,
    total_minutes REAL NOT NULL DEFAULT 0,

    -- Percentages
    sedentary_percentage REAL NOT NULL DEFAULT 0,
    active_percentage REAL NOT NULL DEFAULT 0,

    -- State tracking
    dominant_state VARCHAR(20) NOT NULL,
    activity_score INTEGER NOT NULL DEFAULT 0,    -- 0-100 scale

    -- Alerts
    alert_count INTEGER NOT NULL DEFAULT 0,
    longest_sedentary_period INTEGER NOT NULL DEFAULT 0,

    -- ML results
    detected_patterns JSONB,
    suggested_fidget_threshold REAL,
    suggested_active_threshold REAL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(user_id, date, period_type)
);
```

### 3.4 Redis Usage

| Key Pattern | Type | TTL | Purpose |
|-------------|------|-----|---------|
| `sensor_history` | List | None | Last 500 ProcessedState JSON objects |
| `login_attempts:{email}` | Integer | 60s | Failed login attempt counter |

**Sensor History Operations:**
- `LPUSH sensor_history <json>` - Add new reading
- `LTRIM sensor_history 0 499` - Keep only last 500
- `LRANGE sensor_history 0 99` - Fetch last 100 for reconnection

---

## 4. Real-Time Data Pipeline

### 4.1 Data Flow

```
1. ARDUINO (10Hz)
   └── Send: {"ts":"14:30:25", "pir":0, "acc":0.045}

2. SERIAL LISTENER (serial.rs)
   ├── Parse JSON → RawReading struct
   ├── Update FallbackState.last_data_time
   └── Add to smoothing buffer

3. SMOOTHING (10-sample window = 1 second)
   └── smoothed_acc = mean(last 10 samples)

4. CLASSIFICATION (per reading)
   ├── IF pir=1 OR smoothed_acc > 0.040 → ACTIVE, timer=0
   ├── ELIF smoothed_acc > 0.020 → FIDGET, timer unchanged
   └── ELSE → SEDENTARY, timer++

5. ALERT CHECK
   └── IF timer >= 1200 → alert=true

6. BROADCAST (tokio::broadcast)
   └── Send ProcessedState JSON to all subscribers

7. PARALLEL OUTPUTS:
   ├── WebSocket/SSE → Browser (immediate)
   ├── Redis → LPUSH to sensor_history
   └── Database → INSERT to sedentary_log (async)
```

### 4.2 ProcessedState Structure

```rust
pub struct ProcessedState {
    pub state: String,              // "ACTIVE", "FIDGET", "SEDENTARY"
    pub timer: u64,                 // Seconds of inactivity
    pub val: f32,                   // Smoothed acceleration
    pub alert: bool,                // Sedentary alert triggered?
    pub timestamp: DateTime<Utc>,   // UTC timestamp
}
```

**JSON Output:**
```json
{
  "state": "SEDENTARY",
  "timer": 123,
  "val": 0.015,
  "alert": false,
  "timestamp": "2026-01-28T14:30:25Z"
}
```

### 4.3 WebSocket Implementation

**Endpoint:** `GET /ws`

**Connection Flow:**
1. Client initiates WebSocket upgrade
2. Server fetches last 100 readings from Redis
3. Server sends historical readings (for chart population)
4. Server subscribes to broadcast channel
5. Live updates streamed to client

```rust
async fn handle_socket(mut socket: WebSocket, state: AppState) {
    // Send historical data from Redis
    if let Ok(mut con) = state.redis.get_multiplexed_async_connection().await {
        let history: Vec<String> = con.lrange("sensor_history", 0, 99).await.unwrap_or(vec![]);
        for msg in history.into_iter().rev() {
            let _ = socket.send(Message::Text(msg)).await;
        }
    }

    // Stream live updates
    let mut rx = state.tx.subscribe();
    while let Ok(msg) = rx.recv().await {
        if socket.send(Message::Text(msg)).await.is_err() {
            break;
        }
    }
}
```

### 4.4 Server-Sent Events (SSE) Implementation

**Endpoint:** `GET /events`

**Advantages over WebSocket:**
- Works through HTTP proxies
- Simpler protocol (no upgrade handshake)
- Automatic browser reconnection
- Less overhead

**Response Format:**
```
event: sensor-data
data: {"state":"SEDENTARY","timer":123,"val":0.015,"alert":false,"timestamp":"2026-01-28T14:30:25Z"}

:keepalive
```

---

## 5. Signal Processing

### 5.1 Activity Classification Algorithm

The classification uses a state machine with three states:

```
                    ┌─────────────────────────────────┐
                    │                                 │
                    ▼                                 │
              ┌──────────┐                           │
              │  ACTIVE  │ ◀── PIR=1 OR acc>0.040 ───┤
              └────┬─────┘                           │
                   │                                 │
         (no movement detected)                      │
                   │                                 │
                   ▼                                 │
    ┌──────────────┴──────────────┐                 │
    │                             │                 │
    ▼                             ▼                 │
┌──────────┐               ┌──────────┐             │
│SEDENTARY │ ◀─────────────│  FIDGET  │             │
│ acc<0.02 │               │0.02<acc  │             │
│ timer++  │──────────────▶│  <0.04   │─────────────┘
└──────────┘   acc>0.02    │  timer   │
                           │unchanged │
                           └──────────┘
```

**Thresholds (configurable via environment):**

| Variable | Default | Description |
|----------|---------|-------------|
| `THRESH_FIDGET` | 0.020 | Minimum acceleration for fidgeting |
| `THRESH_ACTIVE` | 0.040 | Minimum acceleration for active state |
| `ALERT_LIMIT_SECONDS` | 1200 | Seconds before sedentary alert (20 min) |

**Timer Behavior:**

| State | Timer Action | Alert |
|-------|--------------|-------|
| ACTIVE | Reset to 0 | false |
| FIDGET | Unchanged (paused) | true if timer >= limit |
| SEDENTARY | Increment by 1 | true if timer >= limit |

### 5.2 Signal Smoothing

A 10-sample moving average window reduces noise while preserving movement patterns:

```rust
const SMOOTHING_WINDOW: usize = 10;

// Add new sample
if acc_buffer.len() >= SMOOTHING_WINDOW {
    acc_buffer.pop_front();
}
acc_buffer.push_back(reading.acc);

// Calculate smoothed value
let smoothed_acc = acc_buffer.iter().sum::<f32>() / acc_buffer.len() as f32;
```

At 10Hz sampling, this provides a 1-second smoothing window.

### 5.3 Hjorth Parameters (logic crate)

Hjorth parameters extract temporal characteristics from accelerometer signals:

```rust
pub struct SignalFeatures {
    pub mean: f64,
    pub variance: f64,
    pub stationarity_passed: bool,
    pub hjorth_activity: f64,      // Signal power (variance)
    pub hjorth_mobility: f64,      // Mean frequency
    pub hjorth_complexity: f64,    // Frequency spread
}
```

**Calculations:**

| Parameter | Formula | Interpretation |
|-----------|---------|----------------|
| **Activity** | var(signal) | Higher = more movement |
| **Mobility** | sqrt(var(1st derivative) / Activity) | Signal smoothness |
| **Complexity** | Mobility(2nd) / Mobility(1st) | Pattern irregularity |

### 5.4 Stationarity Test

Determines if signal properties are consistent over time:

```rust
pub fn check_stationarity(data: &[f64], segments: usize) -> bool {
    // Divide signal into segments
    let chunk_size = data.len() / segments;

    // Calculate variance of each segment
    let segment_variances: Vec<f64> = chunks.map(|c| variance(c)).collect();

    // If variance-of-variances is low, signal is stationary
    variance(segment_variances) < 0.05
}
```

**Application:**
- Stationary signal → User is still (sitting, lying)
- Non-stationary signal → User is changing position

---

## 6. FHIR Healthcare Integration

### 6.1 FHIR Resources

The system implements HL7 FHIR R4 Observation resources for healthcare interoperability.

**Observation Structure:**
```json
{
  "resourceType": "Observation",
  "id": "unique-id",
  "status": "final",
  "code": {
    "coding": [{
      "system": "http://loinc.org",
      "code": "87705-0",
      "display": "Sedentary activity 24 hour"
    }]
  },
  "subject": {
    "reference": "Patient/user-uuid"
  },
  "effectiveDateTime": "2026-01-28T14:30:25Z",
  "valueQuantity": {
    "value": 7.5,
    "unit": "h/(24.h)",
    "system": "http://unitsofmeasure.org"
  }
}
```

### 6.2 LOINC Codes

| Code | Display | Use |
|------|---------|-----|
| **87705-0** | Sedentary activity 24 hour | Daily sedentary time summary |
| **CUSTOM-STATE** | Sedentary State | Real-time state observation |
| **CUSTOM-TIMER** | Inactive Duration | Real-time timer observation |

### 6.3 FHIR API Endpoints

#### GET /api/fhir/observation/latest

Returns the latest sensor reading as FHIR Observations.

**Response:** Array of 2 Observations (state + timer)
```json
[
  {
    "resourceType": "Observation",
    "id": "123-state",
    "status": "final",
    "code": {"coding": [{"system": "http://loinc.org", "code": "CUSTOM-STATE"}]},
    "valueString": "SEDENTARY"
  },
  {
    "resourceType": "Observation",
    "id": "123-timer",
    "status": "final",
    "code": {"coding": [{"system": "http://loinc.org", "code": "CUSTOM-TIMER"}]},
    "valueInteger": 123
  }
]
```

#### GET /api/fhir/analytics/user/:user_id

Returns user's activity summaries as FHIR Bundle.

**Query Parameters:**
- `period`: `daily` | `weekly` | `monthly` (default: daily)
- `limit`: Number of records (default: 30)

#### GET /api/fhir/analytics/latest

Returns latest analytics for all users.

---

## 7. Authentication & Security

### 7.1 JWT Authentication

**Token Structure:**
```rust
struct Claims {
    sub: String,    // User UUID
    name: String,   // Display name
    exp: usize,     // Expiration (Unix timestamp)
}
```

**Configuration:**
- Algorithm: HS256 (HMAC SHA-256)
- Expiration: 1 hour (configurable via `JWT_EXPIRY_HOURS`)
- Secret: 64-byte hex string from `JWT_SECRET`

**Usage:**
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIs...
```

### 7.2 Password Hashing

**Algorithm:** Argon2id (OWASP-recommended)

**Parameters:**
- Memory: 19,456 KB
- Iterations: 2
- Parallelism: 1

**Storage Format (PHC):**
```
$argon2id$v=19$m=19456,t=2,p=1$[salt]$[hash]
```

### 7.3 Rate Limiting

Login attempts are rate-limited using Redis:

| Setting | Value |
|---------|-------|
| Max attempts | 5 |
| Window | 60 seconds |
| Response | HTTP 429 Too Many Requests |

**Timing Attack Mitigation:**
Even for non-existent users, password verification runs against a dummy hash to prevent user enumeration.

### 7.4 Protected Routes

| Route | Auth Required |
|-------|---------------|
| `/stats` | Yes (Bearer token) |
| All other routes | No |

---

## 8. Fallback System

### 8.1 Purpose

When Arduino hardware is unavailable (disconnected, Codespaces, demo mode), the fallback system automatically replays historical data to keep the dashboard functional.

### 8.2 Detection Mechanism

```rust
pub struct FallbackState {
    last_data_time: AtomicU64,      // Last serial data timestamp
    is_fallback_active: AtomicBool, // Currently in fallback?
}
```

**Trigger:** No data received for `FALLBACK_TIMEOUT_SECONDS` (default: 10)

### 8.3 Replay Process

1. Detect data gap (no serial data for N seconds)
2. Set `is_fallback_active = true`
3. Query last 500 records from `sedentary_log`
4. Replay to broadcast channel at configured interval
5. When real data arrives, exit fallback mode

### 8.4 Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `DISABLE_FALLBACK` | false | Set true to disable |
| `FALLBACK_TIMEOUT_SECONDS` | 10 | Seconds without data before triggering |
| `FALLBACK_BATCH_SIZE` | 500 | Records to fetch from database |
| `FALLBACK_REPLAY_INTERVAL_MS` | 100 | Milliseconds between replayed messages |

---

## 9. ML Analytics

### 9.1 Overview

A Python service runs nightly to analyze activity patterns and generate summaries.

**Schedule:** Daily at 2:00 AM (configurable via `ML_NIGHTLY_SCHEDULE`)

### 9.2 KMeans Clustering

The service uses KMeans to identify natural groupings in acceleration data:

```python
from sklearn.cluster import KMeans

kmeans = KMeans(
    n_clusters=3,       # SEDENTARY, FIDGET, ACTIVE
    random_state=42,
    n_init=10
)
df['cluster'] = kmeans.fit_predict(df[['acceleration_val']])
```

**Output:**
- Cluster centers (threshold suggestions)
- Pattern detection (cluster sizes, distributions)
- Adaptive threshold recommendations

### 9.3 Generated Metrics

| Metric | Description |
|--------|-------------|
| `activity_score` | 0-100 scale of overall activity |
| `sedentary_minutes` | Total sedentary time |
| `active_minutes` | Total active + fidget time |
| `alert_count` | Number of 20-minute sedentary alerts |
| `longest_sedentary_period` | Maximum continuous inactivity |
| `suggested_fidget_threshold` | ML-recommended fidget threshold |
| `suggested_active_threshold` | ML-recommended active threshold |

### 9.4 Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `ML_NIGHTLY_SCHEDULE` | `0 2 * * *` | Cron schedule |
| `ML_KMEANS_CLUSTERS` | 3 | Number of clusters |
| `ML_KMEANS_RANDOM_STATE` | 42 | Random seed for reproducibility |
| `ML_MIN_SAMPLES_FOR_CLUSTERING` | 100 | Minimum samples required |
| `ML_SAMPLES_PER_MINUTE` | 600 | Expected samples per minute |

---

## 10. Frontend Dashboard

### 10.1 Features

- **Real-time Acceleration Chart**: D3.js line graph with threshold lines
- **Activity Timeline**: Color-coded bar chart of state history
- **Session Summary**: Donut chart showing active vs. inactive time
- **Status Indicator**: Visual state display with animations
- **Sedentary Timer**: Live counter of inactivity duration
- **Alert System**: Visual and audio alerts for prolonged sedentary periods

### 10.2 D3.js Visualizations

**Acceleration Chart:**
- Line graph showing smoothed acceleration over time
- Yellow dashed line at THRESH_FIDGET (0.020)
- Green dashed line at THRESH_ACTIVE (0.040)
- Area fill under the line

**Timeline:**
- Horizontal bar chart
- Green = ACTIVE, Yellow = FIDGET, Red = SEDENTARY

**Summary Donut:**
- Shows percentage of active vs. inactive time
- Updates in real-time

### 10.3 Connection Handling

```javascript
const eventSource = new EventSource('/events');

eventSource.addEventListener('sensor-data', (event) => {
    const data = JSON.parse(event.data);
    updateCharts(data);
    updateStatus(data);
    checkAlerts(data);
});

eventSource.addEventListener('error', () => {
    // Show disconnected indicator
    // Browser automatically reconnects
});
```

### 10.4 Files

| File | Purpose |
|------|---------|
| `frontend/index.html` | Dashboard structure |
| `frontend/app.js` | D3.js charts and WebSocket/SSE handling |
| `frontend/styles.css` | Styling and animations |
| `frontend/login.html` | Login form |
| `frontend/signup.html` | Registration form |

---

## 11. API Reference

### 11.1 Public Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/` | Dashboard (serves index.html) |
| GET | `/health` | Health check |
| POST | `/signup` | User registration |
| POST | `/login` | JWT token generation |
| WS | `/ws` | WebSocket stream |
| GET | `/events` | SSE stream |
| GET | `/api/replay` | Start data replay |

### 11.2 FHIR Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/fhir/observation/latest` | Latest reading |
| GET | `/api/fhir/analytics/user/:id` | User analytics |
| GET | `/api/fhir/analytics/latest` | All users' latest analytics |

### 11.3 Protected Endpoints

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| GET | `/stats` | Bearer token | User statistics |

### 11.4 Request/Response Examples

**POST /login**
```bash
curl -X POST http://localhost:8000/login \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "email=user@example.com&password=SecurePass123"
```

**Response (200):**
```json
{"token":"eyJhbGciOiJIUzI1NiIs..."}
```

**Response (401):**
```
Invalid email or password.
```

**Response (429):**
```
Too many failed login attempts. Try again later.
```

---

## 12. Configuration

### 12.1 Environment Variables

Create a `.env` file in the project root:

```env
# Database
DATABASE_URL=postgres://postgres:password@localhost:5432/sedentary_data
POSTGRES_USER=postgres
POSTGRES_PASSWORD=password
POSTGRES_DB=sedentary_data
POSTGRES_HOST=localhost
POSTGRES_PORT=5432
DB_MAX_CONNECTIONS=5

# Redis
REDIS_URL=redis://localhost:6379/
REDIS_HOST=localhost
REDIS_PORT=6379
REDIS_CACHE_TTL_SECONDS=3600

# Authentication
JWT_SECRET=your-64-byte-hex-secret-here
JWT_EXPIRY_HOURS=1
RATE_LIMIT_MAX_ATTEMPTS=5
RATE_LIMIT_WINDOW_SECONDS=60

# Hardware
SERIAL_PORT=/dev/ttyACM0    # or /dev/null for no hardware
BAUD_RATE=115200
SERIAL_TIMEOUT_MS=1000

# Server
SERVER_ADDRESS=0.0.0.0:8000
FRONTEND_DIR=/app/frontend
BROADCAST_CAPACITY=100

# Activity Thresholds
THRESH_FIDGET=0.020
THRESH_ACTIVE=0.040
ALERT_LIMIT_SECONDS=1200

# Fallback
DISABLE_FALLBACK=false
FALLBACK_TIMEOUT_SECONDS=10
FALLBACK_BATCH_SIZE=500
FALLBACK_REPLAY_INTERVAL_MS=100

# ML Analytics
ML_NIGHTLY_SCHEDULE=0 2 * * *
ML_KMEANS_CLUSTERS=3
ML_SAMPLES_PER_MINUTE=600

# FHIR/LOINC
LOINC_CODE=87705-0
LOINC_DISPLAY=Sedentary activity 24 hour
LOINC_SYSTEM=http://loinc.org

# Cache
SENSOR_HISTORY_LIMIT=500

# Logging
RUST_LOG=info
```

### 12.2 Docker Environment

When running with Docker Compose, the following are automatically set:
- `DATABASE_URL` uses container hostname `db`
- `REDIS_URL` uses container hostname `redis`
- Ports are mapped from container to host

---

## 13. Deployment

### 13.1 Local Development

```bash
# Start services
docker compose up -d

# View logs
docker compose logs -f backend

# Stop services
docker compose down
```

### 13.2 GitHub Codespaces

The project includes `.devcontainer/` configuration for automatic setup:

1. Open repository in Codespaces
2. Wait for `postCreateCommand` to complete
3. Services start automatically
4. Access dashboard on forwarded port 8000

### 13.3 Production Deployment

1. Set production environment variables
2. Use strong `JWT_SECRET` (generate with `openssl rand -hex 32`)
3. Configure proper `SERIAL_PORT` or enable fallback
4. Set up TLS termination (nginx/traefik)
5. Configure log aggregation
6. Set up monitoring and alerting

### 13.4 CI/CD Pipeline

The project includes GitHub Actions workflow (`.github/workflows/ci.yml`):

**Jobs:**
1. **lint** - Format check + Clippy
2. **test** - Unit tests (errors, logic, server)
3. **test-db** - Database integration tests
4. **docker** - Build and push to GHCR (main branch only)

**Required GitHub Secrets/Variables:**
- `POSTGRES_USER` (variable)
- `POSTGRES_PASSWORD` (secret)
- `POSTGRES_DB` (variable)
- `POSTGRES_HOST` (variable)
- `POSTGRES_PORT` (variable)

---

## Appendix A: Troubleshooting

### Serial Port Issues

**Problem:** "Permission denied" on `/dev/ttyACM0`

**Solution:**
```bash
sudo usermod -a -G dialout $USER
# Logout and login again
```

### Database Connection Issues

**Problem:** "Connection refused" to PostgreSQL

**Solution:**
1. Check if container is running: `docker ps`
2. Check logs: `docker compose logs db`
3. Verify port mapping matches `.env`

### Redis Connection Issues

**Problem:** "Connection refused" to Redis

**Solution:**
1. Check if container is running: `docker ps`
2. Verify Redis port in `.env`
3. Check Redis logs: `docker compose logs redis`

### Frontend Not Loading

**Problem:** Dashboard shows blank page

**Solution:**
1. Check browser console for errors
2. Verify WebSocket/SSE connection
3. Check if backend is running: `curl http://localhost:8000/health`

---

## Appendix B: Performance Tuning

### Database

- Increase `DB_MAX_CONNECTIONS` for high-traffic scenarios
- Add indexes for common query patterns
- Consider partitioning `sedentary_log` by date

### Redis

- Adjust `SENSOR_HISTORY_LIMIT` based on memory
- Configure Redis persistence if data durability needed

### Server

- Adjust `BROADCAST_CAPACITY` for many concurrent clients
- Consider horizontal scaling with load balancer

---

## Appendix C: Security Considerations

1. **Always use HTTPS in production** (TLS termination at proxy)
2. **Rotate JWT_SECRET periodically**
3. **Use strong database passwords**
4. **Enable database connection encryption**
5. **Review rate limiting settings**
6. **Implement CORS if needed**
7. **Audit user access to FHIR endpoints**
8. **Log authentication events for audit trail**

---

*Documentation generated for Sedentary Activity Tracker v1.0*

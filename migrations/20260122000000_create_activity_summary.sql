-- Create activity_summary table for ML-generated daily/weekly/monthly statistics
-- LOINC Code: 87705-0 (Sedentary activity 24 hour)

CREATE TABLE IF NOT EXISTS activity_summary (
    id SERIAL PRIMARY KEY,
    user_id UUID REFERENCES users(user_id) ON DELETE CASCADE,
    date DATE NOT NULL,
    period_type VARCHAR(10) NOT NULL CHECK (period_type IN ('daily', 'weekly', 'monthly')),

    -- Time-based metrics (in minutes)
    sedentary_minutes REAL NOT NULL DEFAULT 0,
    fidget_minutes REAL NOT NULL DEFAULT 0,
    active_minutes REAL NOT NULL DEFAULT 0,
    total_minutes REAL NOT NULL DEFAULT 0,

    -- Percentages
    sedentary_percentage REAL NOT NULL DEFAULT 0,
    active_percentage REAL NOT NULL DEFAULT 0,

    -- State tracking
    dominant_state VARCHAR(20) NOT NULL,

    -- Activity scores (0-100)
    activity_score INTEGER NOT NULL DEFAULT 0,

    -- Alert metrics
    alert_count INTEGER NOT NULL DEFAULT 0,
    longest_sedentary_period INTEGER NOT NULL DEFAULT 0, -- in seconds

    -- Pattern detection (KMeans clusters)
    detected_patterns JSONB,
    suggested_fidget_threshold REAL,
    suggested_active_threshold REAL,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Ensure one summary per user per date per period type
    UNIQUE(user_id, date, period_type)
);

-- Index for efficient queries
CREATE INDEX idx_activity_summary_user_date ON activity_summary(user_id, date DESC);
CREATE INDEX idx_activity_summary_period ON activity_summary(period_type, date DESC);

-- Create a view for FHIR-compliant observations (LOINC 87705-0)
CREATE OR REPLACE VIEW fhir_sedentary_observations AS
SELECT
    id,
    user_id,
    date,
    period_type,
    sedentary_minutes / 60.0 AS sedentary_hours_per_day,
    (sedentary_minutes / total_minutes) * 24 AS sedentary_hours_24h_rate,
    activity_score,
    created_at
FROM activity_summary
WHERE period_type = 'daily'
ORDER BY date DESC;

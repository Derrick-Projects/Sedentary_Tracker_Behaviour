-- Create sensor_data table for user-level sensor readings
-- Mirrors processed readings from sedentary_log but linked to specific users

CREATE TABLE IF NOT EXISTS sensor_data (
    id SERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    state VARCHAR(20) NOT NULL,
    timer_seconds INTEGER NOT NULL DEFAULT 0,
    acceleration_val REAL NOT NULL DEFAULT 0.0,
    alert_triggered BOOLEAN NOT NULL DEFAULT FALSE,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_sensor_data_user_created ON sensor_data(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_sensor_data_user_state ON sensor_data(user_id, state);

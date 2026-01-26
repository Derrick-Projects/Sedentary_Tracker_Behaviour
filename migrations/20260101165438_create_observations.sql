DROP TABLE IF EXISTS sensor_readings; 

CREATE TABLE IF NOT EXISTS sedentary_log (
    id SERIAL PRIMARY KEY,
    state VARCHAR(20) NOT NULL,   
    timer_seconds INTEGER,        
    acceleration_val REAL,        
    created_at TIMESTAMPTZ DEFAULT NOW()
);
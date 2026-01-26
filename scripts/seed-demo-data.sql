-- Demo data for Sedentary Tracker
-- Simulates activity data with realistic patterns

-- Clear existing demo data (if any)
TRUNCATE sedentary_log RESTART IDENTITY;

-- Create demo user (email: demo@example.com, password: password)
INSERT INTO users (user_id, email, password_hash, name, created_at) VALUES
('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'demo@example.com', '$argon2id$v=19$m=65536,t=3,p=4$c29tZXNhbHQ$RdescudvJCsgt3ub+b+dWRWJTmaaJObG', 'Demo User', NOW())
ON CONFLICT (email) DO NOTHING;

-- Insert demo data with varied activity patterns
-- Morning session (mostly sedentary with occasional movement)
INSERT INTO sedentary_log (state, timer_seconds, acceleration_val, created_at) VALUES
('Sedentary', 0, 0.005, NOW() - INTERVAL '4 hours'),
('Sedentary', 60, 0.008, NOW() - INTERVAL '4 hours' + INTERVAL '1 minute'),
('Sedentary', 120, 0.006, NOW() - INTERVAL '4 hours' + INTERVAL '2 minutes'),
('Sedentary', 180, 0.007, NOW() - INTERVAL '4 hours' + INTERVAL '3 minutes'),
('Sedentary', 240, 0.009, NOW() - INTERVAL '4 hours' + INTERVAL '4 minutes'),
('Fidget', 0, 0.025, NOW() - INTERVAL '4 hours' + INTERVAL '5 minutes'),
('Active', 0, 0.055, NOW() - INTERVAL '4 hours' + INTERVAL '6 minutes'),
('Active', 0, 0.062, NOW() - INTERVAL '4 hours' + INTERVAL '7 minutes'),
('Sedentary', 0, 0.012, NOW() - INTERVAL '4 hours' + INTERVAL '8 minutes'),
('Sedentary', 60, 0.008, NOW() - INTERVAL '4 hours' + INTERVAL '9 minutes'),
('Sedentary', 120, 0.006, NOW() - INTERVAL '4 hours' + INTERVAL '10 minutes'),
('Sedentary', 180, 0.005, NOW() - INTERVAL '4 hours' + INTERVAL '11 minutes'),
('Sedentary', 240, 0.007, NOW() - INTERVAL '4 hours' + INTERVAL '12 minutes'),
('Sedentary', 300, 0.008, NOW() - INTERVAL '4 hours' + INTERVAL '13 minutes'),
('Sedentary', 360, 0.006, NOW() - INTERVAL '4 hours' + INTERVAL '14 minutes'),
('Sedentary', 420, 0.009, NOW() - INTERVAL '4 hours' + INTERVAL '15 minutes'),
('Fidget', 0, 0.028, NOW() - INTERVAL '4 hours' + INTERVAL '16 minutes'),
('Fidget', 0, 0.032, NOW() - INTERVAL '4 hours' + INTERVAL '17 minutes'),
('Sedentary', 0, 0.011, NOW() - INTERVAL '4 hours' + INTERVAL '18 minutes'),
('Sedentary', 60, 0.007, NOW() - INTERVAL '4 hours' + INTERVAL '19 minutes');

-- Mid-morning (long sedentary period with alert)
INSERT INTO sedentary_log (state, timer_seconds, acceleration_val, created_at) VALUES
('Sedentary', 0, 0.006, NOW() - INTERVAL '3 hours'),
('Sedentary', 60, 0.005, NOW() - INTERVAL '3 hours' + INTERVAL '1 minute'),
('Sedentary', 120, 0.007, NOW() - INTERVAL '3 hours' + INTERVAL '2 minutes'),
('Sedentary', 180, 0.006, NOW() - INTERVAL '3 hours' + INTERVAL '3 minutes'),
('Sedentary', 240, 0.008, NOW() - INTERVAL '3 hours' + INTERVAL '4 minutes'),
('Sedentary', 300, 0.005, NOW() - INTERVAL '3 hours' + INTERVAL '5 minutes'),
('Sedentary', 360, 0.006, NOW() - INTERVAL '3 hours' + INTERVAL '6 minutes'),
('Sedentary', 420, 0.007, NOW() - INTERVAL '3 hours' + INTERVAL '7 minutes'),
('Sedentary', 480, 0.005, NOW() - INTERVAL '3 hours' + INTERVAL '8 minutes'),
('Sedentary', 540, 0.006, NOW() - INTERVAL '3 hours' + INTERVAL '9 minutes'),
('Sedentary', 600, 0.008, NOW() - INTERVAL '3 hours' + INTERVAL '10 minutes'),
('Sedentary', 660, 0.005, NOW() - INTERVAL '3 hours' + INTERVAL '11 minutes'),
('Sedentary', 720, 0.006, NOW() - INTERVAL '3 hours' + INTERVAL '12 minutes'),
('Sedentary', 780, 0.007, NOW() - INTERVAL '3 hours' + INTERVAL '13 minutes'),
('Sedentary', 840, 0.006, NOW() - INTERVAL '3 hours' + INTERVAL '14 minutes'),
('Sedentary', 900, 0.005, NOW() - INTERVAL '3 hours' + INTERVAL '15 minutes'),
('Sedentary', 960, 0.008, NOW() - INTERVAL '3 hours' + INTERVAL '16 minutes'),
('Sedentary', 1020, 0.006, NOW() - INTERVAL '3 hours' + INTERVAL '17 minutes'),
('Sedentary', 1080, 0.007, NOW() - INTERVAL '3 hours' + INTERVAL '18 minutes'),
('Sedentary', 1140, 0.005, NOW() - INTERVAL '3 hours' + INTERVAL '19 minutes'),
('Sedentary', 1200, 0.006, NOW() - INTERVAL '3 hours' + INTERVAL '20 minutes'),
('Active', 0, 0.075, NOW() - INTERVAL '3 hours' + INTERVAL '21 minutes'),
('Active', 0, 0.068, NOW() - INTERVAL '3 hours' + INTERVAL '22 minutes'),
('Active', 0, 0.072, NOW() - INTERVAL '3 hours' + INTERVAL '23 minutes');

-- Lunch break (active period)
INSERT INTO sedentary_log (state, timer_seconds, acceleration_val, created_at) VALUES
('Active', 0, 0.065, NOW() - INTERVAL '2 hours'),
('Active', 0, 0.058, NOW() - INTERVAL '2 hours' + INTERVAL '1 minute'),
('Active', 0, 0.071, NOW() - INTERVAL '2 hours' + INTERVAL '2 minutes'),
('Fidget', 0, 0.035, NOW() - INTERVAL '2 hours' + INTERVAL '3 minutes'),
('Active', 0, 0.062, NOW() - INTERVAL '2 hours' + INTERVAL '4 minutes'),
('Active', 0, 0.055, NOW() - INTERVAL '2 hours' + INTERVAL '5 minutes'),
('Fidget', 0, 0.028, NOW() - INTERVAL '2 hours' + INTERVAL '6 minutes'),
('Active', 0, 0.068, NOW() - INTERVAL '2 hours' + INTERVAL '7 minutes'),
('Active', 0, 0.072, NOW() - INTERVAL '2 hours' + INTERVAL '8 minutes'),
('Active', 0, 0.059, NOW() - INTERVAL '2 hours' + INTERVAL '9 minutes');

-- Afternoon session (mixed activity)
INSERT INTO sedentary_log (state, timer_seconds, acceleration_val, created_at) VALUES
('Sedentary', 0, 0.008, NOW() - INTERVAL '1 hour'),
('Sedentary', 60, 0.006, NOW() - INTERVAL '1 hour' + INTERVAL '1 minute'),
('Sedentary', 120, 0.007, NOW() - INTERVAL '1 hour' + INTERVAL '2 minutes'),
('Fidget', 0, 0.025, NOW() - INTERVAL '1 hour' + INTERVAL '3 minutes'),
('Sedentary', 0, 0.009, NOW() - INTERVAL '1 hour' + INTERVAL '4 minutes'),
('Sedentary', 60, 0.006, NOW() - INTERVAL '1 hour' + INTERVAL '5 minutes'),
('Sedentary', 120, 0.008, NOW() - INTERVAL '1 hour' + INTERVAL '6 minutes'),
('Sedentary', 180, 0.005, NOW() - INTERVAL '1 hour' + INTERVAL '7 minutes'),
('Active', 0, 0.052, NOW() - INTERVAL '1 hour' + INTERVAL '8 minutes'),
('Sedentary', 0, 0.011, NOW() - INTERVAL '1 hour' + INTERVAL '9 minutes'),
('Sedentary', 60, 0.007, NOW() - INTERVAL '1 hour' + INTERVAL '10 minutes'),
('Sedentary', 120, 0.006, NOW() - INTERVAL '1 hour' + INTERVAL '11 minutes'),
('Fidget', 0, 0.031, NOW() - INTERVAL '1 hour' + INTERVAL '12 minutes'),
('Fidget', 0, 0.027, NOW() - INTERVAL '1 hour' + INTERVAL '13 minutes'),
('Sedentary', 0, 0.008, NOW() - INTERVAL '1 hour' + INTERVAL '14 minutes'),
('Sedentary', 60, 0.006, NOW() - INTERVAL '1 hour' + INTERVAL '15 minutes');

-- Recent data (last 30 minutes)
INSERT INTO sedentary_log (state, timer_seconds, acceleration_val, created_at) VALUES
('Sedentary', 0, 0.007, NOW() - INTERVAL '30 minutes'),
('Sedentary', 60, 0.005, NOW() - INTERVAL '29 minutes'),
('Sedentary', 120, 0.008, NOW() - INTERVAL '28 minutes'),
('Sedentary', 180, 0.006, NOW() - INTERVAL '27 minutes'),
('Sedentary', 240, 0.007, NOW() - INTERVAL '26 minutes'),
('Sedentary', 300, 0.005, NOW() - INTERVAL '25 minutes'),
('Fidget', 0, 0.029, NOW() - INTERVAL '24 minutes'),
('Sedentary', 0, 0.009, NOW() - INTERVAL '23 minutes'),
('Sedentary', 60, 0.006, NOW() - INTERVAL '22 minutes'),
('Sedentary', 120, 0.008, NOW() - INTERVAL '21 minutes'),
('Active', 0, 0.058, NOW() - INTERVAL '20 minutes'),
('Active', 0, 0.065, NOW() - INTERVAL '19 minutes'),
('Sedentary', 0, 0.011, NOW() - INTERVAL '18 minutes'),
('Sedentary', 60, 0.007, NOW() - INTERVAL '17 minutes'),
('Sedentary', 120, 0.006, NOW() - INTERVAL '16 minutes'),
('Sedentary', 180, 0.008, NOW() - INTERVAL '15 minutes'),
('Sedentary', 240, 0.005, NOW() - INTERVAL '14 minutes'),
('Sedentary', 300, 0.007, NOW() - INTERVAL '13 minutes'),
('Sedentary', 360, 0.006, NOW() - INTERVAL '12 minutes'),
('Sedentary', 420, 0.008, NOW() - INTERVAL '11 minutes'),
('Fidget', 0, 0.033, NOW() - INTERVAL '10 minutes'),
('Sedentary', 0, 0.009, NOW() - INTERVAL '9 minutes'),
('Sedentary', 60, 0.006, NOW() - INTERVAL '8 minutes'),
('Sedentary', 120, 0.007, NOW() - INTERVAL '7 minutes'),
('Sedentary', 180, 0.005, NOW() - INTERVAL '6 minutes'),
('Sedentary', 240, 0.008, NOW() - INTERVAL '5 minutes'),
('Sedentary', 300, 0.006, NOW() - INTERVAL '4 minutes'),
('Sedentary', 360, 0.007, NOW() - INTERVAL '3 minutes'),
('Fidget', 0, 0.026, NOW() - INTERVAL '2 minutes'),
('Sedentary', 0, 0.010, NOW() - INTERVAL '1 minute');

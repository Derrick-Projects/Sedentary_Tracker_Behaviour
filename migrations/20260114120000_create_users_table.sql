-- Create users table for registration/login
CREATE TABLE IF NOT EXISTS users (
    user_id UUID PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

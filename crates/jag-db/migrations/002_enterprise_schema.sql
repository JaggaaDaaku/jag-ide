-- migrations/002_enterprise_schema.sql

-- Enterprise Users
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    role TEXT NOT NULL CHECK(role IN ('Admin', 'Developer', 'Viewer')) DEFAULT 'Developer',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- User Sessions / Refresh Tokens
CREATE TABLE user_sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    refresh_token TEXT NOT NULL UNIQUE,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Model Usage Tracking
CREATE TABLE model_usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    model_name TEXT NOT NULL,
    prompt_tokens INTEGER DEFAULT 0,
    completion_tokens INTEGER DEFAULT 0,
    total_tokens INTEGER DEFAULT 0,
    cost_estimated REAL DEFAULT 0,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    is_local INTEGER DEFAULT 1
);

-- Enhance Audit Log with Signatures
ALTER TABLE audit_log ADD COLUMN signature TEXT;

-- Index for analytics
CREATE INDEX idx_usage_timestamp ON model_usage(timestamp);
CREATE INDEX idx_usage_model ON model_usage(model_name);

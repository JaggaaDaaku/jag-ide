-- migrations/001_initial_schema.sql

CREATE TABLE workspaces (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    root_path TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    modified_at TEXT NOT NULL DEFAULT (datetime('now')),
    settings_json TEXT DEFAULT '{}'
);

CREATE TABLE agents (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK(role IN ('Planner','Backend','Frontend','Integration')),
    status TEXT NOT NULL CHECK(status IN ('Idle','Working','Completed','Error')) DEFAULT 'Idle',
    model_id TEXT,
    security_tier TEXT CHECK(security_tier IN ('Off','Auto','Turbo')) DEFAULT 'Auto',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    modified_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    agent_id TEXT REFERENCES agents(id) ON DELETE SET NULL,
    task_type TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('Pending','Running','Completed','Failed','Cancelled')) DEFAULT 'Pending',
    priority TEXT DEFAULT 'Normal',
    payload_json TEXT DEFAULT '{}',
    result_json TEXT,
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    started_at TEXT,
    completed_at TEXT
);

CREATE TABLE task_dependencies (
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    depends_on_task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    dependency_type TEXT CHECK(dependency_type IN ('Hard','Soft','Parallel')),
    PRIMARY KEY(task_id, depends_on_task_id)
);

CREATE TABLE artifacts (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    agent_id TEXT REFERENCES agents(id) ON DELETE SET NULL,
    task_id TEXT REFERENCES tasks(id) ON DELETE SET NULL,
    artifact_type TEXT NOT NULL,
    content_path TEXT,
    content_hash TEXT,
    file_size INTEGER,
    format TEXT,
    verification_status TEXT CHECK(verification_status IN ('Pending','Approved','Rejected')) DEFAULT 'Pending',
    metadata_json TEXT DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    modified_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE models (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    provider TEXT CHECK(provider IN ('local','anthropic','openai','google','mistral')),
    model_type TEXT,
    capabilities_json TEXT,
    context_window INTEGER,
    quantization_level TEXT,
    status TEXT DEFAULT 'available',
    cost_per_1k_tokens REAL DEFAULT 0,
    created_at TEXT DEFAULT (datetime('now')),
    modified_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE agent_model_assignments (
    agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    model_id TEXT NOT NULL REFERENCES models(id) ON DELETE CASCADE,
    assigned_at TEXT DEFAULT (datetime('now')),
    is_sticky_brain INTEGER DEFAULT 0,
    PRIMARY KEY(agent_id, model_id)
);

CREATE TABLE audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id TEXT,
    agent_id TEXT,
    user_id TEXT,
    action_type TEXT NOT NULL,
    resource_type TEXT,
    resource_id TEXT,
    details_json TEXT,
    ip_address TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE file_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id TEXT REFERENCES workspaces(id),
    file_path TEXT NOT NULL,
    content_hash TEXT,
    last_modified TEXT,
    size INTEGER,
    cached_at TEXT DEFAULT (datetime('now')),
    accessed_at TEXT,
    UNIQUE(workspace_id, file_path)
);

CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value_json TEXT NOT NULL,
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX idx_agents_workspace ON agents(workspace_id);
CREATE INDEX idx_tasks_agent ON tasks(agent_id);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_artifacts_workspace ON artifacts(workspace_id);
CREATE INDEX idx_artifacts_agent ON artifacts(agent_id);
CREATE INDEX idx_audit_workspace ON audit_log(workspace_id);
CREATE INDEX idx_audit_created ON audit_log(created_at);

INSERT INTO settings (key, value_json) VALUES
  ('security.default_tier', '"Auto"'),
  ('performance.max_concurrent_agents', '4'),
  ('models.local_endpoint', '"http://localhost:11434"'),
  ('workspace.default_root', '"~/jag-workspaces"');

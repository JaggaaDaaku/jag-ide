-- crates/jag-db/migrations/006_workspace_sharing.sql

CREATE TABLE IF NOT EXISTS workspace_members (
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK(role IN ('Admin', 'Developer', 'Viewer')),
    joined_at DATETIME NOT NULL DEFAULT (datetime('now')),
    invited_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    PRIMARY KEY(workspace_id, user_id)
);

CREATE TABLE IF NOT EXISTS workspace_invites (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    email TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('Admin', 'Developer', 'Viewer')),
    invited_by TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at DATETIME NOT NULL DEFAULT (datetime('now')),
    expires_at DATETIME NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('Pending', 'Accepted', 'Expired')) DEFAULT 'Pending'
);

-- Add versioning to artifacts for conflict resolution
ALTER TABLE artifacts ADD COLUMN version INTEGER DEFAULT 1;

CREATE INDEX idx_workspace_members_workspace ON workspace_members(workspace_id);
CREATE INDEX idx_workspace_members_user ON workspace_members(user_id);
CREATE INDEX idx_workspace_invites_email ON workspace_invites(email);

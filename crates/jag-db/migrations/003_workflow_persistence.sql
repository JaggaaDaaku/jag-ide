-- migrations/003_workflow_persistence.sql

CREATE TABLE workflows (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'Active',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    metadata_json TEXT DEFAULT '{}'
);

-- Add workflow_id to tasks to group them
ALTER TABLE tasks ADD COLUMN workflow_id TEXT REFERENCES workflows(id) ON DELETE CASCADE;

CREATE TABLE workflow_approvals (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    decision TEXT NOT NULL, -- 'AutoApprove', 'ApproveWithNotice', 'RequiresApproval'
    confidence REAL NOT NULL,
    reasoning TEXT,
    suggested_fixes_json TEXT DEFAULT '[]',
    reviewer_id TEXT, -- For manual approvals
    reviewed_at TEXT,
    review_deadline TEXT, -- For ApproveWithNotice
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_workflows_workspace ON workflows(workspace_id);
CREATE INDEX idx_approvals_workflow ON workflow_approvals(workflow_id);
CREATE INDEX idx_approvals_task ON workflow_approvals(task_id);
CREATE INDEX idx_tasks_workflow ON tasks(workflow_id);

-- Link artifacts to approvals for richer verification history
ALTER TABLE artifacts ADD COLUMN approval_id TEXT REFERENCES workflow_approvals(id) ON DELETE SET NULL;

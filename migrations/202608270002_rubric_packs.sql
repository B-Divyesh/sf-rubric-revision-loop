CREATE TABLE rubric_packs (
    token TEXT PRIMARY KEY,
    workspace_key TEXT NOT NULL,
    rubric_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_pack_workspace ON rubric_packs(workspace_key, created_at);

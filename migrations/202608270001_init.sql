CREATE TABLE rubric_codes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_key TEXT NOT NULL,
    code TEXT NOT NULL COLLATE NOCASE,
    title TEXT NOT NULL,
    guidance TEXT NOT NULL,
    next_step TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(workspace_key, code)
);

CREATE INDEX idx_rubric_workspace ON rubric_codes(workspace_key, created_at);

CREATE TABLE feedback_loops (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    token TEXT NOT NULL UNIQUE,
    workspace_key TEXT NOT NULL,
    student_label TEXT NOT NULL DEFAULT '',
    assignment_title TEXT NOT NULL,
    teacher_note TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'awaiting' CHECK(status IN ('awaiting', 'submitted', 'reviewed')),
    before_excerpt TEXT,
    after_excerpt TEXT,
    explanation TEXT,
    checklist_json TEXT NOT NULL DEFAULT '[]',
    retention_days INTEGER NOT NULL DEFAULT 30,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    submitted_at TEXT,
    reviewed_at TEXT,
    deleted_at TEXT
);

CREATE INDEX idx_loop_workspace ON feedback_loops(workspace_key, status, created_at);
CREATE INDEX idx_loop_token ON feedback_loops(token);

CREATE TABLE loop_codes (
    loop_id INTEGER NOT NULL REFERENCES feedback_loops(id) ON DELETE CASCADE,
    rubric_id INTEGER NOT NULL REFERENCES rubric_codes(id) ON DELETE RESTRICT,
    PRIMARY KEY(loop_id, rubric_id)
);

PRAGMA foreign_keys = ON;


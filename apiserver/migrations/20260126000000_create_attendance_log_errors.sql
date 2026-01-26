CREATE TABLE IF NOT EXISTS attendance_log_errors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    agent_id TEXT,
    raw_content JSONB,
    error_message TEXT NOT NULL,
    source TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

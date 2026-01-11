-- Up Migration
CREATE TABLE IF NOT EXISTS attendance_import_jobs (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    user_id UUID NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'processing', -- 'processing', 'completed', 'failed'
    total_rows INT NOT NULL DEFAULT 0,
    processed_rows INT NOT NULL DEFAULT 0,
    error_log TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_attendance_import_jobs_workspace_id ON attendance_import_jobs(workspace_id);
CREATE INDEX IF NOT EXISTS idx_attendance_import_jobs_status ON attendance_import_jobs(status);

-- Down Migration
-- DROP TABLE attendance_import_jobs;

-- Up Migration: Create raw_attendance_logs table
CREATE TABLE IF NOT EXISTS raw_attendance_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    punch_time TIMESTAMPTZ NOT NULL,
    source VARCHAR(50) NOT NULL, -- 'device', 'import', 'manual'
    device_id VARCHAR(50), -- Optional device identifier
    raw_data JSONB, -- Store original record for audit
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_raw_logs_workspace_id ON raw_attendance_logs(workspace_id);
CREATE INDEX IF NOT EXISTS idx_raw_logs_employee_id ON raw_attendance_logs(employee_id);
CREATE INDEX IF NOT EXISTS idx_raw_logs_punch_time ON raw_attendance_logs(punch_time);

-- Down Migration
-- DROP TABLE raw_attendance_logs;

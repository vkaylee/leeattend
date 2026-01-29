-- Up Migration: Create Attendance Daily Summaries for OT Workflow

-- Enum for OT Status
CREATE TYPE ot_status_type AS ENUM ('none', 'pending_employee', 'pending_manager', 'approved', 'rejected');

CREATE TABLE IF NOT EXISTS attendance_daily_summaries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    date DATE NOT NULL,
    
    -- Calculated values (System of Record for time)
    actual_work_minutes INTEGER NOT NULL DEFAULT 0,
    system_detected_ot_minutes INTEGER NOT NULL DEFAULT 0,
    
    -- Workflow values (User input)
    claimed_ot_minutes INTEGER NOT NULL DEFAULT 0,
    approved_ot_minutes INTEGER NOT NULL DEFAULT 0,
    
    ot_status ot_status_type NOT NULL DEFAULT 'none',
    ot_reason TEXT,
    manager_note TEXT,
    
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(employee_id, date)
);

CREATE INDEX IF NOT EXISTS idx_daily_summaries_workspace_date ON attendance_daily_summaries(workspace_id, date);
CREATE INDEX IF NOT EXISTS idx_daily_summaries_status ON attendance_daily_summaries(ot_status);

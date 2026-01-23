-- Up Migration: Configurable Overtime Policies

-- 1. Overtime Policy Definitions
CREATE TABLE IF NOT EXISTS overtime_policy_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 2. Overtime Rules
CREATE TABLE IF NOT EXISTS overtime_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_id UUID NOT NULL REFERENCES overtime_policy_definitions(id) ON DELETE CASCADE,
    priority INTEGER NOT NULL DEFAULT 0,
    multiplier NUMERIC(5, 2) NOT NULL DEFAULT 1.50,
    
    -- Conditions
    time_start TIME, -- NULL means 00:00:00
    time_end TIME,   -- NULL means 23:59:59.999
    days_of_week INTEGER NOT NULL DEFAULT 127,
    is_holiday BOOLEAN NOT NULL DEFAULT FALSE,
    
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 3. Overtime Policy Assignments (Temporal)
CREATE TABLE IF NOT EXISTS overtime_policy_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    
    assignee_type TEXT NOT NULL CHECK (assignee_type IN ('workspace', 'department', 'employee')),
    assignee_id UUID,
    
    policy_id UUID NOT NULL REFERENCES overtime_policy_definitions(id) ON DELETE RESTRICT,
    
    effective_from DATE NOT NULL,
    effective_to DATE,
    
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    
    EXCLUDE USING GIST (
        workspace_id WITH =, 
        assignee_type WITH =, 
        COALESCE(assignee_id, '00000000-0000-0000-0000-000000000000'::uuid) WITH =, 
        daterange(effective_from, effective_to, '[)') WITH &&
    )
);

CREATE INDEX IF NOT EXISTS idx_overtime_policy_assignments_scope ON overtime_policy_assignments(workspace_id, assignee_type, assignee_id);

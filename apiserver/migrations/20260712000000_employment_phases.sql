-- Up Migration: Generic Employment Phase System
-- Replaces hardcoded probation with configurable phases (probation, notice, training, custom)
-- Pattern: Same as leave_policy_definitions → leave_policy_assignments

-- 1. Employment Phase Definitions (workspace-scoped templates)
CREATE TABLE IF NOT EXISTS employment_phase_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    phase_type TEXT NOT NULL DEFAULT 'custom',
    -- 'probation', 'notice_period', 'training', 'onboarding', 'custom'

    -- Flexible configuration per phase type
    -- probation: { "default_duration_days": 60, "salary_percentage": 85.0, "auto_complete": false, "max_extensions": 1 }
    -- notice_period: { "default_duration_days": 30, "salary_percentage": 100.0 }
    -- training: { "default_duration_days": 90, "requires_certification": true }
    rule_config JSONB NOT NULL DEFAULT '{}'::jsonb,

    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(workspace_id, name)
);

CREATE INDEX idx_employment_phase_defs_workspace
    ON employment_phase_definitions(workspace_id);
CREATE INDEX idx_employment_phase_defs_type
    ON employment_phase_definitions(workspace_id, phase_type);

-- 2. Employee Phases (actual tracking of employees going through phases)
CREATE TABLE IF NOT EXISTS employee_phases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    phase_definition_id UUID NOT NULL REFERENCES employment_phase_definitions(id) ON DELETE RESTRICT,

    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    -- active: currently in this phase
    -- completed: successfully passed
    -- failed: did not pass (e.g., probation failed)
    -- extended: was extended (new row created for extension)
    -- cancelled: manually cancelled
    status TEXT NOT NULL DEFAULT 'active',

    -- Override values from definition (NULL = use definition defaults)
    override_config JSONB,
    -- e.g. { "salary_percentage": 90.0 } to override the 85% default

    evaluation_notes TEXT,
    evaluated_by UUID REFERENCES users(id) ON DELETE SET NULL,
    completed_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CHECK (end_date >= start_date)
);

CREATE INDEX idx_employee_phases_workspace
    ON employee_phases(workspace_id);
CREATE INDEX idx_employee_phases_employee
    ON employee_phases(employee_id);
CREATE INDEX idx_employee_phases_definition
    ON employee_phases(phase_definition_id);
CREATE INDEX idx_employee_phases_status
    ON employee_phases(workspace_id, status);
CREATE INDEX idx_employee_phases_active_lookup
    ON employee_phases(employee_id, status, start_date, end_date)
    WHERE status = 'active';

-- OT Request status enum
CREATE TYPE ot_request_status_type AS ENUM (
    'draft',
    'pending_approval',
    'approved',
    'partially_accepted',
    'active',
    'completed',
    'cancelled',
    'rejected'
);

-- Pre-OT planning requests created by managers
CREATE TABLE overtime_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,

    -- Creator
    requested_by_user_id UUID NOT NULL,
    department_id UUID REFERENCES departments(id),

    -- Planning details
    planned_date DATE NOT NULL,
    planned_start_time TIME,
    planned_end_time TIME,
    planned_minutes INT NOT NULL CHECK (planned_minutes > 0),
    reason TEXT NOT NULL,

    -- Status & approval progress
    status ot_request_status_type NOT NULL DEFAULT 'draft',
    workflow_id UUID REFERENCES approval_workflows(id),
    current_step INT NOT NULL DEFAULT 0,  -- 0 = not submitted, 1+ = in approval

    -- Quota warning snapshot (computed at creation, for display only)
    quota_warning JSONB,

    -- Reconciliation
    reconciled_at TIMESTAMPTZ,
    reconciled_by UUID,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_otr_workspace_date ON overtime_requests(workspace_id, planned_date);
CREATE INDEX idx_otr_status ON overtime_requests(workspace_id, status);
CREATE INDEX idx_otr_dept ON overtime_requests(department_id);
CREATE INDEX idx_otr_requested_by ON overtime_requests(requested_by_user_id);

-- Employees assigned to an OT request with consent tracking
CREATE TABLE overtime_request_employees (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    overtime_request_id UUID NOT NULL REFERENCES overtime_requests(id) ON DELETE CASCADE,
    employee_id UUID NOT NULL REFERENCES employees(id),

    -- Employee consent
    consent_status VARCHAR(20) NOT NULL DEFAULT 'pending'
        CHECK (consent_status IN ('pending', 'accepted', 'declined')),
    consent_at TIMESTAMPTZ,
    decline_reason TEXT,

    -- Reconciliation (filled after OT occurs)
    actual_ot_minutes INT,
    matched_summary_id UUID REFERENCES attendance_daily_summaries(id),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(overtime_request_id, employee_id)
);

CREATE INDEX idx_otre_employee ON overtime_request_employees(employee_id);
CREATE INDEX idx_otre_consent ON overtime_request_employees(consent_status);
CREATE INDEX idx_otre_request ON overtime_request_employees(overtime_request_id);

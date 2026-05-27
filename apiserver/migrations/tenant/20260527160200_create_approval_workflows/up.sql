-- Generic approval workflow engine
-- Supports N-level configurable approval for OT requests, leave, payroll, etc.

-- Workflow definitions
CREATE TABLE approval_workflows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,

    -- Generic entity type: 'ot_request', 'leave_request', 'attendance_adjustment', etc.
    entity_type VARCHAR(50) NOT NULL,

    name VARCHAR(255) NOT NULL,
    description TEXT,
    total_steps INT NOT NULL CHECK (total_steps >= 1),
    is_active BOOLEAN NOT NULL DEFAULT true,

    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Only one active workflow per entity_type per workspace
CREATE UNIQUE INDEX idx_aw_active_unique
    ON approval_workflows(workspace_id, entity_type) WHERE (is_active = true);

CREATE INDEX idx_aw_workspace ON approval_workflows(workspace_id);

-- Individual steps within a workflow
CREATE TABLE approval_workflow_steps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID NOT NULL REFERENCES approval_workflows(id) ON DELETE CASCADE,

    step_order INT NOT NULL CHECK (step_order >= 1),
    step_name VARCHAR(100) NOT NULL,

    -- Who can approve this step?
    -- 'department_manager' | 'role' | 'permission' | 'specific_user'
    approver_type VARCHAR(30) NOT NULL
        CHECK (approver_type IN (
            'department_manager',
            'role',
            'permission',
            'specific_user'
        )),

    -- Type-specific config:
    --   department_manager: {}
    --   role:               { "role_name": "hr_manager" }
    --   permission:         { "permission": "overtime:approve" }
    --   specific_user:      { "user_id": "uuid-here" }
    approver_config JSONB NOT NULL DEFAULT '{}',

    -- 'approve' | 'review' | 'acknowledge'
    action_required VARCHAR(20) NOT NULL DEFAULT 'approve'
        CHECK (action_required IN ('approve', 'review', 'acknowledge')),

    -- false = BLOCK if approver can't be resolved (e.g. no department manager)
    -- true  = skip step with audit log entry
    can_skip BOOLEAN NOT NULL DEFAULT false,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(workflow_id, step_order)
);

CREATE INDEX idx_aws_workflow ON approval_workflow_steps(workflow_id, step_order);

-- Generic audit trail for all approval actions across entity types
CREATE TABLE approval_action_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,

    -- Which entity and which instance
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,

    step_order INT NOT NULL,
    step_name VARCHAR(100),

    acted_by_user_id UUID NOT NULL,
    action VARCHAR(20) NOT NULL
        CHECK (action IN ('approved', 'rejected', 'skipped', 'returned')),
    note TEXT,

    acted_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_aal_entity ON approval_action_log(entity_type, entity_id);
CREATE INDEX idx_aal_workspace ON approval_action_log(workspace_id, acted_at);

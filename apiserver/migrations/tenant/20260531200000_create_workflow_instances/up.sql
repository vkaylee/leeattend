-- Create approval_workflow_instances table
CREATE TABLE approval_workflow_instances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    workflow_id UUID REFERENCES approval_workflows(id) ON DELETE SET NULL,
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    current_step_order INT NOT NULL DEFAULT 1,
    steps_snapshot JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create approval_workflow_tasks table
CREATE TABLE approval_workflow_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    instance_id UUID NOT NULL REFERENCES approval_workflow_instances(id) ON DELETE CASCADE,
    step_order INT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    
    -- Assignee Criteria (Late Binding)
    assignee_type VARCHAR(30) NOT NULL,
    assignee_target_id UUID,
    assignee_target_name VARCHAR(100),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    completed_by UUID
);

CREATE INDEX idx_awi_entity ON approval_workflow_instances(entity_type, entity_id);
CREATE INDEX idx_awi_workspace ON approval_workflow_instances(workspace_id);

CREATE INDEX idx_awt_assignee ON approval_workflow_tasks(assignee_type, assignee_target_id);
CREATE INDEX idx_awt_instance ON approval_workflow_tasks(instance_id);
CREATE INDEX idx_awt_status ON approval_workflow_tasks(status);

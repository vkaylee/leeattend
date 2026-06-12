-- 1. Lifecycle Checklist Templates (reusable)
CREATE TABLE lifecycle_checklist_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    checklist_type VARCHAR(20) NOT NULL,      -- 'onboarding', 'offboarding', 'transfer', 'probation_end'
    is_default BOOLEAN NOT NULL DEFAULT FALSE, -- auto-trigger template for this type
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_lct_workspace_type ON lifecycle_checklist_templates(workspace_id, checklist_type)
    WHERE deleted_at IS NULL;
-- Enforce only 1 default per workspace per type
CREATE UNIQUE INDEX idx_lct_unique_default
    ON lifecycle_checklist_templates(workspace_id, checklist_type)
    WHERE is_default = TRUE AND deleted_at IS NULL;

-- 2. Template Items (ordered tasks)
CREATE TABLE lifecycle_checklist_template_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES lifecycle_checklist_templates(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    category VARCHAR(50) NOT NULL,             -- 'hr_admin', 'it_setup', 'documentation', 'training', 'equipment'
    assignee_type VARCHAR(20) NOT NULL,        -- 'hr_manager', 'direct_manager', 'it_admin', 'employee', 'specific_user'
    assignee_user_id UUID,                     -- for 'specific_user' type
    is_required BOOLEAN NOT NULL DEFAULT TRUE,
    due_days_offset INT NOT NULL DEFAULT 0,    -- days relative to employment start_date (negative = before)
    display_order INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_lcti_template ON lifecycle_checklist_template_items(template_id);

-- 3. Lifecycle Checklist Instances (per employee)
CREATE TABLE lifecycle_checklists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    employment_id UUID NOT NULL,
    template_id UUID REFERENCES lifecycle_checklist_templates(id),
    checklist_type VARCHAR(20) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'in_progress',  -- 'in_progress', 'completed', 'cancelled'
    progress_percent INT NOT NULL DEFAULT 0,             -- cached: (completed_items / total_items) * 100
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_lc_workspace_type ON lifecycle_checklists(workspace_id, checklist_type);
CREATE INDEX idx_lc_employment ON lifecycle_checklists(workspace_id, employment_id);
CREATE INDEX idx_lc_status ON lifecycle_checklists(workspace_id, status)
    WHERE status = 'in_progress';

-- 4. Checklist Item Instances (actual tasks)
CREATE TABLE lifecycle_checklist_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    checklist_id UUID NOT NULL REFERENCES lifecycle_checklists(id) ON DELETE CASCADE,
    template_item_id UUID REFERENCES lifecycle_checklist_template_items(id),
    title TEXT NOT NULL,
    description TEXT,
    category VARCHAR(50) NOT NULL,
    assignee_user_id UUID,
    is_required BOOLEAN NOT NULL DEFAULT TRUE,
    due_date DATE,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',   -- 'pending', 'completed', 'skipped', 'overdue'
    completed_at TIMESTAMPTZ,
    completed_by UUID,
    notes TEXT,
    display_order INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_lci_checklist ON lifecycle_checklist_items(checklist_id);
CREATE INDEX idx_lci_assignee_status ON lifecycle_checklist_items(assignee_user_id, status)
    WHERE status = 'pending';

-- Up Migration: Add tags to employees and create assignment_rules table

-- 1. Add tags column to employees
ALTER TABLE employees ADD COLUMN IF NOT EXISTS tags VARCHAR(50)[] DEFAULT '{}';
CREATE INDEX IF NOT EXISTS idx_employees_tags ON employees USING GIN (tags);

-- 2. Create assignment_rules table
CREATE TABLE IF NOT EXISTS assignment_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    pattern_id UUID NOT NULL REFERENCES working_patterns(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    priority INTEGER NOT NULL DEFAULT 0,
    conditions JSONB NOT NULL, -- e.g., {"department_id": "...", "tags": ["NightShift"]}
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_assignment_rules_workspace_id ON assignment_rules(workspace_id);
CREATE INDEX IF NOT EXISTS idx_assignment_rules_priority ON assignment_rules(priority DESC);

COMMENT ON COLUMN assignment_rules.conditions IS 'Schema: { department_id?: UUID, employee_id_numbers?: string[], tags?: string[] }';
-- up.sql
CREATE TABLE employers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    tax_code VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_employers_workspace_id ON employers(workspace_id);

-- Link employees to employers
ALTER TABLE employees ADD COLUMN employer_id UUID REFERENCES employers(id) ON DELETE SET NULL;
CREATE INDEX idx_employees_employer_id ON employees(employer_id);

-- Expand EAV scope_type check constraint
ALTER TABLE workspace_config_entries DROP CONSTRAINT IF EXISTS workspace_config_entries_scope_type_check;
ALTER TABLE workspace_config_entries ADD CONSTRAINT workspace_config_entries_scope_type_check CHECK (scope_type IN ('workspace', 'employer', 'department', 'employee'));

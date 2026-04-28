CREATE TABLE salary_base_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID REFERENCES workspaces(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) NOT NULL,
    description TEXT,
    is_system BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Slug must be unique per workspace, OR unique if workspace_id is null (global)
CREATE UNIQUE INDEX idx_salary_base_groups_workspace_slug ON salary_base_groups (workspace_id, slug) WHERE workspace_id IS NOT NULL;
CREATE UNIQUE INDEX idx_salary_base_groups_global_slug ON salary_base_groups (slug) WHERE workspace_id IS NULL;

CREATE TABLE salary_component_base_members (
    component_definition_id UUID NOT NULL REFERENCES salary_component_definitions(id) ON DELETE CASCADE,
    base_group_id UUID NOT NULL REFERENCES salary_base_groups(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (component_definition_id, base_group_id)
);

-- Seed global bases
INSERT INTO salary_base_groups (id, workspace_id, name, slug, description, is_system) VALUES
    (gen_random_uuid(), NULL, 'Gross Salary', 'gross', 'Total standard salary + fixed allowances', true),
    (gen_random_uuid(), NULL, 'Taxable Income Base', 'taxable_income', 'Base amount subject to Personal Income Tax', true),
    (gen_random_uuid(), NULL, 'Overtime Base', 'ot_base', 'Base amount for overtime calculation', true),
    (gen_random_uuid(), NULL, 'Social Security Base', 'social_security_base', 'Base amount for mandatory social insurance/security calculation', true);

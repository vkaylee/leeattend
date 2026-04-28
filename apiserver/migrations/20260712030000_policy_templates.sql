-- Up Migration: Policy Templates (Future-Ready Skeleton)
-- Templates allow 1-click setup of country-specific configurations
-- e.g. "Vietnam Standard 2026" seeds all statutory deductions, tax brackets, probation rules

CREATE TABLE IF NOT EXISTS policy_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- NULL workspace_id = system-wide template (available to all workspaces)
    -- Non-NULL = custom template created by a workspace (private)
    workspace_id UUID REFERENCES workspaces(id) ON DELETE CASCADE,

    name TEXT NOT NULL,
    description TEXT,
    country_code VARCHAR(3), -- ISO 3166-1 alpha-2: 'VN', 'TH', 'SG', NULL for generic

    -- What this template covers
    category TEXT NOT NULL DEFAULT 'full',
    -- 'employment_phases' = only phase definitions
    -- 'payroll_rules' = only payroll rules + tiers
    -- 'full' = complete country pack (phases + rules + components)

    -- Full configuration snapshot
    -- Structure:
    -- {
    --   "employment_phases": [
    --     { "name": "Probation", "phase_type": "probation", "rule_config": {...} }
    --   ],
    --   "payroll_rules": [
    --     { "name": "BHXH (Employee)", "code": "BHXH_EE", "category": "statutory_deduction",
    --       "calculation_method": "percentage", "rule_config": { "rate": 8.0, ... },
    --       "tiers": null }
    --   ],
    --   "salary_components": [
    --     { "name": "Social Insurance", "code": "SI_EE", "component_type": "statutory", ... }
    --   ]
    -- }
    template_data JSONB NOT NULL DEFAULT '{}'::jsonb,

    version INTEGER NOT NULL DEFAULT 1,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- System templates are globally unique by name+version
CREATE UNIQUE INDEX idx_policy_templates_system_unique
    ON policy_templates(name, version) WHERE workspace_id IS NULL;

-- Workspace templates are unique by name within workspace
CREATE UNIQUE INDEX idx_policy_templates_workspace_unique
    ON policy_templates(workspace_id, name) WHERE workspace_id IS NOT NULL;

CREATE INDEX idx_policy_templates_country
    ON policy_templates(country_code) WHERE country_code IS NOT NULL;
CREATE INDEX idx_policy_templates_category
    ON policy_templates(category);

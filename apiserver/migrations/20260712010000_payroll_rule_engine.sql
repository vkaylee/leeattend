-- Up Migration: Generic Payroll Rule Engine
-- Replaces hardcoded statutory deductions with configurable rules
-- Pattern: Same as overtime_policy_definitions → overtime_rules → overtime_policy_assignments

-- Ensure btree_gist extension (should already exist from leave_policies migration)
CREATE EXTENSION IF NOT EXISTS btree_gist;

-- 1. Payroll Rule Definitions (workspace-scoped)
CREATE TABLE IF NOT EXISTS payroll_rule_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    code TEXT NOT NULL,
    -- e.g. 'BHXH_ER', 'BHXH_EE', 'PIT', 'UNION_FEE'

    -- Rule classification
    category TEXT NOT NULL DEFAULT 'deduction',
    -- 'statutory_deduction', 'tax', 'employer_contribution', 'benefit', 'deduction', 'custom'

    applies_to TEXT NOT NULL DEFAULT 'employee',
    -- 'employee' = deducted from employee pay
    -- 'employer' = employer cost (not deducted from employee)
    -- 'both' = separate rates for each (tracked in rule_config)

    calculation_method TEXT NOT NULL DEFAULT 'percentage',
    -- 'percentage' = percentage of a base amount
    -- 'fixed' = fixed amount per period
    -- 'tiered' = progressive brackets (uses payroll_rule_tiers)
    -- 'formula' = custom formula expression (future)

    -- Flexible configuration per calculation_method
    -- percentage: { "rate": 8.0, "base": "gross_salary", "cap_amount": 36000000, "cap_type": "monthly" }
    -- fixed: { "amount": 500000, "frequency": "monthly" }
    -- tiered: { "base": "taxable_income" } (brackets in payroll_rule_tiers)
    -- formula: { "expression": "base_salary * 0.08 + allowance * 0.05" }
    rule_config JSONB NOT NULL DEFAULT '{}'::jsonb,

    is_mandatory BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    display_order INTEGER NOT NULL DEFAULT 0,

    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(workspace_id, code)
);

CREATE INDEX idx_payroll_rule_defs_workspace
    ON payroll_rule_definitions(workspace_id);
CREATE INDEX idx_payroll_rule_defs_category
    ON payroll_rule_definitions(workspace_id, category);

-- 2. Payroll Rule Tiers (for progressive/tiered calculations)
-- Example: Vietnam PIT brackets
--   tier 1: 0 - 5,000,000 → 5%
--   tier 2: 5,000,001 - 10,000,000 → 10%
--   tier 3: 10,000,001 - 18,000,000 → 15%
--   ...
CREATE TABLE IF NOT EXISTS payroll_rule_tiers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_definition_id UUID NOT NULL REFERENCES payroll_rule_definitions(id) ON DELETE CASCADE,
    tier_order INTEGER NOT NULL,
    min_amount DECIMAL(15, 2) NOT NULL,
    max_amount DECIMAL(15, 2),  -- NULL = unlimited (top bracket)
    rate DECIMAL(8, 4) NOT NULL, -- percentage rate for this tier
    fixed_amount DECIMAL(15, 2) NOT NULL DEFAULT 0, -- fixed component in this tier
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(rule_definition_id, tier_order),
    CHECK (max_amount IS NULL OR max_amount > min_amount)
);

CREATE INDEX idx_payroll_rule_tiers_definition
    ON payroll_rule_tiers(rule_definition_id);

-- 3. Payroll Rule Assignments (temporal, 3-tier scope)
-- Same EXCLUDE USING GIST pattern as leave/overtime policies
CREATE TABLE IF NOT EXISTS payroll_rule_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    rule_definition_id UUID NOT NULL REFERENCES payroll_rule_definitions(id) ON DELETE RESTRICT,

    -- Scope of assignment (same pattern as leave_policy_assignments)
    assignee_type TEXT NOT NULL CHECK (assignee_type IN ('workspace', 'department', 'employee')),
    assignee_id UUID, -- NULL when assignee_type = 'workspace'

    -- Temporal validity
    effective_from DATE NOT NULL,
    effective_to DATE, -- NULL = indefinitely

    -- Override rule_config at assignment level
    -- e.g. a specific department gets a different rate
    override_config JSONB,

    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Prevent overlapping assignments for the same rule + scope
    EXCLUDE USING GIST (
        workspace_id WITH =,
        rule_definition_id WITH =,
        assignee_type WITH =,
        COALESCE(assignee_id, '00000000-0000-0000-0000-000000000000'::uuid) WITH =,
        daterange(effective_from, effective_to, '[)') WITH &&
    )
);

CREATE INDEX idx_payroll_rule_assignments_workspace
    ON payroll_rule_assignments(workspace_id);
CREATE INDEX idx_payroll_rule_assignments_rule
    ON payroll_rule_assignments(rule_definition_id);
CREATE INDEX idx_payroll_rule_assignments_scope
    ON payroll_rule_assignments(workspace_id, assignee_type, assignee_id);
CREATE INDEX idx_payroll_rule_assignments_temporal
    ON payroll_rule_assignments(workspace_id, effective_from, effective_to);

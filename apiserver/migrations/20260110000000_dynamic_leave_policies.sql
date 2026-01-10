-- Up Migration: Dynamic Leave Policies

-- Ensure btree_gist extension is enabled for UUID exclusion constraints
CREATE EXTENSION IF NOT EXISTS btree_gist;

-- 1. Leave Policy Definitions
CREATE TABLE leave_policy_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    -- Configuration for the policy: frequency, amount, seniority rules, etc.
    -- Example: { "frequency": "monthly", "base_amount": 1.0, "seniority_bonus": [...] }
    rule_config JSONB NOT NULL DEFAULT '{}'::jsonb, 
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 2. Leave Policy Assignments (Temporal)
CREATE TABLE leave_policy_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    
    -- Scope of assignment
    assignee_type TEXT NOT NULL CHECK (assignee_type IN ('workspace', 'department', 'employee')),
    assignee_id UUID, -- Can be NULL if assignee_type is 'workspace' (meaning global default)
    
    policy_id UUID NOT NULL REFERENCES leave_policy_definitions(id) ON DELETE RESTRICT,
    
    -- Validity Period
    effective_from DATE NOT NULL,
    effective_to DATE, -- NULL means indefinitely
    
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    
    -- Constraint to prevent overlapping assignments for the same scope
    -- Note: We rely on application logic for complex hierarchy overlaps (e.g. employee vs dept),
    -- but we enforce uniqueness for the exact same assignee_id + type range.
    EXCLUDE USING GIST (
        workspace_id WITH =, 
        assignee_type WITH =, 
        COALESCE(assignee_id, '00000000-0000-0000-0000-000000000000'::uuid) WITH =, 
        daterange(effective_from, effective_to, '[)') WITH &&
    )
);

-- 3. Leave Accrual Logs (Audit Trail)
CREATE TABLE leave_accrual_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    policy_assignment_id UUID REFERENCES leave_policy_assignments(id) ON DELETE SET NULL,
    
    accrual_date DATE NOT NULL, -- The date this accrual is credited for (e.g., 2026-02-01)
    amount NUMERIC(5, 2) NOT NULL, -- Amount added (e.g., 1.0, 1.25)
    
    leave_type VARCHAR(50) NOT NULL, -- 'vacation', 'sick', etc. matched from policy
    
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(employee_id, leave_type, accrual_date) -- Prevent double accrual for same day/type
);

-- 4. Update leave_balances to track accrued amount separately
ALTER TABLE leave_balances 
ADD COLUMN accrued_amount NUMERIC(5, 2) NOT NULL DEFAULT 0.00;

-- Down Migration
-- DROP TABLE leave_accrual_logs;
-- DROP TABLE leave_policy_assignments;
-- DROP TABLE leave_policy_definitions;
-- ALTER TABLE leave_balances DROP COLUMN accrued_amount;

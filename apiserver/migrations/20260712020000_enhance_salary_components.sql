-- Up Migration: Enhance salary_component_definitions with component classification
-- Needed for payroll rule engine integration (which components are subject to which rules)

-- Add component_type to classify salary components
ALTER TABLE salary_component_definitions
    ADD COLUMN IF NOT EXISTS component_type TEXT NOT NULL DEFAULT 'allowance';
    -- 'base' = base salary component
    -- 'allowance' = regular allowance (meal, phone, transport)
    -- 'bonus' = variable bonus
    -- 'deduction' = manual deduction
    -- 'statutory' = statutory item (linked to payroll_rule_definitions)

-- Whether this component is included in social insurance calculation base
ALTER TABLE salary_component_definitions
    ADD COLUMN IF NOT EXISTS is_insurance_base BOOLEAN NOT NULL DEFAULT FALSE;

-- Link to payroll_rule_definitions for automatic calculation
-- NULL = manually set amount, NOT NULL = auto-calculated by rule engine
ALTER TABLE salary_component_definitions
    ADD COLUMN IF NOT EXISTS linked_rule_id UUID REFERENCES payroll_rule_definitions(id) ON DELETE SET NULL;

-- Migrate existing data: mark 'BASE' components
UPDATE salary_component_definitions
SET component_type = 'base', is_insurance_base = TRUE
WHERE code = 'BASE';

-- Index for rule linkage queries
CREATE INDEX IF NOT EXISTS idx_salary_comp_defs_linked_rule
    ON salary_component_definitions(linked_rule_id)
    WHERE linked_rule_id IS NOT NULL;

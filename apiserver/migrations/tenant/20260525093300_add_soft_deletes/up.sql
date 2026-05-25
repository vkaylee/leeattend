-- Add deleted_at to departments
ALTER TABLE departments ADD COLUMN deleted_at TIMESTAMPTZ DEFAULT NULL;

-- Add deleted_at to leave_types
ALTER TABLE leave_types ADD COLUMN deleted_at TIMESTAMPTZ DEFAULT NULL;

-- Add deleted_at to leave_policy_definitions
ALTER TABLE leave_policy_definitions ADD COLUMN deleted_at TIMESTAMPTZ DEFAULT NULL;

-- Add deleted_at to working_patterns
ALTER TABLE working_patterns ADD COLUMN deleted_at TIMESTAMPTZ DEFAULT NULL;

-- Add deleted_at to holidays
ALTER TABLE holidays ADD COLUMN deleted_at TIMESTAMPTZ DEFAULT NULL;

-- Drop and recreate unique constraints to allow same names after soft delete
-- DEPARTMENTS
ALTER TABLE departments DROP CONSTRAINT IF EXISTS uq_departments_name;
CREATE UNIQUE INDEX uq_departments_name ON departments(workspace_id, LOWER(name)) WHERE deleted_at IS NULL;

-- LEAVE TYPES
ALTER TABLE leave_types DROP CONSTRAINT IF EXISTS uq_leave_types_code;
CREATE UNIQUE INDEX uq_leave_types_code ON leave_types(workspace_id, LOWER(code)) WHERE deleted_at IS NULL;

-- LEAVE POLICY DEFINITIONS
ALTER TABLE leave_policy_definitions DROP CONSTRAINT IF EXISTS uq_leave_policy_definitions_name;
CREATE UNIQUE INDEX uq_leave_policy_definitions_name ON leave_policy_definitions(workspace_id, LOWER(name)) WHERE deleted_at IS NULL;

-- WORKING PATTERNS
ALTER TABLE working_patterns DROP CONSTRAINT IF EXISTS uq_working_patterns_name;
CREATE UNIQUE INDEX uq_working_patterns_name ON working_patterns(workspace_id, LOWER(name)) WHERE deleted_at IS NULL;

-- HOLIDAYS (usually unique per date + name)
ALTER TABLE holidays DROP CONSTRAINT IF EXISTS uq_holidays_workspace_date;
CREATE UNIQUE INDEX uq_holidays_workspace_date ON holidays(workspace_id, date, name) WHERE deleted_at IS NULL;

-- Revert DEPARTMENTS
DROP INDEX IF EXISTS uq_departments_name;
ALTER TABLE departments ADD CONSTRAINT uq_departments_name UNIQUE (workspace_id, name);
ALTER TABLE departments DROP COLUMN deleted_at;

-- Revert LEAVE TYPES
DROP INDEX IF EXISTS uq_leave_types_code;
ALTER TABLE leave_types ADD CONSTRAINT uq_leave_types_code UNIQUE (workspace_id, code);
ALTER TABLE leave_types DROP COLUMN deleted_at;

-- Revert LEAVE POLICY DEFINITIONS
DROP INDEX IF EXISTS uq_leave_policy_definitions_name;
ALTER TABLE leave_policy_definitions ADD CONSTRAINT uq_leave_policy_definitions_name UNIQUE (workspace_id, name);
ALTER TABLE leave_policy_definitions DROP COLUMN deleted_at;

-- Revert WORKING PATTERNS
DROP INDEX IF EXISTS uq_working_patterns_name;
ALTER TABLE working_patterns ADD CONSTRAINT uq_working_patterns_name UNIQUE (workspace_id, name);
ALTER TABLE working_patterns DROP COLUMN deleted_at;

-- Revert HOLIDAYS
DROP INDEX IF EXISTS uq_holidays_workspace_date;
ALTER TABLE holidays ADD CONSTRAINT uq_holidays_workspace_date UNIQUE (workspace_id, date, name);
ALTER TABLE holidays DROP COLUMN deleted_at;

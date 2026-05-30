-- down.sql
ALTER TABLE workspace_config_entries DROP CONSTRAINT IF EXISTS workspace_config_entries_scope_type_check;
ALTER TABLE workspace_config_entries ADD CONSTRAINT workspace_config_entries_scope_type_check CHECK (scope_type IN ('workspace', 'department', 'employee'));

ALTER TABLE employees DROP COLUMN IF EXISTS employer_id;
DROP TABLE IF EXISTS employers;

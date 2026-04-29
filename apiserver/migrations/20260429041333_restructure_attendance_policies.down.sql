ALTER TABLE workspaces ADD COLUMN work_unit_minutes INT NOT NULL DEFAULT 1;
ALTER TABLE workspaces ADD COLUMN rounding_policy VARCHAR(20) NOT NULL DEFAULT 'none';

DROP TABLE IF EXISTS attendance_policy_assignments CASCADE;
DROP TABLE IF EXISTS attendance_policy_versions CASCADE;
DROP TABLE IF EXISTS attendance_policies CASCADE;

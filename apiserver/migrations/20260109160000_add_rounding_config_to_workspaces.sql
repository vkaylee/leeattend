-- Up
ALTER TABLE workspaces ADD COLUMN work_unit_minutes INT NOT NULL DEFAULT 1;
ALTER TABLE workspaces ADD COLUMN rounding_policy VARCHAR(20) NOT NULL DEFAULT 'none';

-- Comment: rounding_policy can be 'none', 'floor', 'ceil', 'round'

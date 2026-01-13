-- Add time_format to workspaces
ALTER TABLE workspaces ADD COLUMN time_format VARCHAR(10) NOT NULL DEFAULT '24h';

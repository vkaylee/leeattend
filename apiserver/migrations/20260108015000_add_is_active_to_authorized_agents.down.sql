-- Down migration: Remove is_active from authorized_agents table
ALTER TABLE authorized_agents DROP COLUMN is_active;
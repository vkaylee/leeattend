-- Down Migration: Remove workspaces, users, and workspace_members

ALTER TABLE devices DROP COLUMN workspace_id;
ALTER TABLE authorized_agents DROP COLUMN workspace_id;

DROP TABLE IF EXISTS workspace_members;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS workspaces;

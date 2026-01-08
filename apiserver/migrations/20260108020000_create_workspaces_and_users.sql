-- Up Migration: Create workspaces, users, and workspace_members tables

-- Enable pgcrypto for gen_random_uuid() if not available (Postgres 13+ has it built-in)
-- CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Create workspaces table
CREATE TABLE IF NOT EXISTS workspaces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Create workspace_members table
CREATE TABLE IF NOT EXISTS workspace_members (
    workspace_id UUID REFERENCES workspaces(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL, -- owner, admin, member
    PRIMARY KEY (workspace_id, user_id)
);

-- Add workspace_id to authorized_agents
ALTER TABLE authorized_agents ADD COLUMN workspace_id UUID REFERENCES workspaces(id) ON DELETE CASCADE;
CREATE INDEX IF NOT EXISTS idx_authorized_agents_workspace ON authorized_agents(workspace_id);

-- Add workspace_id to devices
ALTER TABLE devices ADD COLUMN workspace_id UUID REFERENCES workspaces(id) ON DELETE CASCADE;
CREATE INDEX IF NOT EXISTS idx_devices_workspace ON devices(workspace_id);

-- Create a default workspace for existing data
DO $$
DECLARE
    default_ws_id UUID;
BEGIN
    INSERT INTO workspaces (name) VALUES ('Default Workspace') RETURNING id INTO default_ws_id;

    -- Assign existing agents and devices to the default workspace
    UPDATE authorized_agents SET workspace_id = default_ws_id WHERE workspace_id IS NULL;
    UPDATE devices SET workspace_id = default_ws_id WHERE workspace_id IS NULL;
END $$;

-- Make workspace_id NOT NULL after migration
ALTER TABLE authorized_agents ALTER COLUMN workspace_id SET NOT NULL;
ALTER TABLE devices ALTER COLUMN workspace_id SET NOT NULL;

-- Seed data for development environment

-- Create a default admin user with fixed UUID for easier development
INSERT INTO users (id, email, full_name, password_hash)
VALUES ('00000000-0000-0000-0000-000000000001', 'admin@leeattend.com', 'System Administrator', 'hashed_password_here')
ON CONFLICT (id) DO NOTHING;

-- Create a default workspace for the admin
INSERT INTO workspaces (id, name)
VALUES ('00000000-0000-0000-0000-000000000002', 'Default Workspace')
ON CONFLICT (id) DO NOTHING;

-- Link them
INSERT INTO workspace_members (workspace_id, user_id, role)
VALUES ('00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000001', 'owner')
ON CONFLICT DO NOTHING;

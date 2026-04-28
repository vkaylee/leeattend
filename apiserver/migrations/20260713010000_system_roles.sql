-- System-level RBAC: platform-wide roles independent of any workspace.
-- These govern who can create workspaces, view infrastructure, manage billing, etc.
-- Separate from workspace_roles which are tenant-scoped.

CREATE TABLE system_roles (
    id   UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT        NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed the three built-in system roles.
INSERT INTO system_roles (name, description) VALUES
    ('platform_admin', 'Full platform control: create/suspend workspaces, assign system roles, manage infrastructure'),
    ('support',        'Read-only cross-workspace access for debugging customer issues'),
    ('billing_admin',  'Manage billing, subscription plans, and invoicing');

-- Junction table: which users hold which system roles.
CREATE TABLE user_system_roles (
    user_id     UUID        NOT NULL REFERENCES users(id)        ON DELETE CASCADE,
    role_id     UUID        NOT NULL REFERENCES system_roles(id) ON DELETE CASCADE,
    assigned_by UUID        REFERENCES users(id) ON DELETE SET NULL,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, role_id)
);

-- Index for fast lookup of "what system roles does user X have?"
CREATE INDEX idx_user_system_roles_user_id ON user_system_roles(user_id);
-- Index for "who has system role Y?" (admin management UI)
CREATE INDEX idx_user_system_roles_role_id ON user_system_roles(role_id);

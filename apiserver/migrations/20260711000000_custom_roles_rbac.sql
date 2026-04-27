-- Up Migration: Custom Roles and RBAC

CREATE TABLE IF NOT EXISTS workspace_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    color TEXT DEFAULT 'slate',
    is_system BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_workspace_roles_unique_name ON workspace_roles(workspace_id, name) WHERE workspace_id IS NOT NULL;
CREATE UNIQUE INDEX idx_system_roles_unique_name ON workspace_roles(name) WHERE workspace_id IS NULL;

CREATE TABLE IF NOT EXISTS workspace_role_permissions (
    role_id UUID NOT NULL REFERENCES workspace_roles(id) ON DELETE CASCADE,
    permission_name TEXT NOT NULL,
    PRIMARY KEY (role_id, permission_name)
);

ALTER TABLE workspace_members ADD COLUMN role_id UUID REFERENCES workspace_roles(id);

-- Seed System Roles with deterministic UUIDs
INSERT INTO workspace_roles (id, workspace_id, name, description, color, is_system)
VALUES 
  ('a0000000-0000-0000-0000-000000000001', NULL, 'owner', 'Workspace owner with full access', 'rose', TRUE),
  ('a0000000-0000-0000-0000-000000000002', NULL, 'admin', 'Administrator with full access', 'rose', TRUE),
  ('a0000000-0000-0000-0000-000000000003', NULL, 'hr_manager', 'Manage people and time', 'indigo', TRUE),
  ('a0000000-0000-0000-0000-000000000004', NULL, 'shift_manager', 'Manage shifts', 'amber', TRUE),
  ('a0000000-0000-0000-0000-000000000005', NULL, 'team_lead', 'Manage team members', 'blue', TRUE),
  ('a0000000-0000-0000-0000-000000000006', NULL, 'employee', 'Default employee access', 'slate', TRUE)
ON CONFLICT DO NOTHING;

-- Seed Permissions for System Roles
-- Owner has all core permissions
INSERT INTO workspace_role_permissions (role_id, permission_name)
VALUES
  ('a0000000-0000-0000-0000-000000000001', 'workspace:update'),
  ('a0000000-0000-0000-0000-000000000001', 'workspace:delete'),
  ('a0000000-0000-0000-0000-000000000001', 'workspace:settings:read'),
  ('a0000000-0000-0000-0000-000000000001', 'workspace:settings:write'),
  ('a0000000-0000-0000-0000-000000000001', 'members:read'),
  ('a0000000-0000-0000-0000-000000000001', 'members:invite'),
  ('a0000000-0000-0000-0000-000000000001', 'members:roles:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'members:remove'),
  ('a0000000-0000-0000-0000-000000000001', 'employees:read'),
  ('a0000000-0000-0000-0000-000000000001', 'employees:create'),
  ('a0000000-0000-0000-0000-000000000001', 'employees:update'),
  ('a0000000-0000-0000-0000-000000000001', 'employees:delete'),
  ('a0000000-0000-0000-0000-000000000001', 'employees:rehire'),
  ('a0000000-0000-0000-0000-000000000001', 'employees:upload'),
  ('a0000000-0000-0000-0000-000000000001', 'departments:read'),
  ('a0000000-0000-0000-0000-000000000001', 'departments:create'),
  ('a0000000-0000-0000-0000-000000000001', 'departments:update'),
  ('a0000000-0000-0000-0000-000000000001', 'departments:delete'),
  ('a0000000-0000-0000-0000-000000000001', 'departments:transfer'),
  ('a0000000-0000-0000-0000-000000000001', 'attendance:logs:read'),
  ('a0000000-0000-0000-0000-000000000001', 'attendance:logs:import'),
  ('a0000000-0000-0000-0000-000000000001', 'attendance:summaries:read'),
  ('a0000000-0000-0000-0000-000000000001', 'attendance:adjustments:submit'),
  ('a0000000-0000-0000-0000-000000000001', 'attendance:adjustments:approve'),
  ('a0000000-0000-0000-0000-000000000001', 'attendance:reprocess'),
  ('a0000000-0000-0000-0000-000000000001', 'attendance:staging:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'overtime:claim'),
  ('a0000000-0000-0000-0000-000000000001', 'overtime:approve'),
  ('a0000000-0000-0000-0000-000000000001', 'overtime:policies:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'overtime:paper:upload'),
  ('a0000000-0000-0000-0000-000000000001', 'leave:requests:read'),
  ('a0000000-0000-0000-0000-000000000001', 'leave:requests:submit'),
  ('a0000000-0000-0000-0000-000000000001', 'leave:requests:approve'),
  ('a0000000-0000-0000-0000-000000000001', 'leave:balances:read'),
  ('a0000000-0000-0000-0000-000000000001', 'leave:balances:adjust'),
  ('a0000000-0000-0000-0000-000000000001', 'leave:types:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'leave:policies:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'leave:accrual:run'),
  ('a0000000-0000-0000-0000-000000000001', 'shifts:read'),
  ('a0000000-0000-0000-0000-000000000001', 'shifts:create'),
  ('a0000000-0000-0000-0000-000000000001', 'shifts:update'),
  ('a0000000-0000-0000-0000-000000000001', 'shifts:assign'),
  ('a0000000-0000-0000-0000-000000000001', 'shift_swaps:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'shift_swaps:approve'),
  ('a0000000-0000-0000-0000-000000000001', 'patterns:read'),
  ('a0000000-0000-0000-0000-000000000001', 'patterns:create'),
  ('a0000000-0000-0000-0000-000000000001', 'patterns:assign'),
  ('a0000000-0000-0000-0000-000000000001', 'patterns:delete'),
  ('a0000000-0000-0000-0000-000000000001', 'rosters:read'),
  ('a0000000-0000-0000-0000-000000000001', 'rosters:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'assignment_rules:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'payroll:read'),
  ('a0000000-0000-0000-0000-000000000001', 'payroll:calculate'),
  ('a0000000-0000-0000-0000-000000000001', 'payroll:settings:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'payroll:components:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'payroll:incentives:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'holidays:read'),
  ('a0000000-0000-0000-0000-000000000001', 'holidays:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'reports:read'),
  ('a0000000-0000-0000-0000-000000000001', 'reports:export'),
  ('a0000000-0000-0000-0000-000000000001', 'dashboard:read'),
  ('a0000000-0000-0000-0000-000000000001', 'dashboard:preferences'),
  ('a0000000-0000-0000-0000-000000000001', 'agents:read'),
  ('a0000000-0000-0000-0000-000000000001', 'agents:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'devices:read'),
  ('a0000000-0000-0000-0000-000000000001', 'devices:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'recruitment:read'),
  ('a0000000-0000-0000-0000-000000000001', 'recruitment:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'feature_flags:read'),
  ('a0000000-0000-0000-0000-000000000001', 'feature_flags:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'infra:monitor'),
  ('a0000000-0000-0000-0000-000000000001', 'ess:profile'),
  ('a0000000-0000-0000-0000-000000000001', 'ess:attendance'),
  ('a0000000-0000-0000-0000-000000000001', 'ess:clock'),
  ('a0000000-0000-0000-0000-000000000001', 'ess:leave'),
  ('a0000000-0000-0000-0000-000000000001', 'ess:payslips'),
  ('a0000000-0000-0000-0000-000000000001', 'ess:shift_swaps')
ON CONFLICT DO NOTHING;

-- Admin has similar permissions to Owner
INSERT INTO workspace_role_permissions (role_id, permission_name)
VALUES
  ('a0000000-0000-0000-0000-000000000002', 'workspace:settings:write'),
  ('a0000000-0000-0000-0000-000000000002', 'workspace:settings:read'),
  ('a0000000-0000-0000-0000-000000000002', 'members:invite'),
  ('a0000000-0000-0000-0000-000000000002', 'members:roles:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'members:remove'),
  ('a0000000-0000-0000-0000-000000000002', 'members:read'),
  ('a0000000-0000-0000-0000-000000000002', 'employees:read'),
  ('a0000000-0000-0000-0000-000000000002', 'employees:create'),
  ('a0000000-0000-0000-0000-000000000002', 'employees:update'),
  ('a0000000-0000-0000-0000-000000000002', 'employees:delete'),
  ('a0000000-0000-0000-0000-000000000002', 'employees:rehire'),
  ('a0000000-0000-0000-0000-000000000002', 'employees:upload'),
  ('a0000000-0000-0000-0000-000000000002', 'departments:read'),
  ('a0000000-0000-0000-0000-000000000002', 'departments:create'),
  ('a0000000-0000-0000-0000-000000000002', 'departments:update'),
  ('a0000000-0000-0000-0000-000000000002', 'departments:delete'),
  ('a0000000-0000-0000-0000-000000000002', 'departments:transfer'),
  ('a0000000-0000-0000-0000-000000000002', 'attendance:logs:read'),
  ('a0000000-0000-0000-0000-000000000002', 'attendance:logs:import'),
  ('a0000000-0000-0000-0000-000000000002', 'attendance:summaries:read'),
  ('a0000000-0000-0000-0000-000000000002', 'attendance:adjustments:submit'),
  ('a0000000-0000-0000-0000-000000000002', 'attendance:adjustments:approve'),
  ('a0000000-0000-0000-0000-000000000002', 'attendance:reprocess'),
  ('a0000000-0000-0000-0000-000000000002', 'attendance:staging:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'overtime:claim'),
  ('a0000000-0000-0000-0000-000000000002', 'overtime:approve'),
  ('a0000000-0000-0000-0000-000000000002', 'overtime:policies:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'overtime:paper:upload'),
  ('a0000000-0000-0000-0000-000000000002', 'leave:requests:read'),
  ('a0000000-0000-0000-0000-000000000002', 'leave:requests:submit'),
  ('a0000000-0000-0000-0000-000000000002', 'leave:requests:approve'),
  ('a0000000-0000-0000-0000-000000000002', 'leave:balances:read'),
  ('a0000000-0000-0000-0000-000000000002', 'leave:balances:adjust'),
  ('a0000000-0000-0000-0000-000000000002', 'leave:types:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'leave:policies:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'leave:accrual:run'),
  ('a0000000-0000-0000-0000-000000000002', 'shifts:read'),
  ('a0000000-0000-0000-0000-000000000002', 'shifts:create'),
  ('a0000000-0000-0000-0000-000000000002', 'shifts:update'),
  ('a0000000-0000-0000-0000-000000000002', 'shifts:assign'),
  ('a0000000-0000-0000-0000-000000000002', 'shift_swaps:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'shift_swaps:approve'),
  ('a0000000-0000-0000-0000-000000000002', 'patterns:read'),
  ('a0000000-0000-0000-0000-000000000002', 'patterns:create'),
  ('a0000000-0000-0000-0000-000000000002', 'patterns:assign'),
  ('a0000000-0000-0000-0000-000000000002', 'patterns:delete'),
  ('a0000000-0000-0000-0000-000000000002', 'rosters:read'),
  ('a0000000-0000-0000-0000-000000000002', 'rosters:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'assignment_rules:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'payroll:read'),
  ('a0000000-0000-0000-0000-000000000002', 'payroll:calculate'),
  ('a0000000-0000-0000-0000-000000000002', 'payroll:settings:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'payroll:components:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'payroll:incentives:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'holidays:read'),
  ('a0000000-0000-0000-0000-000000000002', 'holidays:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'reports:read'),
  ('a0000000-0000-0000-0000-000000000002', 'reports:export'),
  ('a0000000-0000-0000-0000-000000000002', 'dashboard:read'),
  ('a0000000-0000-0000-0000-000000000002', 'dashboard:preferences'),
  ('a0000000-0000-0000-0000-000000000002', 'agents:read'),
  ('a0000000-0000-0000-0000-000000000002', 'agents:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'devices:read'),
  ('a0000000-0000-0000-0000-000000000002', 'devices:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'recruitment:read'),
  ('a0000000-0000-0000-0000-000000000002', 'recruitment:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'feature_flags:read'),
  ('a0000000-0000-0000-0000-000000000002', 'feature_flags:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'infra:monitor'),
  ('a0000000-0000-0000-0000-000000000002', 'ess:profile'),
  ('a0000000-0000-0000-0000-000000000002', 'ess:attendance'),
  ('a0000000-0000-0000-0000-000000000002', 'ess:clock'),
  ('a0000000-0000-0000-0000-000000000002', 'ess:leave'),
  ('a0000000-0000-0000-0000-000000000002', 'ess:payslips'),
  ('a0000000-0000-0000-0000-000000000002', 'ess:shift_swaps')
ON CONFLICT DO NOTHING;

-- HrManager permissions
INSERT INTO workspace_role_permissions (role_id, permission_name)
VALUES
  ('a0000000-0000-0000-0000-000000000003', 'employees:read'),
  ('a0000000-0000-0000-0000-000000000003', 'employees:create'),
  ('a0000000-0000-0000-0000-000000000003', 'employees:update'),
  ('a0000000-0000-0000-0000-000000000003', 'attendance:logs:read'),
  ('a0000000-0000-0000-0000-000000000003', 'attendance:summaries:read'),
  ('a0000000-0000-0000-0000-000000000003', 'attendance:adjustments:approve'),
  ('a0000000-0000-0000-0000-000000000003', 'leave:requests:read'),
  ('a0000000-0000-0000-0000-000000000003', 'leave:requests:approve'),
  ('a0000000-0000-0000-0000-000000000003', 'payroll:read'),
  ('a0000000-0000-0000-0000-000000000003', 'departments:read'),
  ('a0000000-0000-0000-0000-000000000003', 'holidays:read'),
  ('a0000000-0000-0000-0000-000000000003', 'leave:requests:submit')
ON CONFLICT DO NOTHING;

-- ShiftManager permissions
INSERT INTO workspace_role_permissions (role_id, permission_name)
VALUES
  ('a0000000-0000-0000-0000-000000000004', 'employees:read'),
  ('a0000000-0000-0000-0000-000000000004', 'shifts:read'),
  ('a0000000-0000-0000-0000-000000000004', 'shifts:create'),
  ('a0000000-0000-0000-0000-000000000004', 'shifts:update'),
  ('a0000000-0000-0000-0000-000000000004', 'shifts:assign'),
  ('a0000000-0000-0000-0000-000000000004', 'rosters:read'),
  ('a0000000-0000-0000-0000-000000000004', 'rosters:manage'),
  ('a0000000-0000-0000-0000-000000000004', 'attendance:logs:read')
ON CONFLICT DO NOTHING;

-- TeamLead permissions
INSERT INTO workspace_role_permissions (role_id, permission_name)
VALUES
  ('a0000000-0000-0000-0000-000000000005', 'employees:read'),
  ('a0000000-0000-0000-0000-000000000005', 'attendance:summaries:read'),
  ('a0000000-0000-0000-0000-000000000005', 'leave:requests:read'),
  ('a0000000-0000-0000-0000-000000000005', 'leave:requests:submit'),
  ('a0000000-0000-0000-0000-000000000005', 'leave:requests:approve')
ON CONFLICT DO NOTHING;

-- Employee default permissions
INSERT INTO workspace_role_permissions (role_id, permission_name)
VALUES
  ('a0000000-0000-0000-0000-000000000006', 'ess:profile'),
  ('a0000000-0000-0000-0000-000000000006', 'ess:attendance'),
  ('a0000000-0000-0000-0000-000000000006', 'ess:clock'),
  ('a0000000-0000-0000-0000-000000000006', 'ess:leave'),
  ('a0000000-0000-0000-0000-000000000006', 'ess:payslips'),
  ('a0000000-0000-0000-0000-000000000006', 'ess:shift_swaps')
ON CONFLICT DO NOTHING;


-- Map existing enum roles to role_id
UPDATE workspace_members SET role_id = 'a0000000-0000-0000-0000-000000000001' WHERE role::text = 'owner';
UPDATE workspace_members SET role_id = 'a0000000-0000-0000-0000-000000000002' WHERE role::text = 'admin';
UPDATE workspace_members SET role_id = 'a0000000-0000-0000-0000-000000000003' WHERE role::text = 'hr_manager';
UPDATE workspace_members SET role_id = 'a0000000-0000-0000-0000-000000000004' WHERE role::text = 'shift_manager';
UPDATE workspace_members SET role_id = 'a0000000-0000-0000-0000-000000000005' WHERE role::text = 'team_lead';
UPDATE workspace_members SET role_id = 'a0000000-0000-0000-0000-000000000006' WHERE role::text = 'employee';

-- Make role_id NOT NULL and Drop the old enum column
ALTER TABLE workspace_members ALTER COLUMN role_id SET NOT NULL;
ALTER TABLE workspace_members DROP COLUMN role;

-- We can keep member_role_type enum in DB, just not used by workspace_members anymore.

-- Migrate member_extra_roles
ALTER TABLE member_extra_roles ADD COLUMN role_id UUID REFERENCES workspace_roles(id) ON DELETE CASCADE;

UPDATE member_extra_roles SET role_id = 'a0000000-0000-0000-0000-000000000001' WHERE role::text = 'owner';
UPDATE member_extra_roles SET role_id = 'a0000000-0000-0000-0000-000000000002' WHERE role::text = 'admin';
UPDATE member_extra_roles SET role_id = 'a0000000-0000-0000-0000-000000000003' WHERE role::text = 'hr_manager';
UPDATE member_extra_roles SET role_id = 'a0000000-0000-0000-0000-000000000004' WHERE role::text = 'shift_manager';
UPDATE member_extra_roles SET role_id = 'a0000000-0000-0000-0000-000000000005' WHERE role::text = 'team_lead';
UPDATE member_extra_roles SET role_id = 'a0000000-0000-0000-0000-000000000006' WHERE role::text = 'employee';

-- Delete orphaned roles
DELETE FROM member_extra_roles WHERE role_id IS NULL;

ALTER TABLE member_extra_roles ALTER COLUMN role_id SET NOT NULL;
ALTER TABLE member_extra_roles DROP COLUMN role;

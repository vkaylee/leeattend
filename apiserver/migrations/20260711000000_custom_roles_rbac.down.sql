-- Down Migration: Revert Custom Roles and RBAC

-- Re-add the role column
ALTER TABLE workspace_members ADD COLUMN role member_role_type;

-- Map role_id back to enum
UPDATE workspace_members SET role = 'owner' WHERE role_id = 'a0000000-0000-0000-0000-000000000001';
UPDATE workspace_members SET role = 'admin' WHERE role_id = 'a0000000-0000-0000-0000-000000000002';
UPDATE workspace_members SET role = 'hr_manager' WHERE role_id = 'a0000000-0000-0000-0000-000000000003';
UPDATE workspace_members SET role = 'shift_manager' WHERE role_id = 'a0000000-0000-0000-0000-000000000004';
UPDATE workspace_members SET role = 'team_lead' WHERE role_id = 'a0000000-0000-0000-0000-000000000005';
UPDATE workspace_members SET role = 'employee' WHERE role_id = 'a0000000-0000-0000-0000-000000000006';

-- For custom roles, they don't have a mapping, default them to employee
UPDATE workspace_members SET role = 'employee' WHERE role IS NULL;

-- Make role NOT NULL and drop role_id
ALTER TABLE workspace_members ALTER COLUMN role SET NOT NULL;
ALTER TABLE workspace_members DROP COLUMN role_id;

DROP TABLE IF EXISTS workspace_role_permissions;

-- Revert member_extra_roles
ALTER TABLE member_extra_roles ADD COLUMN role VARCHAR(50);
UPDATE member_extra_roles SET role = 'owner' WHERE role_id = 'a0000000-0000-0000-0000-000000000001';
UPDATE member_extra_roles SET role = 'admin' WHERE role_id = 'a0000000-0000-0000-0000-000000000002';
UPDATE member_extra_roles SET role = 'hr_manager' WHERE role_id = 'a0000000-0000-0000-0000-000000000003';
UPDATE member_extra_roles SET role = 'shift_manager' WHERE role_id = 'a0000000-0000-0000-0000-000000000004';
UPDATE member_extra_roles SET role = 'team_lead' WHERE role_id = 'a0000000-0000-0000-0000-000000000005';
UPDATE member_extra_roles SET role = 'employee' WHERE role_id = 'a0000000-0000-0000-0000-000000000006';
UPDATE member_extra_roles SET role = 'employee' WHERE role IS NULL;
ALTER TABLE member_extra_roles ALTER COLUMN role SET NOT NULL;
ALTER TABLE member_extra_roles DROP COLUMN role_id;

DROP TABLE IF EXISTS workspace_roles;

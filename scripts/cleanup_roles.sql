-- =============================================================================
-- Migration: Unify into Single Admin Role and Remove SaaS/Workspace Roles
-- Removes: owner, platform_admin (merged into admin), support, billing_admin
-- =============================================================================

BEGIN;

-- 1. Ensure system role 'admin' exists
INSERT INTO public.system_roles (id, name, description)
VALUES ('c0000000-0000-0000-0000-000000000001', 'admin', 'System Administrator with full access to infrastructure and global configuration')
ON CONFLICT (id) DO UPDATE SET name = 'admin', description = 'System Administrator with full access to infrastructure and global configuration';

-- Also handle conflict by name if exists
INSERT INTO public.system_roles (id, name, description)
VALUES ('c0000000-0000-0000-0000-000000000001', 'admin', 'System Administrator with full access to infrastructure and global configuration')
ON CONFLICT (name) DO UPDATE SET description = 'System Administrator with full access to infrastructure and global configuration';

-- 2. Migrate any user_system_roles pointing to platform_admin/support/billing_admin
-- Any user with platform_admin gets admin
INSERT INTO user_system_roles (user_id, role_id)
SELECT usr.user_id, 'c0000000-0000-0000-0000-000000000001'
FROM user_system_roles usr
JOIN system_roles sr ON sr.id = usr.role_id
WHERE sr.name = 'platform_admin'
ON CONFLICT DO NOTHING;

-- Remove support, billing_admin assignments and old platform_admin assignments
DELETE FROM user_system_roles
WHERE role_id IN (
    SELECT id FROM system_roles WHERE name IN ('support', 'billing_admin', 'platform_admin') AND id != 'c0000000-0000-0000-0000-000000000001'
);

-- Delete deprecated system roles
DELETE FROM system_roles
WHERE name IN ('support', 'billing_admin', 'platform_admin') AND id != 'c0000000-0000-0000-0000-000000000001';

-- 3. Ensure workspace role 'admin' has full permissions including former owner permissions
INSERT INTO workspace_role_permissions (role_id, permission_name)
SELECT 'a0000000-0000-0000-0000-000000000002', permission_name
FROM workspace_role_permissions
WHERE role_id = 'a0000000-0000-0000-0000-000000000001'
ON CONFLICT DO NOTHING;

-- Migrate any workspace_members with role 'owner' to 'admin'
UPDATE workspace_members
SET role_id = 'a0000000-0000-0000-0000-000000000002'
WHERE role_id = 'a0000000-0000-0000-0000-000000000001';

-- Delete owner permissions and role
DELETE FROM workspace_role_permissions WHERE role_id = 'a0000000-0000-0000-0000-000000000001';
DELETE FROM workspace_roles WHERE id = 'a0000000-0000-0000-0000-000000000001' OR name = 'owner';

COMMIT;

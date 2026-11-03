-- Remove permissions for the retired device domain from existing workspace roles.
DELETE FROM public.workspace_role_permissions
WHERE permission_name IN ('devices:read', 'devices:manage');

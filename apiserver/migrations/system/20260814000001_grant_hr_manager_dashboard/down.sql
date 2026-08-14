DELETE FROM workspace_role_permissions
WHERE (role_id = 'a0000000-0000-0000-0000-000000000003' AND permission_name IN ('dashboard:read', 'dashboard:employee_absence:read'))
   OR (role_id IN ('a0000000-0000-0000-0000-000000000001', 'a0000000-0000-0000-0000-000000000002') AND permission_name = 'dashboard:employee_absence:read');

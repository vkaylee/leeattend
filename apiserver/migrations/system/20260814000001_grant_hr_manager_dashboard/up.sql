-- Grant dashboard access to the hr_manager role so HR Director / HR Operations
-- personas can read attendance analytics. Also grant the new employee_absence
-- (PII) read permission to owner/admin/hr_manager.
INSERT INTO workspace_role_permissions (role_id, permission_name)
VALUES
  -- hr_manager: full dashboard + employee_absence detail
  ('a0000000-0000-0000-0000-000000000003', 'dashboard:read'),
  ('a0000000-0000-0000-0000-000000000003', 'dashboard:employee_absence:read'),
  -- owner
  ('a0000000-0000-0000-0000-000000000001', 'dashboard:employee_absence:read'),
  -- admin
  ('a0000000-0000-0000-0000-000000000002', 'dashboard:employee_absence:read')
ON CONFLICT DO NOTHING;

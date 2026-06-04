INSERT INTO workspace_role_permissions (role_id, permission_name)
VALUES
  ('a0000000-0000-0000-0000-000000000001', 'overtime:request:create_on_behalf'),
  ('a0000000-0000-0000-0000-000000000002', 'overtime:request:create_on_behalf'),
  ('a0000000-0000-0000-0000-000000000003', 'overtime:request:create_on_behalf'),
  ('a0000000-0000-0000-0000-000000000004', 'overtime:request:create_on_behalf'),
  ('a0000000-0000-0000-0000-000000000005', 'overtime:request:create_on_behalf')
ON CONFLICT DO NOTHING;

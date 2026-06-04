INSERT INTO workspace_role_permissions (role_id, permission_name)
VALUES
  -- Owner (a0000000-0000-0000-0000-000000000001)
  ('a0000000-0000-0000-0000-000000000001', 'attendance:adjustments:manage_behalf'),
  ('a0000000-0000-0000-0000-000000000001', 'attendance:adjustments:submit_on_behalf'),
  ('a0000000-0000-0000-0000-000000000001', 'leave:requests:submit_on_behalf'),
  ('a0000000-0000-0000-0000-000000000001', 'overtime:claim_on_behalf'),

  -- Admin (a0000000-0000-0000-0000-000000000002)
  ('a0000000-0000-0000-0000-000000000002', 'attendance:adjustments:manage_behalf'),
  ('a0000000-0000-0000-0000-000000000002', 'attendance:adjustments:submit_on_behalf'),
  ('a0000000-0000-0000-0000-000000000002', 'leave:requests:submit_on_behalf'),
  ('a0000000-0000-0000-0000-000000000002', 'overtime:claim_on_behalf'),

  -- HR Manager (a0000000-0000-0000-0000-000000000003)
  ('a0000000-0000-0000-0000-000000000003', 'attendance:adjustments:manage_behalf'),
  ('a0000000-0000-0000-0000-000000000003', 'attendance:adjustments:submit_on_behalf'),
  ('a0000000-0000-0000-0000-000000000003', 'leave:requests:submit_on_behalf'),
  ('a0000000-0000-0000-0000-000000000003', 'overtime:claim_on_behalf')
ON CONFLICT DO NOTHING;

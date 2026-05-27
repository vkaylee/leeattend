INSERT INTO workspace_role_permissions (role_id, permission_name)
VALUES 
  -- Employee (0006)
  ('a0000000-0000-0000-0000-000000000006', 'overtime:request:create'),
  ('a0000000-0000-0000-0000-000000000006', 'overtime:request:read'),
  ('a0000000-0000-0000-0000-000000000006', 'overtime:claim'),

  -- Team Lead (0005)
  ('a0000000-0000-0000-0000-000000000005', 'overtime:request:create'),
  ('a0000000-0000-0000-0000-000000000005', 'overtime:request:read'),
  ('a0000000-0000-0000-0000-000000000005', 'overtime:claim'),
  ('a0000000-0000-0000-0000-000000000005', 'overtime:approve'),

  -- Shift Manager (0004)
  ('a0000000-0000-0000-0000-000000000004', 'overtime:request:create'),
  ('a0000000-0000-0000-0000-000000000004', 'overtime:request:read'),
  ('a0000000-0000-0000-0000-000000000004', 'overtime:claim'),
  ('a0000000-0000-0000-0000-000000000004', 'overtime:approve'),

  -- HR Manager (0003)
  ('a0000000-0000-0000-0000-000000000003', 'overtime:request:create'),
  ('a0000000-0000-0000-0000-000000000003', 'overtime:request:read'),
  ('a0000000-0000-0000-0000-000000000003', 'overtime:claim'),
  ('a0000000-0000-0000-0000-000000000003', 'overtime:approve'),
  ('a0000000-0000-0000-0000-000000000003', 'overtime:policies:manage'),
  ('a0000000-0000-0000-0000-000000000003', 'overtime:paper:upload'),
  ('a0000000-0000-0000-0000-000000000003', 'overtime:approve:workflow'),
  ('a0000000-0000-0000-0000-000000000003', 'overtime:quota:read'),
  ('a0000000-0000-0000-0000-000000000003', 'approval_workflows:read'),

  -- Admin (0002)
  ('a0000000-0000-0000-0000-000000000002', 'overtime:request:create'),
  ('a0000000-0000-0000-0000-000000000002', 'overtime:request:read'),
  ('a0000000-0000-0000-0000-000000000002', 'overtime:claim'),
  ('a0000000-0000-0000-0000-000000000002', 'overtime:approve'),
  ('a0000000-0000-0000-0000-000000000002', 'overtime:policies:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'overtime:paper:upload'),
  ('a0000000-0000-0000-0000-000000000002', 'overtime:approve:workflow'),
  ('a0000000-0000-0000-0000-000000000002', 'overtime:quota:read'),
  ('a0000000-0000-0000-0000-000000000002', 'approval_workflows:read'),
  ('a0000000-0000-0000-0000-000000000002', 'approval_workflows:manage'),

  -- Owner (0001)
  ('a0000000-0000-0000-0000-000000000001', 'overtime:request:create'),
  ('a0000000-0000-0000-0000-000000000001', 'overtime:request:read'),
  ('a0000000-0000-0000-0000-000000000001', 'overtime:claim'),
  ('a0000000-0000-0000-0000-000000000001', 'overtime:approve'),
  ('a0000000-0000-0000-0000-000000000001', 'overtime:policies:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'overtime:paper:upload'),
  ('a0000000-0000-0000-0000-000000000001', 'overtime:approve:workflow'),
  ('a0000000-0000-0000-0000-000000000001', 'overtime:quota:read'),
  ('a0000000-0000-0000-0000-000000000001', 'approval_workflows:read'),
  ('a0000000-0000-0000-0000-000000000001', 'approval_workflows:manage')
ON CONFLICT DO NOTHING;

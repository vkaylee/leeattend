-- Up Migration: Add Enterprise Premium Permissions to Default Roles

-- Owner ('a0000000-0000-0000-0000-000000000001') gets all
INSERT INTO workspace_role_permissions (role_id, permission_name)
VALUES
  ('a0000000-0000-0000-0000-000000000001', 'audit_logs:read'),
  ('a0000000-0000-0000-0000-000000000001', 'audit_logs:export'),
  ('a0000000-0000-0000-0000-000000000001', 'security:settings:manage'),
  ('a0000000-0000-0000-0000-000000000001', 'employees:bulk_edit'),
  ('a0000000-0000-0000-0000-000000000001', 'attendance:force_clock_out'),
  ('a0000000-0000-0000-0000-000000000001', 'documents:medical:read'),
  ('a0000000-0000-0000-0000-000000000001', 'documents:contract:read'),
  ('a0000000-0000-0000-0000-000000000001', 'payroll:approve'),
  ('a0000000-0000-0000-0000-000000000001', 'payroll:disburse'),
  ('a0000000-0000-0000-0000-000000000001', 'payroll:sensitive:read'),
  ('a0000000-0000-0000-0000-000000000001', 'leave:approve:l1'),
  ('a0000000-0000-0000-0000-000000000001', 'leave:approve:l2')
ON CONFLICT DO NOTHING;

-- Admin ('a0000000-0000-0000-0000-000000000002') gets all except disburse
INSERT INTO workspace_role_permissions (role_id, permission_name)
VALUES
  ('a0000000-0000-0000-0000-000000000002', 'audit_logs:read'),
  ('a0000000-0000-0000-0000-000000000002', 'audit_logs:export'),
  ('a0000000-0000-0000-0000-000000000002', 'security:settings:manage'),
  ('a0000000-0000-0000-0000-000000000002', 'employees:bulk_edit'),
  ('a0000000-0000-0000-0000-000000000002', 'attendance:force_clock_out'),
  ('a0000000-0000-0000-0000-000000000002', 'documents:medical:read'),
  ('a0000000-0000-0000-0000-000000000002', 'documents:contract:read'),
  ('a0000000-0000-0000-0000-000000000002', 'payroll:approve'),
  ('a0000000-0000-0000-0000-000000000002', 'payroll:sensitive:read'),
  ('a0000000-0000-0000-0000-000000000002', 'leave:approve:l1'),
  ('a0000000-0000-0000-0000-000000000002', 'leave:approve:l2')
ON CONFLICT DO NOTHING;

-- HrManager ('a0000000-0000-0000-0000-000000000003') gets HR-specific ones
INSERT INTO workspace_role_permissions (role_id, permission_name)
VALUES
  ('a0000000-0000-0000-0000-000000000003', 'employees:bulk_edit'),
  ('a0000000-0000-0000-0000-000000000003', 'documents:contract:read'),
  ('a0000000-0000-0000-0000-000000000003', 'payroll:approve'),
  ('a0000000-0000-0000-0000-000000000003', 'payroll:sensitive:read'),
  ('a0000000-0000-0000-0000-000000000003', 'leave:approve:l2')
ON CONFLICT DO NOTHING;

-- TeamLead ('a0000000-0000-0000-0000-000000000005') gets L1 approval
INSERT INTO workspace_role_permissions (role_id, permission_name)
VALUES
  ('a0000000-0000-0000-0000-000000000005', 'leave:approve:l1')
ON CONFLICT DO NOTHING;

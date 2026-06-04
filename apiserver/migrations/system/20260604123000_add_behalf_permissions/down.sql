DELETE FROM workspace_role_permissions
WHERE permission_name IN (
  'attendance:adjustments:manage_behalf',
  'attendance:adjustments:submit_on_behalf',
  'leave:requests:submit_on_behalf',
  'overtime:claim_on_behalf'
) AND role_id IN (
  'a0000000-0000-0000-0000-000000000001',
  'a0000000-0000-0000-0000-000000000002',
  'a0000000-0000-0000-0000-000000000003'
);

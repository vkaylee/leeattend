DELETE FROM workspace_role_permissions
WHERE permission_name IN (
  'overtime:claim',
  'overtime:approve',
  'overtime:policies:manage',
  'overtime:paper:upload',
  'overtime:request:create',
  'overtime:request:read',
  'overtime:approve:workflow',
  'overtime:quota:read',
  'approval_workflows:read',
  'approval_workflows:manage'
);

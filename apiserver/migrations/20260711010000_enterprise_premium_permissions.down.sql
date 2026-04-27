-- Down Migration: Remove Enterprise Premium Permissions

DELETE FROM workspace_role_permissions 
WHERE permission_name IN (
    'audit_logs:read',
    'audit_logs:export',
    'security:settings:manage',
    'employees:bulk_edit',
    'attendance:force_clock_out',
    'documents:medical:read',
    'documents:contract:read',
    'payroll:approve',
    'payroll:disburse',
    'payroll:sensitive:read',
    'leave:approve:l1',
    'leave:approve:l2'
);

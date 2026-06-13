-- Reverse: Migrate payroll config scope from "employment" back to "employee"
UPDATE workspace_config_entries wce
SET
    scope_type = 'employee',
    scope_id = emp.employee_id,
    updated_at = NOW()
FROM employments emp
WHERE wce.scope_type = 'employment'
  AND wce.scope_id = emp.id
  AND wce.config_key LIKE 'payroll.%'
  AND emp.deleted_at IS NULL;

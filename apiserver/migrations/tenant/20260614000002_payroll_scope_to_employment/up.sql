-- Migrate payroll config scope from "employee" to "employment"
-- workspace_config_entries stores EAV-style configs with scope_type/scope_id

-- Step 1: Update scope_type and scope_id for payroll configs
-- Map employee scope_id → employment_id via active employments
UPDATE workspace_config_entries wce
SET
    scope_type = 'employment',
    scope_id = emp.id,
    updated_at = NOW()
FROM employments emp
WHERE wce.scope_type = 'employee'
  AND wce.scope_id = emp.employee_id
  AND wce.config_key LIKE 'payroll.%'
  AND emp.status = 'active'
  AND emp.deleted_at IS NULL;

-- Step 2: Handle orphan entries (employees without active employment)
-- Fall back to most recent employment
UPDATE workspace_config_entries wce
SET
    scope_type = 'employment',
    scope_id = sub.employment_id,
    updated_at = NOW()
FROM (
    SELECT DISTINCT ON (e.employee_id) e.id AS employment_id, e.employee_id
    FROM employments e
    WHERE e.deleted_at IS NULL
    ORDER BY e.employee_id, e.start_date DESC
) sub
WHERE wce.scope_type = 'employee'
  AND wce.scope_id = sub.employee_id
  AND wce.config_key LIKE 'payroll.%';

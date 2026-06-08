SELECT id AS employee_id, department_id
FROM employments
WHERE workspace_id = $1 AND status = 'active' AND deleted_at IS NULL AND department_id IS NOT NULL

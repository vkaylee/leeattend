SELECT d.id AS department_id,
       d.name::text AS department_name,
       COUNT(DISTINCT e.id)::bigint AS total_employees
FROM departments d
JOIN employments emp ON emp.department_id = d.id AND emp.workspace_id = $1 AND emp.deleted_at IS NULL
JOIN employees e ON e.id = emp.employee_id AND e.workspace_id = $1 AND e.deleted_at IS NULL
WHERE d.workspace_id = $1
  AND emp.start_date <= $2 AND (emp.end_date IS NULL OR emp.end_date >= $2)
  AND emp.status = 'active'
GROUP BY d.id, d.name


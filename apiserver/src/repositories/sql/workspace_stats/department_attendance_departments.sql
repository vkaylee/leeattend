SELECT d.id AS department_id,
       d.name::text AS department_name,
       COUNT(DISTINCT e.id)::bigint AS total_employees
FROM departments d
JOIN employee_department_history edh ON edh.department_id = d.id
JOIN employees e ON e.id = edh.employee_id AND e.workspace_id = $1
JOIN employments emp ON e.id = emp.employee_id AND emp.workspace_id = $1 AND emp.status = 'active' AND emp.deleted_at IS NULL
WHERE d.workspace_id = $1
  AND edh.effective_date <= $2
  AND emp.start_date <= $2 AND (emp.end_date IS NULL OR emp.end_date >= $2)
  AND edh.id = (
    SELECT edh2.id FROM employee_department_history edh2
    WHERE edh2.employee_id = edh.employee_id AND edh2.effective_date <= $2
    ORDER BY edh2.effective_date DESC LIMIT 1
  )
GROUP BY d.id, d.name

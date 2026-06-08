SELECT
    COALESCE(d.name, 'Unassigned') AS department_name,
    COUNT(*)::bigint AS count
FROM employees e
JOIN employments emp ON e.id = emp.employee_id
LEFT JOIN LATERAL (
    SELECT edh.department_id
    FROM employee_department_history edh
    WHERE edh.employee_id = e.id
    ORDER BY edh.effective_date DESC
    LIMIT 1
) latest_dept ON TRUE
LEFT JOIN departments d ON d.id = latest_dept.department_id
WHERE emp.workspace_id = $1
  AND emp.deleted_at IS NULL
  AND emp.status = 'terminated'
  AND emp.end_date IS NOT NULL
  AND emp.end_date >= $2 AND emp.end_date <= $3
GROUP BY d.name
ORDER BY count DESC

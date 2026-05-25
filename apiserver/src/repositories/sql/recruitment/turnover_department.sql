SELECT
    COALESCE(d.name, 'Unassigned') AS department_name,
    COUNT(*)::bigint AS count
FROM employees e
LEFT JOIN LATERAL (
    SELECT edh.department_id
    FROM employee_department_history edh
    WHERE edh.employee_id = e.id
    ORDER BY edh.effective_date DESC
    LIMIT 1
) latest_dept ON TRUE
LEFT JOIN departments d ON d.id = latest_dept.department_id
WHERE e.workspace_id = $1 AND e.is_active = FALSE
  AND e.termination_date IS NOT NULL
  AND e.termination_date >= $2 AND e.termination_date <= $3
GROUP BY d.name
ORDER BY count DESC

SELECT
    COALESCE(mgr_e.full_name, 'Unassigned')::text AS manager_name,
    COUNT(DISTINCT emp.employee_id)::bigint AS count
FROM employees e
JOIN employments emp ON e.id = emp.employee_id
LEFT JOIN employments mgr_emp ON mgr_emp.id = emp.manager_employment_id
LEFT JOIN employees mgr_e ON mgr_e.id = mgr_emp.employee_id
WHERE emp.workspace_id = $1
  AND emp.deleted_at IS NULL
  AND emp.status = 'terminated'
  AND emp.end_date IS NOT NULL
  AND emp.end_date >= $2 AND emp.end_date <= $3
GROUP BY mgr_e.full_name
ORDER BY count DESC

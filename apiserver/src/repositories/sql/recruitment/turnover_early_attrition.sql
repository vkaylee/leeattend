-- Early attrition: leavers whose tenure at termination was under 90 days.
SELECT COUNT(DISTINCT emp.employee_id)::bigint AS count
FROM employees e
JOIN employments emp ON e.id = emp.employee_id
WHERE emp.workspace_id = $1
  AND emp.deleted_at IS NULL
  AND emp.status = 'terminated'
  AND emp.end_date IS NOT NULL
  AND emp.end_date >= $2 AND emp.end_date <= $3
  AND (emp.end_date - emp.start_date) < 90

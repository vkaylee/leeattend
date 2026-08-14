SELECT
    COALESCE(l.name, 'Unassigned')::text AS location_name,
    COUNT(DISTINCT emp.employee_id)::bigint AS count
FROM employees e
JOIN employments emp ON e.id = emp.employee_id
LEFT JOIN work_locations l ON l.id = emp.primary_location_id
WHERE emp.workspace_id = $1
  AND emp.deleted_at IS NULL
  AND emp.status = 'terminated'
  AND emp.end_date IS NOT NULL
  AND emp.end_date >= $2 AND emp.end_date <= $3
GROUP BY l.name
ORDER BY count DESC

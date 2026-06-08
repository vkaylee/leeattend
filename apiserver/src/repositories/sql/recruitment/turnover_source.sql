SELECT
    COALESCE(rs.source_type::text, 'direct') AS source_type,
    COALESCE(rs.name, 'Direct Hire') AS source_name,
    COUNT(*)::bigint AS count
FROM employees e
JOIN employments emp ON e.id = emp.employee_id
LEFT JOIN recruitment_sources rs ON e.recruitment_source_id = rs.id
WHERE emp.workspace_id = $1
  AND emp.deleted_at IS NULL
  AND emp.status = 'terminated'
  AND emp.end_date IS NOT NULL
  AND emp.end_date >= $2 AND emp.end_date <= $3
GROUP BY rs.source_type, rs.name
ORDER BY count DESC

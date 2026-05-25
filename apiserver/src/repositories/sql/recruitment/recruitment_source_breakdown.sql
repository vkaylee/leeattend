SELECT
    COALESCE(rs.source_type::text, 'direct') AS source_type,
    COALESCE(rs.name, 'Direct Hire') AS source_name,
    COUNT(*) AS count
FROM employees e
LEFT JOIN recruitment_sources rs ON e.recruitment_source_id = rs.id
WHERE e.workspace_id = $1 AND e.is_active = TRUE
GROUP BY rs.source_type, rs.name
ORDER BY count DESC

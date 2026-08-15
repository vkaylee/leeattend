-- Per-source terminations within the period, for sourced hires (recruitment effectiveness).
--   $1 workspace_id, $2 start_date, $3 end_date,
--   $4 department_ids (uuid[] | NULL), $5 location_ids (uuid[] | NULL),
--   $6 job_roles (text[] | NULL), $7 source_types (text[] | NULL),
--   $8 source_ids (uuid[] | NULL)
--
-- A termination is a terminated employment with end_date in [start, end]. Filters
-- resolve through the terminated employment itself (department/location/job at
-- termination), while source type/id come from the employee's recruitment_source.
SELECT
    rs.id AS source_id,
    COALESCE(rs.source_type::text, 'unknown') AS source_type,
    COALESCE(rs.name, 'Unknown source') AS source_name,
    COUNT(DISTINCT emp.employee_id)::bigint AS count
FROM employments emp
JOIN employees e ON e.id = emp.employee_id
LEFT JOIN recruitment_sources rs ON e.recruitment_source_id = rs.id
WHERE emp.workspace_id = $1
  AND emp.deleted_at IS NULL
  AND emp.status = 'terminated'
  AND emp.end_date IS NOT NULL
  AND emp.end_date >= $2 AND emp.end_date <= $3
  AND e.deleted_at IS NULL
  AND ($4::uuid[] IS NULL OR emp.department_id = ANY($4))
  AND ($5::uuid[] IS NULL OR emp.primary_location_id = ANY($5))
  AND ($6::text[] IS NULL OR emp.job_title = ANY($6))
  AND ($7::text[] IS NULL OR rs.source_type::text = ANY($7))
  AND ($8::uuid[] IS NULL OR e.recruitment_source_id = ANY($8))
GROUP BY rs.id, rs.source_type, rs.name
ORDER BY count DESC, source_name ASC

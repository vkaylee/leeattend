-- Per-source 90-day retention for the recruitment effectiveness dashboard.
--   $1 workspace_id, $2 start_date, $3 end_date,
--   $4 department_ids (uuid[] | NULL), $5 location_ids (uuid[] | NULL),
--   $6 job_roles (text[] | NULL), $7 source_types (text[] | NULL),
--   $8 source_ids (uuid[] | NULL)
--
-- retained_90d = hires still active 90 days after hire: not terminated at all, or
-- terminated strictly AFTER hire_date + 90 days. Mirrors turnover_early_attrition.sql
-- semantics (tenure < 90 days is a loss).
SELECT
    rs.id AS source_id,
    COALESCE(rs.source_type::text, 'unknown') AS source_type,
    COALESCE(rs.name, 'Unknown source') AS source_name,
    COUNT(DISTINCT e.id)::bigint AS retained_90d
FROM employees e
LEFT JOIN recruitment_sources rs ON e.recruitment_source_id = rs.id
JOIN employments emp
  ON emp.employee_id = e.id
 AND emp.workspace_id = $1
 AND emp.deleted_at IS NULL
 AND emp.start_date = e.hire_date
WHERE e.workspace_id = $1
  AND e.deleted_at IS NULL
  AND e.hire_date >= $2 AND e.hire_date <= $3
  AND (e.termination_date IS NULL OR e.termination_date > e.hire_date + 90)
  AND ($4::uuid[] IS NULL OR emp.department_id = ANY($4))
  AND ($5::uuid[] IS NULL OR emp.primary_location_id = ANY($5))
  AND ($6::text[] IS NULL OR emp.job_title = ANY($6))
  AND ($7::text[] IS NULL OR rs.source_type::text = ANY($7))
  AND ($8::uuid[] IS NULL OR e.recruitment_source_id = ANY($8))
GROUP BY rs.id, rs.source_type, rs.name
ORDER BY retained_90d DESC, source_name ASC

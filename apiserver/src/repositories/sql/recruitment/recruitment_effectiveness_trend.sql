-- Monthly hires trend for the recruitment effectiveness dashboard.
--   $1 workspace_id, $2 start_date, $3 end_date,
--   $4 department_ids (uuid[] | NULL), $5 location_ids (uuid[] | NULL),
--   $6 job_roles (text[] | NULL), $7 source_types (text[] | NULL),
--   $8 source_ids (uuid[] | NULL)
--
-- Zero-filled via generate_series so every month in the period appears even with
-- no hires. Hires are filtered in a subquery (the same predicate as
-- recruitment_effectiveness_hires.sql) so department/location/role filters apply
-- to the employee, then LEFT JOIN months → hires keeps all months present.
WITH months AS (
    SELECT generate_series(
        date_trunc('month', $2::date)::date,
        date_trunc('month', $3::date)::date,
        '1 month'::interval
    )::date AS m
),
hires AS (
    SELECT
        date_trunc('month', e.hire_date)::date AS hire_month,
        e.id
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
      AND ($4::uuid[] IS NULL OR emp.department_id = ANY($4))
      AND ($5::uuid[] IS NULL OR emp.primary_location_id = ANY($5))
      AND ($6::text[] IS NULL OR emp.job_title = ANY($6))
      AND ($7::text[] IS NULL OR rs.source_type::text = ANY($7))
      AND ($8::uuid[] IS NULL OR e.recruitment_source_id = ANY($8))
)
SELECT
    to_char(m.m, 'YYYY-MM') AS month,
    COUNT(DISTINCT h.id)::bigint AS hires
FROM months m
LEFT JOIN hires h ON h.hire_month = m.m
GROUP BY m.m
ORDER BY m.m ASC

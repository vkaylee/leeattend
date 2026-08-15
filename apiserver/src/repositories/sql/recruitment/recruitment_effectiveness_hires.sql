-- Per-source hires + cost for the recruitment effectiveness dashboard.
--   $1 workspace_id, $2 start_date, $3 end_date,
--   $4 department_ids (uuid[] | NULL), $5 location_ids (uuid[] | NULL),
--   $6 job_roles (text[] | NULL), $7 source_types (text[] | NULL),
--   $8 source_ids (uuid[] | NULL)
--
-- A hire is an employee with hire_date in [start, end]. Department/location/job-role
-- filters resolve through the FIRST employment (start_date == hire_date), so the
-- 1:1 join keeps COUNT(*) exact for the factory's single-employment data. Re-hires
-- with multiple employments share a hire_date and are counted once via DISTINCT.
--
-- Cost semantics: the fee is a per-source lump sum (agency_fee_amount for agency
-- sources, referral_bonus_amount for referral sources), allocated across hires as
-- fee / hires. Direct sources carry no cost.
SELECT
    rs.id AS source_id,
    COALESCE(rs.source_type::text, 'unknown') AS source_type,
    COALESCE(rs.name, 'Unknown source') AS source_name,
    COUNT(DISTINCT e.id)::bigint AS hires,
    -- Fee is a per-source lump sum: DISTINCT prevents it being multiplied by the
    -- number of hires when several employees share one source row.
    SUM(
        DISTINCT CASE
            WHEN rs.source_type = 'agency' THEN rs.agency_fee_amount
            WHEN rs.source_type = 'referral' THEN rs.referral_bonus_amount
            ELSE NULL
        END
    )::numeric AS total_cost,
    BOOL_OR(
        (rs.source_type = 'agency' AND rs.agency_fee_amount IS NOT NULL)
        OR (rs.source_type = 'referral' AND rs.referral_bonus_amount IS NOT NULL)
    ) AS cost_available
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
GROUP BY rs.id, rs.source_type, rs.name
ORDER BY hires DESC, source_name ASC

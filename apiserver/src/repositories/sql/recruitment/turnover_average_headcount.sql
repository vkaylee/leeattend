-- Average headcount over the period = mean of month-start headcounts.
-- Also returns headcount_start (active employments at the first day of the period)
-- so retention rate can be computed in Rust.
WITH months AS (
    SELECT generate_series(
        date_trunc('month', $2::date)::date,
        date_trunc('month', $3::date)::date,
        '1 month'::interval
    )::date AS m
)
SELECT
    COUNT(*)::bigint AS n_months,
    COALESCE(AVG(hc), 0)::bigint AS average_headcount,
    COALESCE((SELECT COUNT(DISTINCT emp.employee_id)
              FROM employments emp
              WHERE emp.workspace_id = $1
                AND emp.deleted_at IS NULL
                AND emp.start_date <= $2::date
                AND (emp.end_date IS NULL OR emp.end_date >= $2::date)), 0)::bigint AS headcount_start
FROM months
LEFT JOIN LATERAL (
    SELECT COUNT(DISTINCT emp.employee_id) AS hc
    FROM employments emp
    WHERE emp.workspace_id = $1
      AND emp.deleted_at IS NULL
      AND emp.start_date <= m
      AND (emp.end_date IS NULL OR emp.end_date >= m)
) hc ON true

-- Cohort retention survival matrix.
--   $1 workspace_id, $2 start_date, $3 end_date, $4 min_cohort_size, $5 period_months
-- Cohorts = hire months fully inside the period, filtered by cohort_size >= $4.
-- For each (cohort, months_since_hire) cell, count employments in that cohort that
-- are still active at the as-of date. Bounded: cohorts ∩ period, offsets 0..period_months.
WITH cohort AS (
    SELECT
        date_trunc('month', emp.start_date)::date AS cohort_month,
        COUNT(DISTINCT emp.employee_id)::bigint AS cohort_size
    FROM employees e
    JOIN employments emp ON e.id = emp.employee_id
    WHERE emp.workspace_id = $1
      AND emp.deleted_at IS NULL
      AND emp.start_date >= date_trunc('month', $2::date)::date
      AND emp.start_date <= $3::date
    GROUP BY 1
    HAVING COUNT(DISTINCT emp.employee_id) >= $4
),
offsets AS (
    SELECT generate_series(0, GREATEST(0, $5::int - 1))::int AS months_since_hire
)
SELECT
    to_char(c.cohort_month, 'YYYY-MM') AS cohort_month,
    o.months_since_hire::bigint,
    c.cohort_size,
    COALESCE(surv.active_count, 0)::bigint AS active_count
FROM cohort c
CROSS JOIN offsets o
LEFT JOIN LATERAL (
    SELECT COUNT(DISTINCT emp.employee_id)::bigint AS active_count
    FROM employees e
    JOIN employments emp ON e.id = emp.employee_id
    WHERE emp.workspace_id = $1
      AND emp.deleted_at IS NULL
      AND date_trunc('month', emp.start_date)::date = c.cohort_month
      -- As-of the last day of the offset month: month 0 = end of the hire month,
      -- so every hire in that month counts as active at month 0.
      AND emp.start_date <= date_trunc('month', (c.cohort_month + make_interval(months => o.months_since_hire)))::date + interval '1 month' - interval '1 day'
      AND (emp.end_date IS NULL OR emp.end_date >= date_trunc('month', (c.cohort_month + make_interval(months => o.months_since_hire)))::date + interval '1 month' - interval '1 day')
) surv ON true
ORDER BY c.cohort_month ASC, o.months_since_hire ASC

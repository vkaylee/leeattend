-- One row per month bucket between $2 and $3: month-start headcount + leavers
-- terminated within that month. Zero-filled via generate_series + LEFT JOIN.
WITH months AS (
    SELECT generate_series(
        date_trunc('month', $2::date)::date,
        date_trunc('month', $3::date)::date,
        '1 month'::interval
    )::date AS m
),
leavers AS (
    SELECT
        date_trunc('month', emp.end_date)::date AS m,
        COUNT(DISTINCT emp.employee_id)::bigint AS leavers
    FROM employees e
    JOIN employments emp ON e.id = emp.employee_id
    WHERE emp.workspace_id = $1
      AND emp.deleted_at IS NULL
      AND emp.status = 'terminated'
      AND emp.end_date IS NOT NULL
      AND emp.end_date >= $2 AND emp.end_date <= $3
    GROUP BY 1
),
headcounts AS (
    SELECT
        m.m AS m,
        COUNT(DISTINCT emp.employee_id)::bigint AS headcount
    FROM months m
    LEFT JOIN employments emp
      ON emp.workspace_id = $1
     AND emp.deleted_at IS NULL
     AND emp.start_date <= m.m
     AND (emp.end_date IS NULL OR emp.end_date >= m.m)
    GROUP BY m.m
)
SELECT
    to_char(hc.m, 'YYYY-MM') AS month,
    COALESCE(hc.headcount, 0)::bigint AS headcount,
    COALESCE(lv.leavers, 0)::bigint AS leavers
FROM headcounts hc
LEFT JOIN leavers lv ON lv.m = hc.m
ORDER BY hc.m ASC

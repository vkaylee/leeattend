WITH days AS (
    SELECT d::date AS date
    FROM generate_series(($1::date - INTERVAL '29 days'), $1::date, '1 day'::interval) AS d
),
active_employments AS (
    SELECT id FROM employments WHERE workspace_id = $2 AND status = 'active' AND deleted_at IS NULL
),
daily_present AS (
    SELECT d.date,
           COUNT(DISTINCT te.employment_id) AS present
    FROM days d
    LEFT JOIN time_entries te
        ON te.punch_in_time >= $3
        AND te.punch_in_time < $4
        AND (te.punch_in_time AT TIME ZONE $5)::date = d.date
        AND te.employment_id IN (SELECT id FROM active_employments)
    GROUP BY d.date
),
daily_leave AS (
    SELECT d.date,
           COUNT(DISTINCT lr.employment_id) AS on_leave
    FROM days d
    LEFT JOIN (
        leave_requests lr
        JOIN employments emp ON lr.employment_id = emp.id AND emp.deleted_at IS NULL
    ) ON lr.status IN ('approved', 'pre_approved')
      AND lr.start_date <= d.date
      AND lr.end_date >= d.date
      AND emp.id IN (SELECT id FROM active_employments)
    GROUP BY d.date
)
SELECT dp.date,
       COALESCE(dp.present, 0)::bigint AS present,
       COALESCE(dle.on_leave, 0)::bigint AS on_leave
FROM daily_present dp
LEFT JOIN daily_leave dle ON dp.date = dle.date
ORDER BY dp.date ASC

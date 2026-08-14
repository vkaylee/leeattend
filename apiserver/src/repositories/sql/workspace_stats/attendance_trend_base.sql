WITH days AS (
    SELECT d::date AS date
    FROM generate_series($1::date, $2::date, '1 day'::interval) AS d
),
daily_active AS (
    SELECT d.date,
           COUNT(DISTINCT emp.employee_id)::bigint AS active_employees
    FROM days d
    JOIN employments emp
      ON emp.workspace_id = $3
     AND emp.status = 'active'
     AND emp.deleted_at IS NULL
     AND emp.start_date <= d.date
     AND (emp.end_date IS NULL OR emp.end_date >= d.date)
     AND ($7::uuid[] IS NULL OR emp.department_id = ANY($7))
    JOIN employees e
      ON e.id = emp.employee_id
     AND e.workspace_id = $3
     AND e.deleted_at IS NULL
    GROUP BY d.date
),
daily_present AS (
    SELECT d.date,
           COUNT(DISTINCT emp.employee_id)::bigint AS present
    FROM days d
    JOIN time_entries te
      ON te.punch_in_time >= $4
     AND te.punch_in_time < $5
     AND (te.punch_in_time AT TIME ZONE $6)::date = d.date
    JOIN employments emp
      ON emp.id = te.employment_id
     AND emp.workspace_id = $3
     AND emp.status = 'active'
     AND emp.deleted_at IS NULL
     AND emp.start_date <= d.date
     AND (emp.end_date IS NULL OR emp.end_date >= d.date)
     AND ($7::uuid[] IS NULL OR emp.department_id = ANY($7))
    JOIN employees e
      ON e.id = emp.employee_id
     AND e.workspace_id = $3
     AND e.deleted_at IS NULL
    GROUP BY d.date
),
daily_leave AS (
    SELECT d.date,
           COUNT(DISTINCT emp.employee_id)::bigint AS on_leave
    FROM days d
    JOIN leave_requests lr
      ON lr.status IN ('approved', 'pre_approved')
     AND lr.start_date <= d.date
     AND lr.end_date >= d.date
    JOIN employments emp
      ON emp.id = lr.employment_id
     AND emp.workspace_id = $3
     AND emp.status = 'active'
     AND emp.deleted_at IS NULL
     AND emp.start_date <= d.date
     AND (emp.end_date IS NULL OR emp.end_date >= d.date)
     AND ($7::uuid[] IS NULL OR emp.department_id = ANY($7))
    JOIN employees e
      ON e.id = emp.employee_id
     AND e.workspace_id = $3
     AND e.deleted_at IS NULL
    GROUP BY d.date
)
SELECT d.date,
       COALESCE(dp.present, 0)::bigint AS present,
       COALESCE(dl.on_leave, 0)::bigint AS on_leave,
       COALESCE(da.active_employees, 0)::bigint AS active_employees
FROM days d
LEFT JOIN daily_present dp ON dp.date = d.date
LEFT JOIN daily_leave dl ON dl.date = d.date
LEFT JOIN daily_active da ON da.date = d.date
ORDER BY d.date ASC

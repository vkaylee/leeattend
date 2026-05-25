WITH first_punches AS (
    SELECT te.employee_id,
           (te.punch_in_time AT TIME ZONE $3)::date AS work_date,
           MIN(te.punch_in_time AT TIME ZONE $3)::time AS first_punch
    FROM time_entries te
    WHERE (te.punch_in_time AT TIME ZONE $3)::date >= ($1::date - INTERVAL '29 days')
      AND (te.punch_in_time AT TIME ZONE $3)::date <= $1
      AND te.employee_id IN (SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE)
    GROUP BY te.employee_id, (te.punch_in_time AT TIME ZONE $3)::date
),
late_entries AS (
    SELECT fp.work_date,
           EXTRACT(ISODOW FROM fp.work_date)::int AS dow,
           EXTRACT(EPOCH FROM (fp.first_punch - rs.start_time)) / 60.0 AS late_minutes
    FROM first_punches fp
    JOIN resolve_employee_shift($2, fp.work_date) rs ON rs.employee_id = fp.employee_id
    WHERE rs.start_time IS NOT NULL
      AND fp.first_punch > rs.start_time
)
SELECT dow,
       COUNT(*)::bigint AS late_count,
       COALESCE(AVG(late_minutes), 0)::float8 AS avg_late_minutes
FROM late_entries
GROUP BY dow
ORDER BY dow

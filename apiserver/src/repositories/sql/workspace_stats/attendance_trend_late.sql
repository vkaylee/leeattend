WITH first_punches AS (
    SELECT te.employee_id,
           (te.punch_in_time AT TIME ZONE $3)::date AS work_date,
           MIN(te.punch_in_time AT TIME ZONE $3)::time AS first_punch
    FROM time_entries te
    WHERE (te.punch_in_time AT TIME ZONE $3)::date >= ($1::date - INTERVAL '29 days')
      AND (te.punch_in_time AT TIME ZONE $3)::date <= $1::date
      AND te.employee_id IN (SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE)
    GROUP BY te.employee_id, (te.punch_in_time AT TIME ZONE $3)::date
)
SELECT fp.work_date,
       COUNT(DISTINCT fp.employee_id)::bigint AS late_count
FROM first_punches fp
JOIN resolve_employee_shift($2, fp.work_date) ar ON ar.employee_id = fp.employee_id
WHERE fp.first_punch > ar.start_time
GROUP BY fp.work_date

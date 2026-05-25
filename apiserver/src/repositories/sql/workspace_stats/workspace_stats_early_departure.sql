WITH today_completed AS (
    SELECT te.employee_id,
           MAX(te.punch_out_time AT TIME ZONE $4)::time as last_punch_out
    FROM time_entries te
    WHERE (te.punch_in_time AT TIME ZONE $4)::date = $3
      AND te.punch_out_time IS NOT NULL
      AND te.employee_id IN (SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE)
    GROUP BY te.employee_id
)
SELECT COUNT(*)::bigint as count
FROM today_completed tc
JOIN resolve_employee_shift($2, $3) ar ON ar.employee_id = tc.employee_id
WHERE tc.last_punch_out < ar.end_time

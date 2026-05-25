WITH today_punches AS (
    SELECT te.employee_id,
           MIN(te.punch_in_time AT TIME ZONE $4)::time AS first_punch
    FROM time_entries te
    WHERE (te.punch_in_time AT TIME ZONE $4)::date = $3
      AND te.employee_id IN (SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE)
    GROUP BY te.employee_id
)
SELECT COUNT(*)::bigint as count
FROM today_punches tp
JOIN resolve_employee_shift($2, $3) ar ON ar.employee_id = tp.employee_id
WHERE tp.first_punch > ar.start_time

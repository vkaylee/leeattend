SELECT te.employee_id,
       (te.punch_in_time AT TIME ZONE $3)::date AS work_date,
       MIN(te.punch_in_time AT TIME ZONE $3)::time AS first_punch
FROM time_entries te
WHERE (te.punch_in_time AT TIME ZONE $3)::date >= $1
  AND (te.punch_in_time AT TIME ZONE $3)::date <= $2
  AND te.employee_id IN (SELECT id FROM employees WHERE workspace_id = $4 AND is_active = TRUE)
GROUP BY te.employee_id, (te.punch_in_time AT TIME ZONE $3)::date

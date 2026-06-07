SELECT te.employee_id,
       MAX(te.punch_out_time AT TIME ZONE $3)::time AS last_punch_out
FROM time_entries te
WHERE (te.punch_in_time AT TIME ZONE $3)::date = $1
  AND te.punch_out_time IS NOT NULL
  AND te.employee_id IN (SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE)
GROUP BY te.employee_id

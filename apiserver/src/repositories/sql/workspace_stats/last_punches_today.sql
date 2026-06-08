SELECT te.employment_id AS employee_id,
       MAX(te.punch_out_time AT TIME ZONE $3)::time AS last_punch_out
FROM time_entries te
WHERE (te.punch_in_time AT TIME ZONE $3)::date = $1
  AND te.punch_out_time IS NOT NULL
  AND te.employment_id IN (SELECT id FROM employments WHERE workspace_id = $2 AND status = 'active' AND deleted_at IS NULL)
GROUP BY te.employment_id

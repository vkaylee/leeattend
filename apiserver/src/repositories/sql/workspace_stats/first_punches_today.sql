SELECT te.employment_id AS employee_id,
       MIN(te.punch_in_time AT TIME ZONE $4)::time AS first_punch
FROM time_entries te
WHERE te.punch_in_time >= $1
  AND te.punch_in_time < $2
  AND te.employment_id IN (SELECT id FROM employments WHERE workspace_id = $3 AND status = 'active' AND deleted_at IS NULL)
GROUP BY te.employment_id

SELECT te.employment_id AS employee_id,
       (te.punch_in_time AT TIME ZONE $3)::date AS work_date,
       MIN(te.punch_in_time AT TIME ZONE $3)::time AS first_punch
FROM time_entries te
WHERE (te.punch_in_time AT TIME ZONE $3)::date >= $1
  AND (te.punch_in_time AT TIME ZONE $3)::date <= $2
  AND te.employment_id IN (SELECT id FROM employments WHERE workspace_id = $4 AND status = 'active' AND deleted_at IS NULL)
GROUP BY te.employment_id, (te.punch_in_time AT TIME ZONE $3)::date

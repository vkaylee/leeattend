SELECT te.employment_id,
       (te.punch_in_time AT TIME ZONE $3)::date AS work_date,
       MIN(te.punch_in_time AT TIME ZONE $3)::time AS first_punch
FROM time_entries te
JOIN employments emp
  ON emp.id = te.employment_id
 AND emp.workspace_id = $4
 AND emp.status = 'active'
 AND emp.deleted_at IS NULL
 AND (te.punch_in_time AT TIME ZONE $3)::date >= emp.start_date
 AND (emp.end_date IS NULL OR (te.punch_in_time AT TIME ZONE $3)::date <= emp.end_date)
 AND ($5::uuid[] IS NULL OR emp.department_id = ANY($5))
JOIN employees e
  ON e.id = emp.employee_id
 AND e.workspace_id = $4
 AND e.deleted_at IS NULL
WHERE te.punch_in_time >= $1
  AND te.punch_in_time < $2
GROUP BY te.employment_id, (te.punch_in_time AT TIME ZONE $3)::date

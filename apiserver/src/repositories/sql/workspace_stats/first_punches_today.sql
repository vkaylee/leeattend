SELECT te.employment_id,
       MIN(te.punch_in_time AT TIME ZONE $4)::time AS first_punch
FROM time_entries te
JOIN employments emp
  ON emp.id = te.employment_id
 AND emp.workspace_id = $3
 AND emp.status = 'active'
 AND emp.deleted_at IS NULL
 AND $5 BETWEEN emp.start_date AND COALESCE(emp.end_date, '9999-12-31'::date)
JOIN employees e
  ON e.id = emp.employee_id
 AND e.workspace_id = $3
 AND e.deleted_at IS NULL
WHERE te.punch_in_time >= $1
  AND te.punch_in_time < $2
GROUP BY te.employment_id

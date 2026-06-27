WITH latest_dept AS (
    SELECT emp.employee_id, emp.department_id
    FROM employments emp
    JOIN employees e ON e.id = emp.employee_id AND e.workspace_id = $1 AND e.deleted_at IS NULL
    WHERE emp.workspace_id = $1
      AND emp.deleted_at IS NULL
      AND emp.start_date <= $3 AND (emp.end_date IS NULL OR emp.end_date >= $3)
      AND emp.status = 'active'
)
SELECT ld.department_id,
       COUNT(DISTINCT emp_te.employee_id)::bigint AS present_count
FROM time_entries te
JOIN employments emp_te ON te.employment_id = emp_te.id AND emp_te.workspace_id = $1 AND emp_te.deleted_at IS NULL
JOIN latest_dept ld ON ld.employee_id = emp_te.employee_id
WHERE te.punch_in_time >= $2
  AND te.punch_in_time < $4
GROUP BY ld.department_id

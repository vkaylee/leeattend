WITH latest_dept AS (
    SELECT DISTINCT ON (edh.employee_id) edh.employee_id, edh.department_id
    FROM employee_department_history edh
    JOIN employees e ON e.id = edh.employee_id AND e.workspace_id = $1
    JOIN employments emp ON e.id = emp.employee_id AND emp.workspace_id = $1 AND emp.status = 'active' AND emp.deleted_at IS NULL
    WHERE edh.effective_date <= $3
      AND emp.start_date <= $3 AND (emp.end_date IS NULL OR emp.end_date >= $3)
    ORDER BY edh.employee_id, edh.effective_date DESC
)
SELECT ld.department_id,
       COUNT(DISTINCT emp_te.employee_id)::bigint AS present_count
FROM time_entries te
JOIN employments emp_te ON te.employment_id = emp_te.id AND emp_te.workspace_id = $1 AND emp_te.deleted_at IS NULL
JOIN latest_dept ld ON ld.employee_id = emp_te.employee_id
WHERE te.punch_in_time >= $2
GROUP BY ld.department_id

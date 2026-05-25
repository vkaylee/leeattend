WITH latest_dept AS (
    SELECT DISTINCT ON (edh.employee_id) edh.employee_id, edh.department_id
    FROM employee_department_history edh
    JOIN employees e ON e.id = edh.employee_id AND e.is_active = TRUE AND e.workspace_id = $1
    WHERE edh.effective_date <= $3
    ORDER BY edh.employee_id, edh.effective_date DESC
)
SELECT ld.department_id,
       COUNT(DISTINCT lr.employee_id)::bigint AS leave_count
FROM leave_requests lr
JOIN latest_dept ld ON ld.employee_id = lr.employee_id
WHERE lr.status IN ('approved', 'pre_approved')
  AND lr.start_date <= $3 AND lr.end_date >= $2
GROUP BY ld.department_id

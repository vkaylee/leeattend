WITH latest_dept AS (
    SELECT DISTINCT ON (edh.employee_id) edh.employee_id, edh.department_id
    FROM employee_department_history edh
    JOIN employees e ON e.id = edh.employee_id AND e.is_active = TRUE AND e.workspace_id = $1
    WHERE edh.effective_date <= $3
    ORDER BY edh.employee_id, edh.effective_date DESC
)
SELECT ld.department_id,
       COUNT(DISTINCT te.employee_id)::bigint AS present_count
FROM time_entries te
JOIN latest_dept ld ON ld.employee_id = te.employee_id
WHERE te.punch_in_time >= $2
GROUP BY ld.department_id

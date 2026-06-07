SELECT DISTINCT ON (edh.employee_id) edh.employee_id, edh.department_id
FROM employee_department_history edh
JOIN employees e ON e.id = edh.employee_id AND e.is_active = TRUE AND e.workspace_id = $1
WHERE edh.effective_date <= $2
ORDER BY edh.employee_id, edh.effective_date DESC

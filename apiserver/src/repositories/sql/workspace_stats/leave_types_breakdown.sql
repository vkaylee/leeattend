SELECT COALESCE(lt.name, 'Other')::text AS leave_type,
       COUNT(*)::bigint AS count
FROM leave_requests lr
LEFT JOIN leave_types lt ON lt.id = lr.leave_type_id
JOIN employments emp ON lr.employment_id = emp.id AND emp.deleted_at IS NULL
WHERE lr.status IN ('approved', 'pre_approved')
  AND lr.start_date <= $2
  AND lr.end_date >= $1
  AND emp.workspace_id = $3
  AND emp.employee_id IN (
      SELECT e.id FROM employees e
      JOIN employments emp_inner ON e.id = emp_inner.employee_id
      WHERE e.workspace_id = $3 AND emp_inner.status = 'active' AND emp_inner.deleted_at IS NULL
        AND CURRENT_DATE BETWEEN emp_inner.start_date AND COALESCE(emp_inner.end_date, '9999-12-31'::date)
  )
GROUP BY COALESCE(lt.name, 'Other')
ORDER BY count DESC

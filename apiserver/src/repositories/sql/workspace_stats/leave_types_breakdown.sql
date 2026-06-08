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
      SELECT id FROM employees WHERE workspace_id = $3 AND is_active = TRUE
  )
GROUP BY COALESCE(lt.name, 'Other')
ORDER BY count DESC

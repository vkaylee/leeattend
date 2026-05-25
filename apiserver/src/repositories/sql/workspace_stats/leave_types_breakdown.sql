SELECT COALESCE(lt.name, 'Other')::text AS leave_type,
       COUNT(*)::bigint AS count
FROM leave_requests lr
LEFT JOIN leave_types lt ON lt.id = lr.leave_type_id
WHERE lr.status IN ('approved', 'pre_approved')
  AND lr.start_date <= $2
  AND lr.end_date >= $1
  AND lr.employee_id IN (
      SELECT id FROM employees WHERE workspace_id = $3 AND is_active = TRUE
  )
GROUP BY COALESCE(lt.name, 'Other')
ORDER BY count DESC

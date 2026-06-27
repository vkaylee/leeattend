SELECT COALESCE(lt.name, 'Other')::text AS leave_type,
       COUNT(*)::bigint AS count
FROM leave_requests lr
LEFT JOIN leave_types lt ON lt.id = lr.leave_type_id
JOIN employments emp ON lr.employment_id = emp.id
WHERE lr.status IN ('approved', 'pre_approved')
  AND lr.start_date <= $2
  AND lr.end_date >= $1
  AND emp.workspace_id = $3
  AND emp.deleted_at IS NULL
  AND emp.status = 'active'
  AND CURRENT_DATE BETWEEN emp.start_date AND COALESCE(emp.end_date, '9999-12-31'::date)
GROUP BY COALESCE(lt.name, 'Other')
ORDER BY count DESC

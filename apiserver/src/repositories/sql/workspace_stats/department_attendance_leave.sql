WITH active_employments AS (
    SELECT emp.id,
           emp.employee_id,
           emp.department_id,
           ROW_NUMBER() OVER (
               PARTITION BY emp.employee_id
               ORDER BY emp.start_date DESC, emp.id DESC
           ) AS employment_rank
    FROM employments emp
    JOIN employees e
      ON e.id = emp.employee_id
     AND e.workspace_id = $1
     AND e.deleted_at IS NULL
    WHERE emp.workspace_id = $1
      AND emp.deleted_at IS NULL
      AND emp.status = 'active'
      AND emp.department_id IS NOT NULL
      AND emp.start_date <= $3
      AND (emp.end_date IS NULL OR emp.end_date >= $2)
)
SELECT ae.department_id,
       COUNT(DISTINCT ae.employee_id)::bigint AS leave_count
FROM leave_requests lr
JOIN active_employments ae
  ON ae.id = lr.employment_id
 AND ae.employment_rank = 1
WHERE lr.status IN ('approved', 'pre_approved')
  AND lr.start_date <= $3
  AND lr.end_date >= $2
GROUP BY ae.department_id

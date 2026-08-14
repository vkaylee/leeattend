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
      AND emp.start_date <= $2
      AND (emp.end_date IS NULL OR emp.end_date >= $2)
      AND ($3::uuid[] IS NULL OR emp.department_id = ANY($3))
)
SELECT d.id AS department_id,
       d.name::text AS department_name,
       COUNT(DISTINCT ae.employee_id)::bigint AS total_employees
FROM departments d
JOIN active_employments ae
  ON ae.department_id = d.id
 AND ae.employment_rank = 1
WHERE d.workspace_id = $1
GROUP BY d.id, d.name

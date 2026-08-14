WITH active_employments AS (
    SELECT emp.id AS employment_id,
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
      AND emp.status = 'active'
      AND emp.deleted_at IS NULL
      AND emp.department_id IS NOT NULL
      AND emp.start_date <= $2
      AND (emp.end_date IS NULL OR emp.end_date >= $2)
      AND ($3::uuid[] IS NULL OR emp.department_id = ANY($3))
)
SELECT employment_id,
       department_id
FROM active_employments
WHERE employment_rank = 1

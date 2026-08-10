WITH active_employments AS (
    SELECT emp.employee_id,
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
      AND emp.start_date <= $2
      AND (emp.end_date IS NULL OR emp.end_date >= $2)
),
latest_components AS (
    SELECT esc.employee_id,
           esc.component_definition_id,
           esc.amount,
           ROW_NUMBER() OVER (
               PARTITION BY esc.employee_id, esc.component_definition_id
               ORDER BY esc.effective_date DESC
           ) AS component_rank
    FROM employee_salary_components esc
    JOIN active_employments ae
      ON ae.employee_id = esc.employee_id
     AND ae.employment_rank = 1
    WHERE esc.effective_date <= $2
)
SELECT COALESCE(SUM(amount), 0)::numeric AS amount
FROM latest_components
WHERE component_rank = 1

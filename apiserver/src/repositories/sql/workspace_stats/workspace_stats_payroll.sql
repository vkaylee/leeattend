WITH active_emps AS (
    SELECT e.id FROM employees e
    JOIN employments emp ON e.id = emp.employee_id
    WHERE e.workspace_id = $1
      AND emp.status = 'active'
      AND emp.deleted_at IS NULL
      AND $2 BETWEEN emp.start_date AND COALESCE(emp.end_date, '9999-12-31'::date)
),
dynamic_payroll AS (
    SELECT esc.employee_id, SUM(esc.amount) as total_amount
    FROM employee_salary_components esc
    INNER JOIN (
        SELECT employee_id, component_definition_id, MAX(effective_date) as max_date
        FROM employee_salary_components
        WHERE effective_date <= $2
        GROUP BY employee_id, component_definition_id
    ) latest ON esc.employee_id = latest.employee_id
             AND esc.component_definition_id = latest.component_definition_id
             AND esc.effective_date = latest.max_date
    WHERE esc.employee_id IN (SELECT id FROM active_emps)
    GROUP BY esc.employee_id
)
SELECT COALESCE(SUM(total_amount), 0)::numeric as amount FROM dynamic_payroll

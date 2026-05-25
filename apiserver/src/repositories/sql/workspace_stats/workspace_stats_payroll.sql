WITH active_emps AS (
    SELECT id FROM employees WHERE workspace_id = $1 AND is_active = TRUE
),
dynamic_payroll AS (
    SELECT esc.employee_id, SUM(esc.amount) as total_amount
    FROM employee_salary_components esc
    INNER JOIN (
        SELECT employee_id, component_definition_id, MAX(effective_date) as max_date
        FROM employee_salary_components
        WHERE effective_date <= CURRENT_DATE
        GROUP BY employee_id, component_definition_id
    ) latest ON esc.employee_id = latest.employee_id
             AND esc.component_definition_id = latest.component_definition_id
             AND esc.effective_date = latest.max_date
    WHERE esc.employee_id IN (SELECT id FROM active_emps)
    GROUP BY esc.employee_id
)
SELECT COALESCE(SUM(total_amount), 0)::numeric as amount FROM dynamic_payroll

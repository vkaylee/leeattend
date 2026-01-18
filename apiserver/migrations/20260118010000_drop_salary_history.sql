-- Migration to drop legacy salary_history table after migrating data
-- 1. Ensure all 'BASE' components are migrated
INSERT INTO salary_component_definitions (workspace_id, name, code, is_base_for_ot, is_taxable)
SELECT DISTINCT w.id, 'Lương cơ bản', 'BASE', TRUE, TRUE
FROM workspaces w
JOIN employees e ON e.workspace_id = w.id
JOIN salary_history sh ON sh.employee_id = e.id
ON CONFLICT (workspace_id, code) DO NOTHING;

-- 2. Migrate remaining history to dynamic system
INSERT INTO employee_salary_components (employee_id, component_definition_id, amount, effective_date, created_at)
SELECT 
    sh.employee_id, 
    scd.id, 
    sh.base_salary, 
    sh.effective_date,
    sh.created_at
FROM salary_history sh
JOIN employees e ON e.id = sh.employee_id
JOIN salary_component_definitions scd ON scd.workspace_id = e.workspace_id AND scd.code = 'BASE'
ON CONFLICT DO NOTHING; -- Avoid duplicates if already migrated

-- 3. Drop legacy table
DROP TABLE IF EXISTS salary_history;

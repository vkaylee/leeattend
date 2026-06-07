-- This file should undo anything in `up.sql`

-- 1. Re-add employer_id column to employees table
ALTER TABLE employees ADD COLUMN employer_id UUID REFERENCES employers(id) ON DELETE SET NULL;
CREATE INDEX idx_employees_employer_id ON employees(employer_id);

-- 2. Populate employees.employer_id from the active contract
UPDATE employees e
SET employer_id = ec.employer_id
FROM (
    SELECT DISTINCT ON (employee_id) employee_id, employer_id
    FROM employee_contracts
    WHERE status = 'active' AND deleted_at IS NULL
    ORDER BY employee_id, start_date DESC
) ec
WHERE e.id = ec.employee_id;

-- 3. Remove index and column from employee_contracts
DROP INDEX IF EXISTS idx_employee_contracts_employer_id;
ALTER TABLE employee_contracts DROP COLUMN employer_id;

-- 1. Add back deleted legacy columns
ALTER TABLE employees ADD COLUMN primary_location_id UUID REFERENCES work_locations(id) ON DELETE SET NULL;

ALTER TABLE employee_contracts ADD COLUMN employee_id UUID REFERENCES employees(id) ON DELETE RESTRICT;
ALTER TABLE employee_contracts ADD COLUMN employer_id UUID REFERENCES employers(id) ON DELETE RESTRICT;

ALTER TABLE leave_balances ADD COLUMN employee_id UUID REFERENCES employees(id) ON DELETE RESTRICT;
ALTER TABLE leave_requests ADD COLUMN employee_id UUID REFERENCES employees(id) ON DELETE RESTRICT;
ALTER TABLE overtime_request_employees ADD COLUMN employee_id UUID REFERENCES employees(id) ON DELETE RESTRICT;
ALTER TABLE payslips ADD COLUMN employee_id UUID REFERENCES employees(id) ON DELETE RESTRICT;
ALTER TABLE employee_assignments ADD COLUMN employee_id UUID REFERENCES employees(id) ON DELETE RESTRICT;

-- 2. Restore data from employments table
-- A. primary_location_id on employees (take the most recent active location)
UPDATE employees e
SET primary_location_id = emp.primary_location_id
FROM employments emp
WHERE e.id = emp.employee_id
  AND emp.status = 'active';

-- Fallback if no active employment found
UPDATE employees e
SET primary_location_id = (SELECT primary_location_id FROM employments WHERE employee_id = e.id ORDER BY start_date DESC LIMIT 1)
WHERE primary_location_id IS NULL;

-- B. employee_id and employer_id on employee_contracts
UPDATE employee_contracts ec
SET employee_id = emp.employee_id,
    employer_id = emp.employer_id
FROM employments emp
WHERE ec.employment_id = emp.id;

-- C. employee_id on business tables
UPDATE leave_balances lb
SET employee_id = emp.employee_id
FROM employments emp
WHERE lb.employment_id = emp.id;

UPDATE leave_requests lr
SET employee_id = emp.employee_id
FROM employments emp
WHERE lr.employment_id = emp.id;

UPDATE overtime_request_employees ore
SET employee_id = emp.employee_id
FROM employments emp
WHERE ore.employment_id = emp.id;

UPDATE payslips p
SET employee_id = emp.employee_id
FROM employments emp
WHERE p.employment_id = emp.id;

UPDATE employee_assignments ea
SET employee_id = emp.employee_id
FROM employments emp
WHERE ea.employment_id = emp.id;

-- 3. Set restored columns to NOT NULL where applicable
ALTER TABLE employee_contracts ALTER COLUMN employee_id SET NOT NULL;
ALTER TABLE employee_contracts ALTER COLUMN employer_id SET NOT NULL;

ALTER TABLE leave_balances ALTER COLUMN employee_id SET NOT NULL;
ALTER TABLE leave_requests ALTER COLUMN employee_id SET NOT NULL;
ALTER TABLE overtime_request_employees ALTER COLUMN employee_id SET NOT NULL;
ALTER TABLE payslips ALTER COLUMN employee_id SET NOT NULL;
ALTER TABLE employee_assignments ALTER COLUMN employee_id SET NOT NULL;

-- Re-create legacy unique constraints
ALTER TABLE leave_balances ADD CONSTRAINT leave_balances_employee_leave_type_id_year_key UNIQUE (employee_id, leave_type_id, year);
ALTER TABLE overtime_request_employees ADD CONSTRAINT overtime_request_employees_overtime_request_id_employee_id_key UNIQUE (overtime_request_id, employee_id);

-- 4. Re-create legacy indexes
CREATE INDEX idx_employees_primary_location ON employees(primary_location_id);
CREATE INDEX idx_employee_contracts_employee ON employee_contracts(employee_id);
CREATE INDEX idx_employee_contracts_employer ON employee_contracts(employer_id);
CREATE INDEX idx_leave_balances_employee ON leave_balances(employee_id);
CREATE INDEX idx_leave_requests_employee ON leave_requests(employee_id);
CREATE INDEX idx_overtime_request_employees_employee ON overtime_request_employees(employee_id);
CREATE INDEX idx_payslips_employee ON payslips(employee_id);
CREATE INDEX idx_employee_assignments_employee ON employee_assignments(employee_id);

-- 5. Drop new indexes
DROP INDEX IF EXISTS idx_employee_contracts_employment_id;
DROP INDEX IF EXISTS idx_leave_balances_employment_id;
DROP INDEX IF EXISTS idx_leave_requests_employment_id;
DROP INDEX IF EXISTS idx_overtime_request_employees_employment_id;
DROP INDEX IF EXISTS idx_payslips_employment_id;
DROP INDEX IF EXISTS idx_employee_assignments_employment_id;

-- 6. Drop employment_id columns from business tables
ALTER TABLE leave_balances DROP CONSTRAINT IF EXISTS leave_balances_employment_id_leave_type_id_year_key;
ALTER TABLE overtime_request_employees DROP CONSTRAINT IF EXISTS overtime_request_employees_overtime_request_id_employment_id_key;

ALTER TABLE employee_contracts DROP COLUMN employment_id;
ALTER TABLE leave_balances DROP COLUMN employment_id;
ALTER TABLE leave_requests DROP COLUMN employment_id;
ALTER TABLE overtime_request_employees DROP COLUMN employment_id;
ALTER TABLE payslips DROP COLUMN employment_id;
ALTER TABLE employee_assignments DROP COLUMN employment_id;

-- 7. Drop employments table
DROP TABLE IF EXISTS employments;

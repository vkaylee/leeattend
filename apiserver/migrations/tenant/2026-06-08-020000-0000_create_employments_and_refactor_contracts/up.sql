-- 1. Create employments table
CREATE TABLE employments (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE RESTRICT,
    employer_id UUID NOT NULL REFERENCES employers(id) ON DELETE RESTRICT,
    primary_location_id UUID REFERENCES work_locations(id) ON DELETE SET NULL,
    department_id UUID REFERENCES departments(id) ON DELETE SET NULL,
    status VARCHAR(50) NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_employments_workspace_id ON employments(workspace_id);
CREATE INDEX idx_employments_employee_id ON employments(employee_id);
CREATE INDEX idx_employments_employer_id ON employments(employer_id);
CREATE INDEX idx_employments_date_range ON employments(start_date, end_date);

-- 2. Backfill initial data into employments table from existing employee_contracts and employees
-- We group contracts of the same employee at the same employer to form a single continuous employment cycle
INSERT INTO employments (id, workspace_id, employee_id, employer_id, primary_location_id, status, start_date, end_date, created_at, updated_at)
SELECT 
    gen_random_uuid() as id,
    ec.workspace_id,
    ec.employee_id,
    ec.employer_id,
    e.primary_location_id,
    CASE WHEN e.is_active = TRUE THEN 'active' ELSE 'terminated' END as status,
    COALESCE(MIN(ec.start_date), e.hire_date) as start_date,
    MAX(ec.end_date) as end_date,
    NOW() as created_at,
    NOW() as updated_at
FROM employees e
LEFT JOIN employee_contracts ec ON ec.employee_id = e.id AND ec.deleted_at IS NULL
WHERE e.deleted_at IS NULL
GROUP BY ec.workspace_id, ec.employee_id, ec.employer_id, e.id, e.primary_location_id, e.is_active, e.hire_date;

-- Fallback for any employee without contract to ensure we don't miss any active employment profiles
INSERT INTO employments (id, workspace_id, employee_id, employer_id, primary_location_id, status, start_date, created_at, updated_at)
SELECT 
    gen_random_uuid() as id,
    e.workspace_id,
    e.id as employee_id,
    (SELECT id FROM employers WHERE workspace_id = e.workspace_id AND deleted_at IS NULL ORDER BY created_at ASC LIMIT 1) as employer_id,
    e.primary_location_id,
    CASE WHEN e.is_active = TRUE THEN 'active' ELSE 'terminated' END as status,
    e.hire_date as start_date,
    NOW() as created_at,
    NOW() as updated_at
FROM employees e
WHERE e.deleted_at IS NULL 
  AND e.id NOT IN (SELECT DISTINCT employee_id FROM employments);

-- 3. Add employment_id to business tables
ALTER TABLE employee_contracts ADD COLUMN employment_id UUID REFERENCES employments(id) ON DELETE RESTRICT;
ALTER TABLE leave_balances ADD COLUMN employment_id UUID REFERENCES employments(id) ON DELETE RESTRICT;
ALTER TABLE leave_requests ADD COLUMN employment_id UUID REFERENCES employments(id) ON DELETE RESTRICT;
ALTER TABLE overtime_request_employees ADD COLUMN employment_id UUID REFERENCES employments(id) ON DELETE RESTRICT;
ALTER TABLE payslips ADD COLUMN employment_id UUID REFERENCES employments(id) ON DELETE RESTRICT;
ALTER TABLE employee_assignments ADD COLUMN employment_id UUID REFERENCES employments(id) ON DELETE RESTRICT;

-- 4. Update employment_id links based on timeline match
-- A. employee_contracts
UPDATE employee_contracts ec
SET employment_id = emp.id
FROM employments emp
WHERE ec.employee_id = emp.employee_id AND ec.employer_id = emp.employer_id;

-- B. leave_balances
UPDATE leave_balances lb
SET employment_id = emp.id
FROM employments emp
WHERE lb.employee_id = emp.employee_id
  AND (
      (emp.start_date <= TO_DATE(lb.year || '-12-31', 'YYYY-MM-DD') AND (emp.end_date IS NULL OR emp.end_date >= TO_DATE(lb.year || '-01-01', 'YYYY-MM-DD')))
      OR emp.id = (
          SELECT id FROM employments 
          WHERE employee_id = lb.employee_id 
          ORDER BY start_date DESC LIMIT 1
      )
  );

-- C. leave_requests
UPDATE leave_requests lr
SET employment_id = emp.id
FROM employments emp
WHERE lr.employee_id = emp.employee_id
  AND (
      (lr.start_date >= emp.start_date AND (emp.end_date IS NULL OR lr.start_date <= emp.end_date))
      OR emp.id = (
          SELECT id FROM employments 
          WHERE employee_id = lr.employee_id 
          ORDER BY start_date DESC LIMIT 1
      )
  );

-- D. overtime_request_employees
UPDATE overtime_request_employees ore
SET employment_id = emp.id
FROM overtime_requests orq, employments emp
WHERE ore.overtime_request_id = orq.id
  AND ore.employee_id = emp.employee_id
  AND (
      (orq.planned_date >= emp.start_date AND (emp.end_date IS NULL OR orq.planned_date <= emp.end_date))
      OR emp.id = (
          SELECT id FROM employments 
          WHERE employee_id = ore.employee_id 
          ORDER BY start_date DESC LIMIT 1
      )
  );

-- E. payslips
UPDATE payslips p
SET employment_id = emp.id
FROM employments emp
WHERE p.employee_id = emp.employee_id
  AND (
      (p.period_start >= emp.start_date AND (emp.end_date IS NULL OR p.period_start <= emp.end_date))
      OR emp.id = (
          SELECT id FROM employments 
          WHERE employee_id = p.employee_id 
          ORDER BY start_date DESC LIMIT 1
      )
  );

-- F. employee_assignments
UPDATE employee_assignments ea
SET employment_id = emp.id
FROM employments emp
WHERE ea.employee_id = emp.employee_id
  AND (
      (ea.start_date >= emp.start_date AND (emp.end_date IS NULL OR ea.start_date <= emp.end_date))
      OR emp.id = (
          SELECT id FROM employments 
          WHERE employee_id = ea.employee_id 
          ORDER BY start_date DESC LIMIT 1
      )
  );

-- Fallback default for any remaining NULL employment_id
UPDATE employee_contracts SET employment_id = (SELECT id FROM employments WHERE employee_id = employee_contracts.employee_id ORDER BY start_date DESC LIMIT 1) WHERE employment_id IS NULL;
UPDATE leave_balances SET employment_id = (SELECT id FROM employments WHERE employee_id = leave_balances.employee_id ORDER BY start_date DESC LIMIT 1) WHERE employment_id IS NULL;
UPDATE leave_requests SET employment_id = (SELECT id FROM employments WHERE employee_id = leave_requests.employee_id ORDER BY start_date DESC LIMIT 1) WHERE employment_id IS NULL;
UPDATE overtime_request_employees SET employment_id = (SELECT id FROM employments WHERE employee_id = overtime_request_employees.employee_id ORDER BY start_date DESC LIMIT 1) WHERE employment_id IS NULL;
UPDATE payslips SET employment_id = (SELECT id FROM employments WHERE employee_id = payslips.employee_id ORDER BY start_date DESC LIMIT 1) WHERE employment_id IS NULL;
UPDATE employee_assignments SET employment_id = (SELECT id FROM employments WHERE employee_id = employee_assignments.employee_id ORDER BY start_date DESC LIMIT 1) WHERE employment_id IS NULL;

-- 5. Set employment_id to NOT NULL
ALTER TABLE employee_contracts ALTER COLUMN employment_id SET NOT NULL;
ALTER TABLE leave_balances ALTER COLUMN employment_id SET NOT NULL;
ALTER TABLE leave_requests ALTER COLUMN employment_id SET NOT NULL;
ALTER TABLE overtime_request_employees ALTER COLUMN employment_id SET NOT NULL;
ALTER TABLE payslips ALTER COLUMN employment_id SET NOT NULL;
ALTER TABLE employee_assignments ALTER COLUMN employment_id SET NOT NULL;

-- 6. Add indexes for new foreign keys
CREATE INDEX idx_employee_contracts_employment_id ON employee_contracts(employment_id);
CREATE INDEX idx_leave_balances_employment_id ON leave_balances(employment_id);
CREATE INDEX idx_leave_requests_employment_id ON leave_requests(employment_id);
CREATE INDEX idx_overtime_request_employees_employment_id ON overtime_request_employees(employment_id);
CREATE INDEX idx_payslips_employment_id ON payslips(employment_id);
CREATE INDEX idx_employee_assignments_employment_id ON employee_assignments(employment_id);

-- Add new unique constraints for employment_id
ALTER TABLE leave_balances ADD CONSTRAINT leave_balances_employment_id_leave_type_id_year_key UNIQUE (employment_id, leave_type_id, year);
ALTER TABLE overtime_request_employees ADD CONSTRAINT overtime_request_employees_overtime_request_id_employment_id_key UNIQUE (overtime_request_id, employment_id);

-- 7. Remove legacy/redundant columns
-- A. primary_location_id from employees (now resides in employments)
ALTER TABLE employees DROP COLUMN primary_location_id;

-- B. employee_id and employer_id from employee_contracts (governed via employment_id)
ALTER TABLE employee_contracts DROP COLUMN employee_id;
ALTER TABLE employee_contracts DROP COLUMN employer_id;

-- C. employee_id from business tables (now linked via employment_id)
ALTER TABLE leave_balances DROP CONSTRAINT IF EXISTS leave_balances_employee_leave_type_id_year_key;
ALTER TABLE leave_balances DROP COLUMN employee_id;
ALTER TABLE leave_requests DROP COLUMN employee_id;
ALTER TABLE overtime_request_employees DROP CONSTRAINT IF EXISTS overtime_request_employees_overtime_request_id_employee_id_key;
ALTER TABLE overtime_request_employees DROP CONSTRAINT IF EXISTS overtime_request_employees_overtime_request_id_fkey;
ALTER TABLE overtime_request_employees DROP COLUMN employee_id;
ALTER TABLE payslips DROP COLUMN employee_id;
ALTER TABLE employee_assignments DROP COLUMN employee_id;

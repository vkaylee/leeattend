-- Add migration script here

-- 1. Add employer_id column as NULL initially to employee_contracts
ALTER TABLE employee_contracts ADD COLUMN employer_id UUID REFERENCES employers(id) ON DELETE RESTRICT;

-- 2. Populate employee_contracts.employer_id using employees.employer_id
UPDATE employee_contracts ec
SET employer_id = e.employer_id
FROM employees e
WHERE ec.employee_id = e.id AND e.employer_id IS NOT NULL;

-- If there are employees with an employer_id but NO contracts, create a default contract for them
INSERT INTO employee_contracts (
    id, workspace_id, employee_id, contract_type, status, start_date, end_date, notes, created_at, updated_at, employer_id
)
SELECT
    gen_random_uuid(),
    e.workspace_id,
    e.id,
    'indefinite'::public.contract_type_enum,
    'active'::public.contract_status_enum,
    e.hire_date,
    NULL,
    'Hệ thống tự động khởi tạo từ dữ liệu pháp nhân cũ',
    NOW(),
    NOW(),
    e.employer_id
FROM employees e
WHERE e.employer_id IS NOT NULL
  AND NOT EXISTS (
      SELECT 1 FROM employee_contracts ec WHERE ec.employee_id = e.id
  );

-- 3. Fallback: For any remaining contracts with NULL employer_id, use the first available employer in the workspace
UPDATE employee_contracts ec
SET employer_id = emp.id
FROM (
    SELECT DISTINCT ON (workspace_id) workspace_id, id
    FROM employers
    WHERE deleted_at IS NULL
    ORDER BY workspace_id, created_at ASC
) emp
WHERE ec.workspace_id = emp.workspace_id AND ec.employer_id IS NULL;

-- 4. Set employer_id as NOT NULL after migration
ALTER TABLE employee_contracts ALTER COLUMN employer_id SET NOT NULL;

-- 5. Create index for employer_id on employee_contracts
CREATE INDEX idx_employee_contracts_employer_id ON employee_contracts(employer_id);

-- 6. Drop employer_id from employees table
ALTER TABLE employees DROP COLUMN employer_id;

-- 1. Clean up and delete dummy contracts from employee_contracts table
DELETE FROM employee_contracts WHERE id IN (SELECT contract_id FROM temp_dummy_contracts);

-- 2. Insert missing employments for soft-deleted employees that were skipped in previously migrated environments (like staging/dev)
-- First, insert for employees who have active contracts
DO $$
BEGIN
    -- Only run if employee_contracts still has employee_id (meaning we are running after the failed migration)
    -- In staging/dev, employee_contracts.employee_id is already dropped, so this is skipped.
    IF EXISTS (
        SELECT 1 
        FROM pg_attribute 
        WHERE attrelid = to_regclass('employee_contracts')
          AND attname = 'employee_id'
          AND NOT attisdropped
    ) THEN
        EXECUTE '
            INSERT INTO employments (id, workspace_id, employee_id, employer_id, primary_location_id, status, start_date, end_date, created_at, updated_at, deleted_at)
            SELECT 
                gen_random_uuid() as id,
                ec.workspace_id,
                ec.employee_id,
                ec.employer_id,
                e.primary_location_id,
                CASE WHEN e.is_active = TRUE THEN ''active'' ELSE ''terminated'' END as status,
                COALESCE(MIN(ec.start_date), e.hire_date, ''2026-01-01''::date) as start_date,
                MAX(ec.end_date) as end_date,
                NOW() as created_at,
                NOW() as updated_at,
                t.deleted_at
            FROM employees e
            JOIN temp_deleted_employees t ON e.id = t.employee_id
            INNER JOIN employee_contracts ec ON ec.employee_id = e.id AND ec.deleted_at IS NULL
            WHERE e.id NOT IN (SELECT DISTINCT employee_id FROM employments)
            GROUP BY ec.workspace_id, ec.employee_id, ec.employer_id, e.id, e.primary_location_id, e.is_active, e.hire_date, t.deleted_at
        ';
    END IF;
END $$;

DO $$
BEGIN
    -- Fallback for missing employments without contracts
    -- We check if primary_location_id column exists on employees (was dropped in step 7.A of 2026-06-08-020000-0000)
    IF EXISTS (
        SELECT 1 
        FROM pg_attribute 
        WHERE attrelid = to_regclass('employees')
          AND attname = 'primary_location_id'
          AND NOT attisdropped
    ) THEN
        EXECUTE '
            INSERT INTO employments (id, workspace_id, employee_id, employer_id, primary_location_id, status, start_date, created_at, updated_at, deleted_at)
            SELECT 
                gen_random_uuid() as id,
                e.workspace_id,
                e.id as employee_id,
                (SELECT id FROM employers WHERE workspace_id = e.workspace_id ORDER BY (deleted_at IS NULL) DESC, created_at ASC LIMIT 1) as employer_id,
                e.primary_location_id,
                CASE WHEN e.is_active = TRUE THEN ''active'' ELSE ''terminated'' END as status,
                COALESCE(e.hire_date, ''2026-01-01''::date) as start_date,
                NOW() as created_at,
                NOW() as updated_at,
                t.deleted_at
            FROM employees e
            JOIN temp_deleted_employees t ON e.id = t.employee_id
            WHERE e.id NOT IN (SELECT DISTINCT employee_id FROM employments)
        ';
    ELSE
        EXECUTE '
            INSERT INTO employments (id, workspace_id, employee_id, employer_id, primary_location_id, status, start_date, created_at, updated_at, deleted_at)
            SELECT 
                gen_random_uuid() as id,
                e.workspace_id,
                e.id as employee_id,
                (SELECT id FROM employers WHERE workspace_id = e.workspace_id ORDER BY (deleted_at IS NULL) DESC, created_at ASC LIMIT 1) as employer_id,
                NULL as primary_location_id,
                CASE WHEN e.is_active = TRUE THEN ''active'' ELSE ''terminated'' END as status,
                COALESCE(e.hire_date, ''2026-01-01''::date) as start_date,
                NOW() as created_at,
                NOW() as updated_at,
                t.deleted_at
            FROM employees e
            JOIN temp_deleted_employees t ON e.id = t.employee_id
            WHERE e.id NOT IN (SELECT DISTINCT employee_id FROM employments)
        ';
    END IF;
END $$;

DO $$
BEGIN
    -- 3. Link legacy NULL employment_id in business tables if the legacy columns still exist
    -- A. employee_contracts
    IF EXISTS (
        SELECT 1 
        FROM pg_attribute 
        WHERE attrelid = to_regclass('employee_contracts')
          AND attname = 'employee_id'
          AND NOT attisdropped
    ) THEN
        EXECUTE 'UPDATE employee_contracts SET employment_id = (SELECT id FROM employments WHERE employee_id = employee_contracts.employee_id ORDER BY start_date DESC LIMIT 1) WHERE employment_id IS NULL';
    END IF;

    -- B. leave_balances
    IF EXISTS (
        SELECT 1 
        FROM pg_attribute 
        WHERE attrelid = to_regclass('leave_balances')
          AND attname = 'employee_id'
          AND NOT attisdropped
    ) THEN
        EXECUTE 'UPDATE leave_balances SET employment_id = (SELECT id FROM employments WHERE employee_id = leave_balances.employee_id ORDER BY start_date DESC LIMIT 1) WHERE employment_id IS NULL';
    END IF;

    -- C. leave_requests
    IF EXISTS (
        SELECT 1 
        FROM pg_attribute 
        WHERE attrelid = to_regclass('leave_requests')
          AND attname = 'employee_id'
          AND NOT attisdropped
    ) THEN
        EXECUTE 'UPDATE leave_requests SET employment_id = (SELECT id FROM employments WHERE employee_id = leave_requests.employee_id ORDER BY start_date DESC LIMIT 1) WHERE employment_id IS NULL';
    END IF;

    -- D. overtime_request_employees
    IF EXISTS (
        SELECT 1 
        FROM pg_attribute 
        WHERE attrelid = to_regclass('overtime_request_employees')
          AND attname = 'employee_id'
          AND NOT attisdropped
    ) THEN
        EXECUTE 'UPDATE overtime_request_employees SET employment_id = (SELECT id FROM employments WHERE employee_id = overtime_request_employees.employee_id ORDER BY start_date DESC LIMIT 1) WHERE employment_id IS NULL';
    END IF;

    -- E. payslips
    IF EXISTS (
        SELECT 1 
        FROM pg_attribute 
        WHERE attrelid = to_regclass('payslips')
          AND attname = 'employee_id'
          AND NOT attisdropped
    ) THEN
        EXECUTE 'UPDATE payslips SET employment_id = (SELECT id FROM employments WHERE employee_id = payslips.employee_id ORDER BY start_date DESC LIMIT 1) WHERE employment_id IS NULL';
    END IF;

    -- F. employee_assignments
    IF EXISTS (
        SELECT 1 
        FROM pg_attribute 
        WHERE attrelid = to_regclass('employee_assignments')
          AND attname = 'employee_id'
          AND NOT attisdropped
    ) THEN
        EXECUTE 'UPDATE employee_assignments SET employment_id = (SELECT id FROM employments WHERE employee_id = employee_assignments.employee_id ORDER BY start_date DESC LIMIT 1) WHERE employment_id IS NULL';
    END IF;
END $$;

-- 4. Restore the deleted_at timestamps to employees and propagate them to employments
UPDATE employees e
SET deleted_at = t.deleted_at
FROM temp_deleted_employees t
WHERE e.id = t.employee_id;

UPDATE employments em
SET deleted_at = t.deleted_at
FROM temp_deleted_employees t
WHERE em.employee_id = t.employee_id;

-- 5. Clean up temp tables
DROP TABLE IF EXISTS temp_dummy_contracts;
DROP TABLE IF EXISTS temp_deleted_employees;

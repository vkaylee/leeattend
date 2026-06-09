-- 1. Create table to track temporary deleted employees' timestamps
CREATE TABLE IF NOT EXISTS temp_deleted_employees (
    employee_id UUID PRIMARY KEY,
    deleted_at TIMESTAMPTZ NOT NULL
);

INSERT INTO temp_deleted_employees (employee_id, deleted_at)
SELECT id, deleted_at FROM employees WHERE deleted_at IS NOT NULL
ON CONFLICT (employee_id) DO NOTHING;

-- Make all employees temporarily active to bypass WHERE e.deleted_at IS NULL filters
UPDATE employees SET deleted_at = NULL WHERE deleted_at IS NOT NULL;

-- 2. Create table to track temporary dummy contracts
CREATE TABLE IF NOT EXISTS temp_dummy_contracts (
    contract_id UUID PRIMARY KEY,
    employee_id UUID NOT NULL
);

-- 3. Create dummy contracts for any employee who has no active contracts
-- Use to_regclass to check for table/column existence, bypassing current_schema() mismatch on production schemas
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 
        FROM pg_attribute 
        WHERE attrelid = to_regclass('employee_contracts')
          AND attname = 'employee_id'
          AND NOT attisdropped
    ) THEN
        EXECUTE '
            INSERT INTO employee_contracts (
                id, workspace_id, employee_id, contract_type, status, start_date, notes, created_at, updated_at, employer_id
            )
            SELECT
                gen_random_uuid() as id,
                e.workspace_id,
                e.id as employee_id,
                ''indefinite''::public.contract_type_enum as contract_type,
                ''active''::public.contract_status_enum as status,
                COALESCE(e.hire_date, ''2026-01-01''::date) as start_date,
                ''Hệ thống tự động tạo tạm thời để di trú dữ liệu'' as notes,
                NOW() as created_at,
                NOW() as updated_at,
                (SELECT id FROM employers WHERE workspace_id = e.workspace_id ORDER BY (deleted_at IS NULL) DESC, created_at ASC LIMIT 1) as employer_id
            FROM employees e
            WHERE NOT EXISTS (
                SELECT 1 FROM employee_contracts ec 
                WHERE ec.employee_id = e.id AND ec.deleted_at IS NULL
            )
        ';

        EXECUTE '
            INSERT INTO temp_dummy_contracts (contract_id, employee_id)
            SELECT id, employee_id
            FROM employee_contracts
            WHERE notes = ''Hệ thống tự động tạo tạm thời để di trú dữ liệu''
            ON CONFLICT (contract_id) DO NOTHING
        ';
    END IF;
END $$;

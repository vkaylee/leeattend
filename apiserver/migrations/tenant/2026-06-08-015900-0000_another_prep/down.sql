-- Delete dummy contracts if employee_id column exists
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 
        FROM pg_attribute 
        WHERE attrelid = to_regclass('employee_contracts')
          AND attname = 'employee_id'
          AND NOT attisdropped
    ) THEN
        DELETE FROM employee_contracts WHERE id IN (SELECT contract_id FROM temp_dummy_contracts);
    END IF;
END $$;

-- Restore deleted_at for employees
UPDATE employees e
SET deleted_at = t.deleted_at
FROM temp_deleted_employees t
WHERE e.id = t.employee_id;

DROP TABLE IF EXISTS temp_dummy_contracts;
DROP TABLE IF EXISTS temp_deleted_employees;

-- Up Migration: Normalize existing employee IDs
-- 1. Trim whitespace
-- 2. Remove leading zeros only if the ID is purely numeric

UPDATE employees 
SET employee_id_number = 
    CASE 
        WHEN employee_id_number ~ '^[0-9]+$' THEN 
            LTRIM(TRIM(employee_id_number), '0')
        ELSE 
            TRIM(employee_id_number)
    END;

-- Ensure "000" doesn't become empty string (LTRIM of '000' is empty)
UPDATE employees 
SET employee_id_number = '0' 
WHERE employee_id_number = '';

-- Down Migration (Optional: Hard to revert normalization, but we can trim)
-- UPDATE employees SET employee_id_number = TRIM(employee_id_number);

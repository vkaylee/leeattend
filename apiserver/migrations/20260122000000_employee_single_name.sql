-- Add full_name column
ALTER TABLE employees ADD COLUMN full_name VARCHAR(255);

-- Migrate existing data
UPDATE employees SET full_name = TRIM(first_name || ' ' || last_name);

-- Handle cases where both were empty (should not happen due to NOT NULL constraints, but being safe)
UPDATE employees SET full_name = 'Unknown' WHERE full_name IS NULL OR full_name = '';

-- Make full_name NOT NULL
ALTER TABLE employees ALTER COLUMN full_name SET NOT NULL;

-- Drop old columns
ALTER TABLE employees DROP COLUMN first_name;
ALTER TABLE employees DROP COLUMN last_name;

-- Update constraints if they exist (Checking for existing constraint from search results)
-- FROM search: L41: ALTER TABLE employees ADD CONSTRAINT chk_employee_names CHECK (length(first_name) > 0 AND length(last_name) > 0);
ALTER TABLE employees DROP CONSTRAINT IF EXISTS chk_employee_names;
ALTER TABLE employees ADD CONSTRAINT chk_employee_full_name CHECK (length(full_name) > 0);

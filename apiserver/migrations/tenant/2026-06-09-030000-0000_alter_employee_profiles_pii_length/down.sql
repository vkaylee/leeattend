-- Revert national_id and bank_account_number column types back to VARCHAR(50)
ALTER TABLE employee_profiles ALTER COLUMN national_id TYPE VARCHAR(50);
ALTER TABLE employee_profiles ALTER COLUMN bank_account_number TYPE VARCHAR(50);

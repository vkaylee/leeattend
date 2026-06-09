-- Alter national_id and bank_account_number column types to TEXT to support AEAD encryption values
ALTER TABLE employee_profiles ALTER COLUMN national_id TYPE TEXT;
ALTER TABLE employee_profiles ALTER COLUMN bank_account_number TYPE TEXT;

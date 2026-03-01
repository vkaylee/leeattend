-- Down Migration: Remove Trigram Index for employee_id_number

DROP INDEX IF EXISTS idx_employees_id_number_search;

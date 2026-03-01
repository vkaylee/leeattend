-- Up Migration: Add Trigram Index for employee_id_number

-- Enable pg_trgm extension if not already enabled (should be by previous migration, but safe to repeat)
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Add Trigram Index for Employee ID Number Search
-- This allows fast '%term%' searching on employee IDs, avoiding full table scans
CREATE INDEX IF NOT EXISTS idx_employees_id_number_search ON employees USING GIN (employee_id_number gin_trgm_ops);

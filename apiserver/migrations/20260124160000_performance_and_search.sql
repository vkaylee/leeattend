-- Up Migration: Performance Indexes and Search Optimization

-- 1. Enable pg_trgm extension for fuzzy text search
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- 2. Add Trigram Index for Employee Name Search
-- This allows fast '%term%' searching on names
CREATE INDEX IF NOT EXISTS idx_employees_name_search ON employees USING GIN (full_name gin_trgm_ops);

-- 3. Add Missing Foreign Key Indexes
CREATE INDEX IF NOT EXISTS idx_employee_shifts_shift_id ON employee_shifts(shift_id);
CREATE INDEX IF NOT EXISTS idx_leave_requests_requested_by ON leave_requests(requested_by_user_id);

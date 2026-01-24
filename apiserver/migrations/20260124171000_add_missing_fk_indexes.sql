-- Up Migration: Add missing FK indexes for query performance

-- 1. Index on leave_requests.approved_by_user_id
-- Speeds up JOINs when looking up approver info and CASCADE deletes on users
CREATE INDEX IF NOT EXISTS idx_leave_requests_approved_by ON leave_requests(approved_by_user_id);

-- 2. Index on working_pattern_details.shift_id
-- Speeds up JOINs when resolving shift assignments from patterns
CREATE INDEX IF NOT EXISTS idx_working_pattern_details_shift ON working_pattern_details(shift_id);

-- Down Migration (commented):
-- DROP INDEX IF EXISTS idx_working_pattern_details_shift;
-- DROP INDEX IF EXISTS idx_leave_requests_approved_by;

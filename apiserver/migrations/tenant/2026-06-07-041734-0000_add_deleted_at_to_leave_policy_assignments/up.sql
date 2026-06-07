-- Add deleted_at column to leave_policy_assignments for soft deletes
ALTER TABLE leave_policy_assignments ADD COLUMN deleted_at TIMESTAMP WITH TIME ZONE;

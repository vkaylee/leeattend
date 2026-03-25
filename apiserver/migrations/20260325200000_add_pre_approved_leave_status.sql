-- Add pre_approved variant to leave_status_type enum
-- Used when TeamLead submits leave request for their department employees (auto-approve)
ALTER TYPE leave_status_type ADD VALUE IF NOT EXISTS 'pre_approved';

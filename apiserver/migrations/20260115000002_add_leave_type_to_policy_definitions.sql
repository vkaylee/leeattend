-- Add leave_type to leave_policy_definitions
ALTER TABLE leave_policy_definitions ADD COLUMN leave_type VARCHAR(50) NOT NULL DEFAULT 'vacation';

-- Update the existing records if any (though likely empty or test data)
UPDATE leave_policy_definitions SET leave_type = 'vacation' WHERE leave_type IS NULL;
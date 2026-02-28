-- Holiday-Leave Deduction: link unpaid holidays to leave type for auto-deduction
ALTER TABLE holidays ADD COLUMN leave_type_id UUID REFERENCES leave_types(id) ON DELETE SET NULL;
-- Only used when is_paid = false: specifies which leave type to auto-deduct

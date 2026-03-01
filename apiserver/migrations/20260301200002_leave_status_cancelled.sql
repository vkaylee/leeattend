-- Add 'cancelled' to leave_status_type enum for holiday leave reversal
ALTER TYPE leave_status_type ADD VALUE IF NOT EXISTS 'cancelled';

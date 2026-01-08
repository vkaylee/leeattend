-- Up Migration: Create attendance_adjustment_requests table

CREATE TABLE IF NOT EXISTS attendance_adjustment_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    time_entry_id UUID REFERENCES time_entries(id) ON DELETE SET NULL, -- Null if adding a new missing entry
    requested_punch_in_time TIMESTAMPTZ NOT NULL,
    requested_punch_out_time TIMESTAMPTZ,
    reason TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- 'pending', 'approved', 'rejected'
    requested_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    approved_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_attendance_adjustment_requests_employee_id ON attendance_adjustment_requests(employee_id);
CREATE INDEX IF NOT EXISTS idx_attendance_adjustment_requests_status ON attendance_adjustment_requests(status);

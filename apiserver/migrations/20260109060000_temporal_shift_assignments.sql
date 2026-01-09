-- Up Migration: Temporal Shift Assignments

-- 1. Modify employee_shifts table to support end_date and priority by creation time
-- We need to handle existing data if any. 
-- For safety, we'll recreate the table or modify it.

-- First, let's see if we can just alter it.
-- The current PK is (employee_id, shift_id, effective_date). 
-- We want a single UUID PK and add end_date.

CREATE TABLE IF NOT EXISTS employee_shifts_new (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    shift_id UUID NOT NULL REFERENCES shifts(id) ON DELETE CASCADE,
    start_date DATE NOT NULL,
    end_date DATE, -- NULL means forever
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Copy data from old table if it exists
INSERT INTO employee_shifts_new (employee_id, shift_id, start_date)
SELECT employee_id, shift_id, effective_date FROM employee_shifts;

-- Drop old table and rename new one
DROP TABLE employee_shifts;
ALTER TABLE employee_shifts_new RENAME TO employee_shifts;

CREATE INDEX IF NOT EXISTS idx_employee_shifts_employee_id ON employee_shifts(employee_id);
CREATE INDEX IF NOT EXISTS idx_employee_shifts_dates ON employee_shifts(start_date, end_date);

-- Up Migration: Shift Rostering and Default Assignments

-- 1. Create shift_rosters table (Date-specific assignment)
CREATE TABLE IF NOT EXISTS shift_rosters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    shift_id UUID REFERENCES shifts(id) ON DELETE SET NULL, -- NULL means day off
    date DATE NOT NULL,
    status VARCHAR(20) DEFAULT 'confirmed', -- 'draft', 'confirmed'
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(employee_id, date) -- One employee can only have one roster entry per day
);

CREATE INDEX IF NOT EXISTS idx_shift_rosters_employee_date ON shift_rosters(employee_id, date);

-- 2. Create department_default_shifts table
CREATE TABLE IF NOT EXISTS department_default_shifts (
    department_id UUID NOT NULL REFERENCES departments(id) ON DELETE CASCADE,
    shift_id UUID NOT NULL REFERENCES shifts(id) ON DELETE CASCADE,
    PRIMARY KEY (department_id, shift_id)
);

-- 3. Modify employee_shifts to act as Employee Default Shifts (already exists but we'll use it as fallback)
-- We'll rename it or use it as is for backward compatibility. 
-- In our system: Roster > Employee Default (employee_shifts) > Department Default.

-- 4. Seed some sample rosters for dev
-- (Will be done via API)

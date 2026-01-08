-- Up Migration: Create timekeeping related tables (employees, time_entries, leave_requests)

-- Create employees table
CREATE TABLE IF NOT EXISTS employees (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL, -- Nullable if not all employees have direct user accounts
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    employee_id_number VARCHAR(50) UNIQUE NOT NULL, -- Unique identifier for payroll etc.
    hire_date DATE NOT NULL,
    termination_date DATE, -- Nullable
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_employees_workspace_id ON employees(workspace_id);
CREATE INDEX IF NOT EXISTS idx_employees_user_id ON employees(user_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_employees_employee_id_number ON employees(employee_id_number);

-- Create time_entries table
CREATE TABLE IF NOT EXISTS time_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    punch_in_time TIMESTAMPTZ NOT NULL,
    punch_out_time TIMESTAMPTZ, -- Nullable until punch_out
    duration_minutes INTEGER, -- Calculated, nullable
    entry_type VARCHAR(50) NOT NULL DEFAULT 'work', -- e.g., 'work', 'break', 'training'
    note TEXT, -- Nullable
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_time_entries_employee_id ON time_entries(employee_id);
CREATE INDEX IF NOT EXISTS idx_time_entries_punch_in_time ON time_entries(punch_in_time);

-- Create leave_requests table
CREATE TABLE IF NOT EXISTS leave_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    leave_type VARCHAR(50) NOT NULL, -- e.g., 'vacation', 'sick', 'personal'
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    reason TEXT, -- Nullable
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- e.g., 'pending', 'approved', 'rejected'
    requested_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL, -- User who submitted the request (can be employee's user or manager's user)
    approved_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL, -- User who approved the request (manager/admin)
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_start_before_end CHECK (start_date <= end_date)
);

CREATE INDEX IF NOT EXISTS idx_leave_requests_employee_id ON leave_requests(employee_id);
CREATE INDEX IF NOT EXISTS idx_leave_requests_status ON leave_requests(status);

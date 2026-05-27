CREATE TYPE assignment_status AS ENUM ('active', 'completed', 'cancelled');

CREATE TABLE employee_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    employee_id UUID NOT NULL REFERENCES employees(id),
    home_location_id UUID NOT NULL REFERENCES work_locations(id),
    host_location_id UUID NOT NULL REFERENCES work_locations(id),
    start_date DATE NOT NULL,
    end_date DATE,
    reason TEXT,
    status assignment_status NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Index for fast lookup of active assignments by date
CREATE INDEX idx_employee_assignments_date 
    ON employee_assignments(employee_id, start_date, end_date) 
    WHERE status = 'active';

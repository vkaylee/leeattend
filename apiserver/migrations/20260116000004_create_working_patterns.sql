-- Working Patterns
CREATE TABLE working_patterns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Working Pattern Details (Shifts for each day of the week)
CREATE TABLE working_pattern_details (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pattern_id UUID NOT NULL REFERENCES working_patterns(id) ON DELETE CASCADE,
    day_of_week INT NOT NULL CHECK (day_of_week BETWEEN 0 AND 6), -- 0=Sunday, 1=Monday, ..., 6=Saturday
    shift_id UUID REFERENCES shifts(id) ON DELETE SET NULL, -- NULL means Day Off
    UNIQUE (pattern_id, day_of_week)
);

-- Employee Working Pattern Assignments
CREATE TABLE employee_working_patterns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    pattern_id UUID NOT NULL REFERENCES working_patterns(id) ON DELETE CASCADE,
    start_date DATE NOT NULL,
    end_date DATE, -- NULL means until changed
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (end_date IS NULL OR end_date >= start_date)
);

CREATE INDEX idx_employee_working_patterns_employee_date ON employee_working_patterns (employee_id, start_date);

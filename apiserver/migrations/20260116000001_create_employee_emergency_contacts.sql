-- Employee Self-Service: Emergency contacts table
-- Stores emergency contact information for employees

CREATE TABLE employee_emergency_contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    contact_name VARCHAR(100) NOT NULL,
    relationship VARCHAR(50) NOT NULL,
    phone VARCHAR(20) NOT NULL,
    is_primary BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for faster lookups
CREATE INDEX idx_employee_emergency_contacts_employee_id ON employee_emergency_contacts(employee_id);

-- Ensure only one primary contact per employee
CREATE UNIQUE INDEX idx_employee_emergency_contacts_primary
    ON employee_emergency_contacts(employee_id)
    WHERE is_primary = true;

-- Up Migration: Create leave_balances table

CREATE TABLE IF NOT EXISTS leave_balances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    leave_type VARCHAR(50) NOT NULL, -- e.g., 'vacation', 'sick', 'personal'
    year INTEGER NOT NULL,
    entitlement INTEGER NOT NULL DEFAULT 12, -- Total days allowed per year
    carried_over INTEGER NOT NULL DEFAULT 0, -- Days carried over from previous year
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(employee_id, leave_type, year)
);

-- Seed some default balances for existing employees (e.g., for 2026)
INSERT INTO leave_balances (employee_id, leave_type, year, entitlement)
SELECT id, 'vacation', 2026, 12 FROM employees
ON CONFLICT DO NOTHING;

INSERT INTO leave_balances (employee_id, leave_type, year, entitlement)
SELECT id, 'sick', 2026, 5 FROM employees
ON CONFLICT DO NOTHING;

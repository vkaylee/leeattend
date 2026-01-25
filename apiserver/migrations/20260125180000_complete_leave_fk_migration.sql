-- Up Migration: Complete Leave Management FK Migration
-- This migration finishes the FK migration started in 20260124170000_create_leave_types.sql

-- 1. Add leave_type_id to leave_balances (currently missing)
ALTER TABLE leave_balances
ADD COLUMN IF NOT EXISTS leave_type_id UUID REFERENCES leave_types(id) ON DELETE RESTRICT;

-- 2. Backfill leave_balances.leave_type_id from leave_type string
-- Join via employees to get workspace_id for proper FK lookup
UPDATE leave_balances
SET leave_type_id = lt.id
FROM leave_types lt, employees e
WHERE e.id = leave_balances.employee_id
  AND lt.workspace_id = e.workspace_id
  AND lt.code = leave_balances.leave_type
  AND leave_balances.leave_type_id IS NULL;

-- 3. Handle orphan records in leave_balances (fallback to 'vacation' type)
UPDATE leave_balances
SET leave_type_id = lt.id
FROM leave_types lt, employees e
WHERE e.id = leave_balances.employee_id
  AND lt.workspace_id = e.workspace_id
  AND lt.code = 'vacation'
  AND leave_balances.leave_type_id IS NULL;

-- 4. Backfill leave_requests.leave_type_id (ensure all populated)
UPDATE leave_requests
SET leave_type_id = lt.id
FROM leave_types lt, employees e
WHERE e.id = leave_requests.employee_id
  AND lt.workspace_id = e.workspace_id
  AND lt.code = leave_requests.leave_type
  AND leave_requests.leave_type_id IS NULL;

-- 5. Handle orphan records in leave_requests (fallback to 'vacation')
UPDATE leave_requests
SET leave_type_id = lt.id
FROM leave_types lt, employees e
WHERE e.id = leave_requests.employee_id
  AND lt.workspace_id = e.workspace_id
  AND lt.code = 'vacation'
  AND leave_requests.leave_type_id IS NULL;

-- 6. Backfill leave_accrual_logs.leave_type_id
UPDATE leave_accrual_logs
SET leave_type_id = lt.id
FROM leave_types lt
WHERE lt.workspace_id = leave_accrual_logs.workspace_id
  AND lt.code = leave_accrual_logs.leave_type
  AND leave_accrual_logs.leave_type_id IS NULL;

-- 7. Handle orphan records in leave_accrual_logs (fallback to 'vacation')
UPDATE leave_accrual_logs
SET leave_type_id = lt.id
FROM leave_types lt
WHERE lt.workspace_id = leave_accrual_logs.workspace_id
  AND lt.code = 'vacation'
  AND leave_accrual_logs.leave_type_id IS NULL;

-- 8. Create index for new FK on leave_balances
CREATE INDEX IF NOT EXISTS idx_leave_balances_leave_type_id ON leave_balances(leave_type_id);

-- 9. Make FK columns NOT NULL (data must be backfilled first)
ALTER TABLE leave_balances ALTER COLUMN leave_type_id SET NOT NULL;
ALTER TABLE leave_requests ALTER COLUMN leave_type_id SET NOT NULL;
ALTER TABLE leave_accrual_logs ALTER COLUMN leave_type_id SET NOT NULL;

-- 10. Update unique constraint on leave_balances
-- Old: (employee_id, leave_type, year) -> New: (employee_id, leave_type_id, year)
ALTER TABLE leave_balances DROP CONSTRAINT IF EXISTS leave_balances_employee_id_leave_type_year_key;
ALTER TABLE leave_balances ADD CONSTRAINT leave_balances_employee_leave_type_id_year_key
    UNIQUE (employee_id, leave_type_id, year);

-- 11. Update unique constraint on leave_accrual_logs
ALTER TABLE leave_accrual_logs DROP CONSTRAINT IF EXISTS leave_accrual_logs_employee_id_leave_type_accrual_date_key;
ALTER TABLE leave_accrual_logs ADD CONSTRAINT leave_accrual_logs_employee_leave_type_id_date_key
    UNIQUE (employee_id, leave_type_id, accrual_date);

-- 12. Drop legacy string columns
ALTER TABLE leave_balances DROP COLUMN IF EXISTS leave_type;
ALTER TABLE leave_requests DROP COLUMN IF EXISTS leave_type;
ALTER TABLE leave_accrual_logs DROP COLUMN IF EXISTS leave_type;
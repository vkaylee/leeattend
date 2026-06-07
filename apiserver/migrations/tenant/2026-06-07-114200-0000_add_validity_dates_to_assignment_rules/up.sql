-- 1. Add validity dates to assignment rules
ALTER TABLE assignment_rules ADD COLUMN start_date DATE;
ALTER TABLE assignment_rules ADD COLUMN end_date DATE;

-- 2. Drop department fallback default patterns and shifts
ALTER TABLE departments DROP COLUMN IF EXISTS default_working_pattern_id CASCADE;
DROP TABLE IF EXISTS department_default_shifts CASCADE;

-- 3. Drop resolve_employee_shift function completely
DROP FUNCTION IF EXISTS resolve_employee_shift(UUID, DATE);

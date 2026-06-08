-- 1. Re-add legacy columns to employees
ALTER TABLE employees ADD COLUMN hire_date DATE;
ALTER TABLE employees ADD COLUMN termination_date DATE;
ALTER TABLE employees ADD COLUMN original_hire_date DATE;
ALTER TABLE employees ADD COLUMN is_active BOOLEAN;

-- Populate legacy columns in employees
UPDATE employees e
SET 
    hire_date = COALESCE((SELECT MIN(start_date) FROM employments WHERE employee_id = e.id), '2026-01-01'),
    original_hire_date = COALESCE((SELECT MIN(start_date) FROM employments WHERE employee_id = e.id), '2026-01-01'),
    termination_date = (SELECT MAX(end_date) FROM employments WHERE employee_id = e.id),
    is_active = COALESCE((SELECT EXISTS (SELECT 1 FROM employments WHERE employee_id = e.id AND status = 'active')), false);

ALTER TABLE employees ALTER COLUMN hire_date SET NOT NULL;
ALTER TABLE employees ALTER COLUMN original_hire_date SET NOT NULL;
ALTER TABLE employees ALTER COLUMN is_active SET NOT NULL;

-- 2. Restore employee_id to working_conditions
ALTER TABLE working_conditions ADD COLUMN employee_id UUID REFERENCES employees(id) ON DELETE RESTRICT;
UPDATE working_conditions wc
SET employee_id = emp.employee_id
FROM employments emp
WHERE wc.employment_id = emp.id;
ALTER TABLE working_conditions ALTER COLUMN employee_id SET NOT NULL;
ALTER TABLE working_conditions DROP COLUMN employment_id;

-- 3. Restore employee_id to attendance_adjustment_requests
ALTER TABLE attendance_adjustment_requests ADD COLUMN employee_id UUID REFERENCES employees(id) ON DELETE RESTRICT;
UPDATE attendance_adjustment_requests aar
SET employee_id = emp.employee_id
FROM employments emp
WHERE aar.employment_id = emp.id;
ALTER TABLE attendance_adjustment_requests ALTER COLUMN employee_id SET NOT NULL;
ALTER TABLE attendance_adjustment_requests DROP COLUMN employment_id;

-- 4. Restore employee_id to attendance_daily_summaries
ALTER TABLE attendance_daily_summaries ADD COLUMN employee_id UUID REFERENCES employees(id) ON DELETE RESTRICT;
UPDATE attendance_daily_summaries ads
SET employee_id = emp.employee_id
FROM employments emp
WHERE ads.employment_id = emp.id;
ALTER TABLE attendance_daily_summaries ALTER COLUMN employee_id SET NOT NULL;
ALTER TABLE attendance_daily_summaries DROP COLUMN employment_id;

-- 5. Restore employee_id to shift_rosters
ALTER TABLE shift_rosters ADD COLUMN employee_id UUID REFERENCES employees(id) ON DELETE RESTRICT;
UPDATE shift_rosters sr
SET employee_id = emp.employee_id
FROM employments emp
WHERE sr.employment_id = emp.id;
ALTER TABLE shift_rosters ALTER COLUMN employee_id SET NOT NULL;
ALTER TABLE shift_rosters DROP COLUMN employment_id;

-- 6. Restore employee_id to time_entries
ALTER TABLE time_entries ADD COLUMN employee_id UUID REFERENCES employees(id) ON DELETE RESTRICT;
UPDATE time_entries te
SET employee_id = emp.employee_id
FROM employments emp
WHERE te.employment_id = emp.id;
ALTER TABLE time_entries ALTER COLUMN employee_id SET NOT NULL;
ALTER TABLE time_entries DROP COLUMN employment_id;

-- 7. Restore employee_id to raw_attendance_logs
ALTER TABLE raw_attendance_logs ADD COLUMN employee_id UUID REFERENCES employees(id) ON DELETE RESTRICT;
UPDATE raw_attendance_logs ral
SET employee_id = emp.employee_id
FROM employments emp
WHERE ral.employment_id = emp.id;
ALTER TABLE raw_attendance_logs ALTER COLUMN employee_id SET NOT NULL;
ALTER TABLE raw_attendance_logs DROP COLUMN employment_id;

-- 8. Remove job details from employments
ALTER TABLE employments DROP COLUMN job_title;
ALTER TABLE employments DROP COLUMN employment_type;
ALTER TABLE employments DROP COLUMN manager_employment_id;

-- Recreate unique constraints replacing employment_id with employee_id
ALTER TABLE raw_attendance_logs ADD CONSTRAINT unique_raw_log UNIQUE (workspace_id, employee_id, punch_time, punch, status);
ALTER TABLE shift_rosters ADD CONSTRAINT shift_rosters_employee_id_date_key UNIQUE (employee_id, date);
ALTER TABLE attendance_daily_summaries ADD CONSTRAINT attendance_daily_summaries_employee_id_date_key UNIQUE (employee_id, date);

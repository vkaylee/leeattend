-- 1. Add job details to employments table
ALTER TABLE employments ADD COLUMN job_title VARCHAR(100);
ALTER TABLE employments ADD COLUMN employment_type VARCHAR(50) NOT NULL DEFAULT 'full_time';
ALTER TABLE employments ADD COLUMN manager_employment_id UUID REFERENCES employments(id) ON DELETE SET NULL;

-- 2. Alter raw_attendance_logs
ALTER TABLE raw_attendance_logs ADD COLUMN employment_id UUID REFERENCES employments(id) ON DELETE RESTRICT;
UPDATE raw_attendance_logs ral
SET employment_id = emp.id
FROM employments emp
WHERE ral.employee_id = emp.employee_id
  AND (
      (ral.punch_time >= emp.start_date AND (emp.end_date IS NULL OR ral.punch_time <= emp.end_date))
      OR emp.id = (
          SELECT id FROM employments
          WHERE employee_id = ral.employee_id
          ORDER BY start_date DESC LIMIT 1
      )
  );
-- Fallback in case of unmatched employments
UPDATE raw_attendance_logs SET employment_id = (SELECT id FROM employments WHERE employee_id = raw_attendance_logs.employee_id ORDER BY start_date DESC LIMIT 1) WHERE employment_id IS NULL;
ALTER TABLE raw_attendance_logs ALTER COLUMN employment_id SET NOT NULL;
ALTER TABLE raw_attendance_logs DROP COLUMN employee_id;
CREATE INDEX idx_raw_attendance_logs_employment_id ON raw_attendance_logs(employment_id);

-- 3. Alter time_entries
ALTER TABLE time_entries ADD COLUMN employment_id UUID REFERENCES employments(id) ON DELETE RESTRICT;
UPDATE time_entries te
SET employment_id = emp.id
FROM employments emp
WHERE te.employee_id = emp.employee_id
  AND (
      (te.punch_in_time >= emp.start_date AND (emp.end_date IS NULL OR te.punch_in_time <= emp.end_date))
      OR emp.id = (
          SELECT id FROM employments
          WHERE employee_id = te.employee_id
          ORDER BY start_date DESC LIMIT 1
      )
  );
-- Fallback
UPDATE time_entries SET employment_id = (SELECT id FROM employments WHERE employee_id = time_entries.employee_id ORDER BY start_date DESC LIMIT 1) WHERE employment_id IS NULL;
ALTER TABLE time_entries ALTER COLUMN employment_id SET NOT NULL;
ALTER TABLE time_entries DROP COLUMN employee_id;
CREATE INDEX idx_time_entries_employment_id ON time_entries(employment_id);

-- 4. Alter shift_rosters
ALTER TABLE shift_rosters ADD COLUMN employment_id UUID REFERENCES employments(id) ON DELETE RESTRICT;
UPDATE shift_rosters sr
SET employment_id = emp.id
FROM employments emp
WHERE sr.employee_id = emp.employee_id
  AND (
      (sr.date >= emp.start_date AND (emp.end_date IS NULL OR sr.date <= emp.end_date))
      OR emp.id = (
          SELECT id FROM employments
          WHERE employee_id = sr.employee_id
          ORDER BY start_date DESC LIMIT 1
      )
  );
-- Fallback
UPDATE shift_rosters SET employment_id = (SELECT id FROM employments WHERE employee_id = shift_rosters.employee_id ORDER BY start_date DESC LIMIT 1) WHERE employment_id IS NULL;
ALTER TABLE shift_rosters ALTER COLUMN employment_id SET NOT NULL;
ALTER TABLE shift_rosters DROP COLUMN employee_id;
CREATE INDEX idx_shift_rosters_employment_id ON shift_rosters(employment_id);

-- 5. Alter attendance_daily_summaries
ALTER TABLE attendance_daily_summaries ADD COLUMN employment_id UUID REFERENCES employments(id) ON DELETE RESTRICT;
UPDATE attendance_daily_summaries ads
SET employment_id = emp.id
FROM employments emp
WHERE ads.employee_id = emp.employee_id
  AND (
      (ads.date >= emp.start_date AND (emp.end_date IS NULL OR ads.date <= emp.end_date))
      OR emp.id = (
          SELECT id FROM employments
          WHERE employee_id = ads.employee_id
          ORDER BY start_date DESC LIMIT 1
      )
  );
-- Fallback
UPDATE attendance_daily_summaries SET employment_id = (SELECT id FROM employments WHERE employee_id = attendance_daily_summaries.employee_id ORDER BY start_date DESC LIMIT 1) WHERE employment_id IS NULL;
ALTER TABLE attendance_daily_summaries ALTER COLUMN employment_id SET NOT NULL;
ALTER TABLE attendance_daily_summaries DROP COLUMN employee_id;
CREATE INDEX idx_attendance_daily_summaries_employment_id ON attendance_daily_summaries(employment_id);

-- 6. Alter attendance_adjustment_requests
ALTER TABLE attendance_adjustment_requests ADD COLUMN employment_id UUID REFERENCES employments(id) ON DELETE RESTRICT;
UPDATE attendance_adjustment_requests aar
SET employment_id = emp.id
FROM employments emp
WHERE aar.employee_id = emp.employee_id
  AND (
      (aar.requested_punch_in_time >= emp.start_date AND (emp.end_date IS NULL OR aar.requested_punch_in_time <= emp.end_date))
      OR emp.id = (
          SELECT id FROM employments
          WHERE employee_id = aar.employee_id
          ORDER BY start_date DESC LIMIT 1
      )
  );
-- Fallback
UPDATE attendance_adjustment_requests SET employment_id = (SELECT id FROM employments WHERE employee_id = attendance_adjustment_requests.employee_id ORDER BY start_date DESC LIMIT 1) WHERE employment_id IS NULL;
ALTER TABLE attendance_adjustment_requests ALTER COLUMN employment_id SET NOT NULL;
ALTER TABLE attendance_adjustment_requests DROP COLUMN employee_id;
CREATE INDEX idx_attendance_adjustment_requests_employment_id ON attendance_adjustment_requests(employment_id);

-- 7. Alter working_conditions
ALTER TABLE working_conditions ADD COLUMN employment_id UUID REFERENCES employments(id) ON DELETE RESTRICT;
UPDATE working_conditions wc
SET employment_id = emp.id
FROM employments emp
WHERE wc.employee_id = emp.employee_id
  AND (
      (wc.start_date >= emp.start_date AND (emp.end_date IS NULL OR wc.start_date <= emp.end_date))
      OR emp.id = (
          SELECT id FROM employments
          WHERE employee_id = wc.employee_id
          ORDER BY start_date DESC LIMIT 1
      )
  );
-- Fallback
UPDATE working_conditions SET employment_id = (SELECT id FROM employments WHERE employee_id = working_conditions.employee_id ORDER BY start_date DESC LIMIT 1) WHERE employment_id IS NULL;
ALTER TABLE working_conditions ALTER COLUMN employment_id SET NOT NULL;
ALTER TABLE working_conditions DROP COLUMN employee_id;
CREATE INDEX idx_working_conditions_employment_id ON working_conditions(employment_id);

-- 8. Drop legacy columns from employees table
ALTER TABLE employees DROP COLUMN hire_date;
ALTER TABLE employees DROP COLUMN termination_date;
ALTER TABLE employees DROP COLUMN original_hire_date;
ALTER TABLE employees DROP COLUMN is_active;

-- Recreate unique constraints replacing employee_id with employment_id
ALTER TABLE raw_attendance_logs ADD CONSTRAINT unique_raw_log UNIQUE (workspace_id, employment_id, punch_time, punch, status);
ALTER TABLE shift_rosters ADD CONSTRAINT shift_rosters_employment_id_date_key UNIQUE (employment_id, date);
ALTER TABLE attendance_daily_summaries ADD CONSTRAINT attendance_daily_summaries_employment_id_date_key UNIQUE (employment_id, date);

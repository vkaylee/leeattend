-- Revert rotation pattern support
ALTER TABLE working_pattern_details DROP CONSTRAINT IF EXISTS working_pattern_details_schedule_day_check;
ALTER TABLE working_pattern_details RENAME COLUMN schedule_day TO day_of_week;
ALTER TABLE working_pattern_details ADD CONSTRAINT working_pattern_details_day_of_week_check CHECK (day_of_week >= 0 AND day_of_week <= 6);
ALTER TABLE employee_working_patterns DROP COLUMN IF EXISTS anchor_date;
ALTER TABLE working_patterns DROP CONSTRAINT IF EXISTS chk_rotation_fields;
ALTER TABLE working_patterns DROP COLUMN IF EXISTS cycle_length_days;
ALTER TABLE working_patterns DROP COLUMN IF EXISTS pattern_type;


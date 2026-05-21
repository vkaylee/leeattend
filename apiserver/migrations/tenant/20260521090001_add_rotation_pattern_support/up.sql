-- Add rotation pattern support to working_patterns and employee_working_patterns
-- This enables cycle-based (N-day) schedules for security guards and similar roles.

-- 1. Add rotation fields to working_patterns (pattern-level)
ALTER TABLE working_patterns
  ADD COLUMN pattern_type VARCHAR(10) NOT NULL DEFAULT 'weekly',
  ADD COLUMN cycle_length_days INT;

-- Constraint: rotation patterns MUST have cycle_length_days
ALTER TABLE working_patterns
  ADD CONSTRAINT chk_rotation_fields CHECK (
    (pattern_type = 'weekly' AND cycle_length_days IS NULL)
    OR (pattern_type = 'rotation' AND cycle_length_days IS NOT NULL AND cycle_length_days > 0)
  );

-- 2. Add anchor_date to employee_working_patterns (assignment-level)
--    Allows same rotation pattern assigned with different phases per-team/individual
ALTER TABLE employee_working_patterns
  ADD COLUMN anchor_date DATE;

-- 3. Rename day_of_week → schedule_day (dual-purpose)
ALTER TABLE working_pattern_details
  RENAME COLUMN day_of_week TO schedule_day;

COMMENT ON COLUMN working_pattern_details.schedule_day IS
  'For weekly patterns: day_of_week (0=Sun..6=Sat). For rotation patterns: cycle_day (0..N-1)';

COMMENT ON COLUMN employee_working_patterns.anchor_date IS
  'Reference start date for rotation patterns. Determines phase offset per assignment.';

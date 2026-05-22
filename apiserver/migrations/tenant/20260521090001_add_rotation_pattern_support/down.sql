-- Revert rotation pattern support
ALTER TABLE working_pattern_details DROP CONSTRAINT IF EXISTS working_pattern_details_schedule_day_check;
ALTER TABLE working_pattern_details RENAME COLUMN schedule_day TO day_of_week;
ALTER TABLE working_pattern_details ADD CONSTRAINT working_pattern_details_day_of_week_check CHECK (day_of_week >= 0 AND day_of_week <= 6);
ALTER TABLE employee_working_patterns DROP COLUMN IF EXISTS anchor_date;
ALTER TABLE working_patterns DROP CONSTRAINT IF EXISTS chk_rotation_fields;
ALTER TABLE working_patterns DROP COLUMN IF EXISTS cycle_length_days;
ALTER TABLE working_patterns DROP COLUMN IF EXISTS pattern_type;

DROP FUNCTION IF EXISTS resolve_employee_shift(UUID, DATE);

CREATE OR REPLACE FUNCTION resolve_employee_shift(
    p_workspace_id UUID,
    p_target_date DATE
) RETURNS TABLE (
    employee_id UUID,
    shift_id UUID,
    start_time TIME,
    end_time TIME
) AS $$
BEGIN
    RETURN QUERY
    WITH active_employees AS (
        SELECT id FROM employees WHERE workspace_id = p_workspace_id AND is_active = TRUE
    ),
    -- Priority 1: shift_rosters (explicit roster assignment)
    p1_roster AS (
        SELECT sr.employee_id, sr.shift_id, s.start_time, s.end_time
        FROM shift_rosters sr
        JOIN shifts s ON s.id = sr.shift_id
        WHERE sr.date = p_target_date
          AND sr.employee_id IN (SELECT id FROM active_employees)
          AND sr.shift_id IS NOT NULL
    ),
    -- Priority 2: employee working patterns
    p2_pattern AS (
        SELECT DISTINCT ON (ewp.employee_id)
               ewp.employee_id, wpd.shift_id, s.start_time, s.end_time
        FROM employee_working_patterns ewp
        JOIN active_employees ae ON ae.id = ewp.employee_id
        JOIN working_pattern_details wpd ON wpd.pattern_id = ewp.pattern_id
            AND wpd.day_of_week = EXTRACT(DOW FROM p_target_date)::int
        JOIN shifts s ON s.id = wpd.shift_id
        WHERE ewp.start_date <= p_target_date
          AND (ewp.end_date IS NULL OR ewp.end_date >= p_target_date)
          AND NOT EXISTS (SELECT 1 FROM p1_roster p1 WHERE p1.employee_id = ewp.employee_id)
        ORDER BY ewp.employee_id, ewp.created_at DESC
    ),
    -- Priority 4: department working patterns
    p4_dept_pattern AS (
        SELECT DISTINCT ON (edh.employee_id)
               edh.employee_id, wpd.shift_id, s.start_time, s.end_time
        FROM employee_department_history edh
        JOIN active_employees ae ON ae.id = edh.employee_id
        JOIN departments d ON d.id = edh.department_id
            AND d.default_working_pattern_id IS NOT NULL
        JOIN working_pattern_details wpd ON wpd.pattern_id = d.default_working_pattern_id
            AND wpd.day_of_week = EXTRACT(DOW FROM p_target_date)::int
        JOIN shifts s ON s.id = wpd.shift_id
        WHERE edh.effective_date <= p_target_date
          AND NOT EXISTS (SELECT 1 FROM p1_roster p1 WHERE p1.employee_id = edh.employee_id)
          AND NOT EXISTS (SELECT 1 FROM p2_pattern p2 WHERE p2.employee_id = edh.employee_id)
        ORDER BY edh.employee_id, edh.effective_date DESC
    ),
    -- Priority 5: employee_shifts (legacy fixed assignment)
    p5_emp_shift AS (
        SELECT DISTINCT ON (es.employee_id)
               es.employee_id, es.shift_id, s.start_time, s.end_time
        FROM employee_shifts es
        JOIN active_employees ae ON ae.id = es.employee_id
        JOIN shifts s ON s.id = es.shift_id
        WHERE es.start_date <= p_target_date
          AND (es.end_date IS NULL OR es.end_date >= p_target_date)
          AND NOT EXISTS (SELECT 1 FROM p1_roster p1 WHERE p1.employee_id = es.employee_id)
          AND NOT EXISTS (SELECT 1 FROM p2_pattern p2 WHERE p2.employee_id = es.employee_id)
          AND NOT EXISTS (SELECT 1 FROM p4_dept_pattern p4 WHERE p4.employee_id = es.employee_id)
        ORDER BY es.employee_id, es.created_at DESC
    ),
    -- Priority 6: department default shifts
    p6_dept_shift AS (
        SELECT DISTINCT ON (edh.employee_id)
               edh.employee_id, dds.shift_id, s.start_time, s.end_time
        FROM employee_department_history edh
        JOIN active_employees ae ON ae.id = edh.employee_id
        JOIN department_default_shifts dds ON dds.department_id = edh.department_id
        JOIN shifts s ON s.id = dds.shift_id
        WHERE edh.effective_date <= p_target_date
          AND NOT EXISTS (SELECT 1 FROM p1_roster p1 WHERE p1.employee_id = edh.employee_id)
          AND NOT EXISTS (SELECT 1 FROM p2_pattern p2 WHERE p2.employee_id = edh.employee_id)
          AND NOT EXISTS (SELECT 1 FROM p4_dept_pattern p4 WHERE p4.employee_id = edh.employee_id)
          AND NOT EXISTS (SELECT 1 FROM p5_emp_shift p5 WHERE p5.employee_id = edh.employee_id)
        ORDER BY edh.employee_id, edh.effective_date DESC
    )
    SELECT p1_roster.employee_id, p1_roster.shift_id, p1_roster.start_time, p1_roster.end_time FROM p1_roster
    UNION ALL SELECT p2_pattern.employee_id, p2_pattern.shift_id, p2_pattern.start_time, p2_pattern.end_time FROM p2_pattern
    UNION ALL SELECT p4_dept_pattern.employee_id, p4_dept_pattern.shift_id, p4_dept_pattern.start_time, p4_dept_pattern.end_time FROM p4_dept_pattern
    UNION ALL SELECT p5_emp_shift.employee_id, p5_emp_shift.shift_id, p5_emp_shift.start_time, p5_emp_shift.end_time FROM p5_emp_shift
    UNION ALL SELECT p6_dept_shift.employee_id, p6_dept_shift.shift_id, p6_dept_shift.start_time, p6_dept_shift.end_time FROM p6_dept_shift;

END;
$$ LANGUAGE plpgsql STABLE;


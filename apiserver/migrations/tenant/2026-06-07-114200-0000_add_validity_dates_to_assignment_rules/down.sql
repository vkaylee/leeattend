-- 1. Drop validity dates from assignment rules
ALTER TABLE assignment_rules DROP COLUMN IF EXISTS start_date;
ALTER TABLE assignment_rules DROP COLUMN IF EXISTS end_date;

-- 2. Restore department default patterns column and table
ALTER TABLE departments ADD COLUMN default_working_pattern_id UUID;
ALTER TABLE departments ADD CONSTRAINT departments_default_working_pattern_id_fkey FOREIGN KEY (default_working_pattern_id) REFERENCES working_patterns(id) ON DELETE SET NULL;

CREATE TABLE department_default_shifts (
    department_id UUID NOT NULL,
    shift_id UUID NOT NULL
);
ALTER TABLE ONLY department_default_shifts ADD CONSTRAINT department_default_shifts_pkey PRIMARY KEY (department_id, shift_id);
ALTER TABLE ONLY department_default_shifts ADD CONSTRAINT department_default_shifts_department_id_fkey FOREIGN KEY (department_id) REFERENCES departments(id) ON DELETE CASCADE;
ALTER TABLE ONLY department_default_shifts ADD CONSTRAINT department_default_shifts_shift_id_fkey FOREIGN KEY (shift_id) REFERENCES shifts(id) ON DELETE CASCADE;

-- 3. Restore resolve_employee_shift function to its original definition
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
        JOIN working_patterns wp ON wp.id = ewp.pattern_id
        JOIN working_pattern_details wpd ON wpd.pattern_id = ewp.pattern_id
            AND wpd.schedule_day = CASE
                WHEN wp.pattern_type = 'rotation' THEN
                    MOD(MOD(p_target_date - COALESCE(ewp.anchor_date, ewp.start_date), wp.cycle_length_days) + wp.cycle_length_days, wp.cycle_length_days)
                ELSE EXTRACT(DOW FROM p_target_date)::int
            END
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
        JOIN working_patterns wp ON wp.id = d.default_working_pattern_id
        JOIN working_pattern_details wpd ON wpd.pattern_id = d.default_working_pattern_id
            AND wpd.schedule_day = CASE
                WHEN wp.pattern_type = 'rotation' THEN
                    MOD(MOD(p_target_date - edh.effective_date, wp.cycle_length_days) + wp.cycle_length_days, wp.cycle_length_days)
                ELSE EXTRACT(DOW FROM p_target_date)::int
            END
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

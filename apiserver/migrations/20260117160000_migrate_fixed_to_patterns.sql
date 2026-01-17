-- Migration: Migrate Legacy Fixed Shifts to Working Patterns (Soft Deprecation)
-- 1. Create Temporary Table for mapping
CREATE TEMP TABLE tmp_fixed_shift_map (
    shift_id UUID,
    workspace_id UUID,
    shift_name TEXT,
    new_pattern_id UUID
);

-- 2. Identify unique shifts used in legacy assignments
INSERT INTO tmp_fixed_shift_map (shift_id, workspace_id, shift_name, new_pattern_id)
SELECT DISTINCT s.id, s.workspace_id, s.name, gen_random_uuid()
FROM shifts s
WHERE s.id IN (SELECT shift_id FROM employee_shifts)
   OR s.id IN (SELECT shift_id FROM department_default_shifts);

-- 3. Create Patterns
INSERT INTO working_patterns (id, workspace_id, name, description, is_active, created_at, updated_at)
SELECT 
    new_pattern_id, 
    workspace_id, 
    'Fixed: ' || shift_name, 
    'Auto-migrated from legacy fixed assignment', 
    true, 
    NOW(), 
    NOW()
FROM tmp_fixed_shift_map;

-- 4. Create Pattern Details (7 days)
INSERT INTO working_pattern_details (id, pattern_id, day_of_week, shift_id)
SELECT gen_random_uuid(), new_pattern_id, dow, shift_id
FROM tmp_fixed_shift_map, generate_series(0, 6) as dow;

-- 5. Migrate Employee Assignments
INSERT INTO employee_working_patterns (id, workspace_id, employee_id, pattern_id, start_date, end_date, created_at, updated_at)
SELECT 
    gen_random_uuid(), 
    e.workspace_id, 
    es.employee_id, 
    m.new_pattern_id, 
    es.start_date, 
    es.end_date, 
    es.created_at, 
    es.created_at
FROM employee_shifts es
JOIN employees e ON es.employee_id = e.id
JOIN tmp_fixed_shift_map m ON es.shift_id = m.shift_id;

-- 6. Migrate Department Assignments
UPDATE departments d
SET default_working_pattern_id = m.new_pattern_id
FROM department_default_shifts dds
JOIN tmp_fixed_shift_map m ON dds.shift_id = m.shift_id
WHERE d.id = dds.department_id;

-- 7. Cleanup
DROP TABLE tmp_fixed_shift_map;
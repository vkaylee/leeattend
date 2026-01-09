-- Deduplicate: Keep only the latest assignment for each employee and start_date
DELETE FROM employee_shifts a
USING employee_shifts b
WHERE a.id < b.id
  AND a.employee_id = b.employee_id
  AND a.start_date = b.start_date;

-- Add Unique Constraint
ALTER TABLE employee_shifts 
ADD CONSTRAINT uq_employee_shift_start UNIQUE (employee_id, start_date);

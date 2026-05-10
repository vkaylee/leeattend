ALTER TABLE employees 
ADD COLUMN hire_date DATE,
ADD COLUMN termination_date DATE;

UPDATE employees e
SET hire_date = (
    SELECT hire_date FROM employment_periods ep 
    WHERE ep.employee_id = e.id 
    ORDER BY hire_date ASC LIMIT 1
);

UPDATE employees e
SET termination_date = (
    SELECT termination_date FROM employment_periods ep 
    WHERE ep.employee_id = e.id 
    ORDER BY hire_date DESC LIMIT 1
);

UPDATE employees SET hire_date = created_at::DATE WHERE hire_date IS NULL;

ALTER TABLE employees ALTER COLUMN hire_date SET NOT NULL;

DROP TABLE employment_periods;

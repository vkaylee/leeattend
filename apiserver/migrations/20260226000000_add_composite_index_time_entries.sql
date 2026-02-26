-- Add composite index for the batch time entry query used in attendance report generation.
-- The query filters by employee_id = ANY($1) AND punch_in_time between range,
-- and orders by employee_id, punch_in_time DESC.
-- Previously, PostgreSQL had to bitmap-merge two separate single-column indexes.

CREATE INDEX IF NOT EXISTS idx_time_entries_emp_punch
ON time_entries(employee_id, punch_in_time DESC);

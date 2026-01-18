-- Up Migration: Standardize schema with ENUMs and constraints

-- 1. Member Roles
CREATE TYPE member_role_type AS ENUM ('owner', 'admin', 'member');
ALTER TABLE workspace_members ALTER COLUMN role DROP DEFAULT;
ALTER TABLE workspace_members ALTER COLUMN role TYPE member_role_type USING role::member_role_type;

ALTER TABLE member_extra_roles ALTER COLUMN role DROP DEFAULT;
ALTER TABLE member_extra_roles ALTER COLUMN role TYPE member_role_type USING role::member_role_type;

-- 2. Leave Status
CREATE TYPE leave_status_type AS ENUM ('pending', 'approved', 'rejected');
ALTER TABLE leave_requests ALTER COLUMN status DROP DEFAULT;
ALTER TABLE leave_requests ALTER COLUMN status TYPE leave_status_type USING status::leave_status_type;
ALTER TABLE leave_requests ALTER COLUMN status SET DEFAULT 'pending'::leave_status_type;

-- 3. Attendance Adjustment Status
CREATE TYPE adjustment_status_type AS ENUM ('pending', 'approved', 'rejected');
ALTER TABLE attendance_adjustment_requests ALTER COLUMN status DROP DEFAULT;
ALTER TABLE attendance_adjustment_requests ALTER COLUMN status TYPE adjustment_status_type USING status::adjustment_status_type;
ALTER TABLE attendance_adjustment_requests ALTER COLUMN status SET DEFAULT 'pending'::adjustment_status_type;

-- 4. Attendance Log Source
CREATE TYPE log_source_type AS ENUM ('device', 'import', 'manual');
ALTER TABLE raw_attendance_logs ALTER COLUMN source DROP DEFAULT;
ALTER TABLE raw_attendance_logs ALTER COLUMN source TYPE log_source_type USING source::log_source_type;

-- 5. Attendance Import Job Status
CREATE TYPE import_job_status_type AS ENUM ('pending', 'processing', 'completed', 'failed');
ALTER TABLE attendance_import_jobs ALTER COLUMN status DROP DEFAULT;
ALTER TABLE attendance_import_jobs ALTER COLUMN status TYPE import_job_status_type USING status::import_job_status_type;
ALTER TABLE attendance_import_jobs ALTER COLUMN status SET DEFAULT 'pending'::import_job_status_type;

-- 6. Leave Transaction Type
CREATE TYPE leave_transaction_type AS ENUM ('accrual', 'usage', 'adjustment', 'carry_over');

-- 7. Assignment History Type
CREATE TYPE assignment_history_type AS ENUM ('roster', 'pattern', 'department_pattern', 'fixed', 'department_fixed');

-- 8. Additional Constraints
ALTER TABLE employees ADD CONSTRAINT chk_employee_names CHECK (length(first_name) > 0 AND length(last_name) > 0);
ALTER TABLE workspaces ADD CONSTRAINT chk_workspace_name CHECK (length(name) > 0);
ALTER TABLE leave_requests DROP COLUMN IF EXISTS attachment_file_ids;
ALTER TABLE employee_profiles DROP COLUMN IF EXISTS avatar_file_id;
DROP TABLE IF EXISTS workspace_files;

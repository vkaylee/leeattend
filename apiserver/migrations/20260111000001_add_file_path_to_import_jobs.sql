-- Up Migration
ALTER TABLE attendance_import_jobs ADD COLUMN file_path TEXT;

-- Down Migration
-- ALTER TABLE attendance_import_jobs DROP COLUMN file_path;

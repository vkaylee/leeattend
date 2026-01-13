-- Add target_type to attendance_import_jobs
ALTER TABLE attendance_import_jobs ADD COLUMN target_type VARCHAR(50) NOT NULL DEFAULT 'adjustment';

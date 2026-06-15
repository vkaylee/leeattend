-- Migration Down: Remove attachment_file_ids columns
ALTER TABLE attendance_adjustment_requests DROP COLUMN attachment_file_ids;
ALTER TABLE overtime_requests DROP COLUMN attachment_file_ids;

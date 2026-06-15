-- Migration: Add attachment_file_ids to attendance_adjustment_requests and overtime_requests
ALTER TABLE attendance_adjustment_requests ADD COLUMN attachment_file_ids UUID[];
ALTER TABLE overtime_requests ADD COLUMN attachment_file_ids UUID[];

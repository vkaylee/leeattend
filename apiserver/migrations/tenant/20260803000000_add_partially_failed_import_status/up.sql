-- Add partially_failed status to import_job_status_type enum
ALTER TYPE public.import_job_status_type ADD VALUE 'partially_failed';

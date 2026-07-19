-- Reverse Biometric Enrollment

-- 3. Unlink devices from work location
ALTER TABLE public.devices
    DROP CONSTRAINT IF EXISTS devices_location_id_fkey;

DROP INDEX IF EXISTS idx_devices_location_id;

ALTER TABLE public.devices
    DROP COLUMN IF EXISTS location_id;

-- 2. Drop device_enrollments
DROP TABLE IF EXISTS public.device_enrollments CASCADE;

-- 1. Drop biometric template store
DROP TABLE IF EXISTS public.employee_biometric_templates CASCADE;

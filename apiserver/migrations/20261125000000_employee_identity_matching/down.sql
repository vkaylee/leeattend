BEGIN;

DROP INDEX IF EXISTS public.idx_employee_profiles_phone_lookup_hash;
DROP INDEX IF EXISTS public.idx_employee_profiles_national_id_lookup_hash;

ALTER TABLE public.employee_profiles
    DROP CONSTRAINT IF EXISTS employee_profiles_identity_lookup_hash_check,
    DROP COLUMN IF EXISTS phone_lookup_hash,
    DROP COLUMN IF EXISTS national_id_lookup_hash;

DELETE FROM public.schema_migrations
WHERE version = '20261125000000_employee_identity_matching';

COMMIT;

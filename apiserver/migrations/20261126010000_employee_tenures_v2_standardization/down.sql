BEGIN;

-- Guarded rollback: restore pre-v2 transitional structure only on disposable databases.
DROP TRIGGER IF EXISTS trg_validate_segment_relationship_bounds ON public.employments;
DROP TRIGGER IF EXISTS trg_validate_relationship_tenure_bounds ON public.employment_relationships;
DROP TRIGGER IF EXISTS trg_validate_tenure_bounds_guard ON public.employee_tenures;
DROP TRIGGER IF EXISTS trg_roster_cache_invalidation_employee_tenures ON public.employee_tenures;

DROP FUNCTION IF EXISTS public.fn_validate_segment_relationship_bounds();
DROP FUNCTION IF EXISTS public.fn_validate_relationship_tenure_bounds();
DROP FUNCTION IF EXISTS public.fn_validate_tenure_bounds_guard();

ALTER TABLE public.file_acknowledgements
    DROP CONSTRAINT IF EXISTS file_acknowledgements_tenure_identity_fkey,
    DROP COLUMN IF EXISTS tenure_id;
DROP INDEX IF EXISTS public.idx_file_acknowledgements_tenure_id;

ALTER TABLE public.employee_documents
    DROP CONSTRAINT IF EXISTS employee_documents_tenure_identity_fkey,
    DROP COLUMN IF EXISTS tenure_id;
DROP INDEX IF EXISTS public.idx_employee_documents_tenure_id;

ALTER TABLE public.employments
    DROP CONSTRAINT IF EXISTS employments_dates_check,
    ALTER COLUMN relationship_id DROP NOT NULL;

ALTER TABLE public.employment_relationships
    ALTER COLUMN tenure_id DROP NOT NULL;

ALTER TABLE public.employee_tenures
    DROP CONSTRAINT IF EXISTS employee_tenures_permanent_code_owner,
    DROP CONSTRAINT IF EXISTS employee_tenures_hire_type_check,
    DROP CONSTRAINT IF EXISTS employee_tenures_prior_service_check,
    DROP COLUMN IF EXISTS notes,
    DROP COLUMN IF EXISTS rehire_eligible,
    DROP COLUMN IF EXISTS termination_reason,
    DROP COLUMN IF EXISTS prior_service_credit_days,
    DROP COLUMN IF EXISTS hire_type,
    DROP COLUMN IF EXISTS employee_number_normalized;

DELETE FROM public.schema_migrations
WHERE version = '20261126010000_employee_tenures_v2_standardization';

COMMIT;

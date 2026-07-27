-- Reverse of 20260727120000_attendance_policy_versions_extend/up.sql
DROP TRIGGER IF EXISTS trg_attendance_policy_versions_no_delete_frozen
    ON public.attendance_policy_versions;
DROP FUNCTION IF EXISTS public.enforce_policy_version_no_delete_when_frozen();

DROP TRIGGER IF EXISTS trg_attendance_policy_versions_immutability
    ON public.attendance_policy_versions;
DROP FUNCTION IF EXISTS public.enforce_policy_version_immutability();

DROP INDEX IF EXISTS public.idx_attendance_policy_versions_rules_hash;
DROP INDEX IF EXISTS public.idx_attendance_policy_versions_policy_status;

ALTER TABLE public.attendance_policy_versions
    DROP COLUMN IF EXISTS rules_hash,
    DROP COLUMN IF EXISTS first_referenced_at,
    DROP COLUMN IF EXISTS delta_specs,
    DROP COLUMN IF EXISTS conditional_overrides,
    DROP COLUMN IF EXISTS base_rules,
    DROP COLUMN IF EXISTS updated_at,
    DROP COLUMN IF EXISTS updated_by,
    DROP COLUMN IF EXISTS description,
    DROP COLUMN IF EXISTS name,
    DROP COLUMN IF EXISTS status;

DROP TYPE IF EXISTS public.policy_version_status_type;

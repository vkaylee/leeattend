BEGIN;

-- Guarded disposable-DB rollback for Work Policy v2 standardization.
DROP TRIGGER IF EXISTS trg_employment_relationships_policy_bounds_guard ON public.employment_relationships;
DROP TRIGGER IF EXISTS trg_work_policy_memberships_rel_bounds ON public.work_policy_memberships;
DROP TRIGGER IF EXISTS trg_guard_work_policy_version_immutability ON public.work_policy_versions;
DROP TRIGGER IF EXISTS trg_guard_overtime_policy_version_immutability ON public.overtime_policy_versions;

DROP FUNCTION IF EXISTS public.fn_validate_relationship_policy_bounds_guard();
DROP FUNCTION IF EXISTS public.fn_validate_work_policy_membership_rel_bounds();
DROP FUNCTION IF EXISTS public.fn_guard_work_policy_version_immutability();
DROP FUNCTION IF EXISTS public.fn_guard_overtime_policy_version_immutability();

ALTER TABLE public.attendance_daily_summaries
    DROP CONSTRAINT IF EXISTS attendance_daily_summaries_work_policy_version_id_fkey;

DROP INDEX IF EXISTS public.payslip_segments_work_policy_version_idx;
ALTER TABLE public.payslip_segments
    DROP CONSTRAINT IF EXISTS payslip_segments_work_policy_version_id_fkey,
    DROP COLUMN IF EXISTS work_policy_version_id;

DROP TABLE IF EXISTS public.work_policy_memberships CASCADE;
DROP TABLE IF EXISTS public.work_policy_versions CASCADE;
DROP TABLE IF EXISTS public.work_policy_groups CASCADE;
DROP TABLE IF EXISTS public.overtime_policy_versions CASCADE;

DELETE FROM public.schema_migrations
WHERE version = '20261127000000_work_policy_v2_standardization';

COMMIT;

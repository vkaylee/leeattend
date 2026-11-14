BEGIN;

DELETE FROM public.schema_migrations
WHERE version = '20261114110000_payroll_segment_policy_snapshots';

DROP INDEX IF EXISTS public.payslip_segments_labor_policy_version_idx;
ALTER TABLE public.payslip_segments
    DROP COLUMN IF EXISTS labor_policy_group_version_id;

COMMIT;

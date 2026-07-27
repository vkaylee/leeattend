DROP TRIGGER IF EXISTS trg_daily_summary_snapshot_immutability
    ON public.attendance_daily_summaries;

DROP FUNCTION IF EXISTS enforce_daily_summary_snapshot_immutability();

DROP INDEX IF EXISTS idx_daily_summary_superseded_chain;
DROP INDEX IF EXISTS idx_daily_summary_live;
DROP INDEX IF EXISTS idx_daily_summary_policy_version;

ALTER TABLE public.attendance_daily_summaries
    DROP COLUMN IF EXISTS superseded_at,
    DROP COLUMN IF EXISTS superseded_by_id,
    DROP COLUMN IF EXISTS is_superseded,
    DROP COLUMN IF EXISTS effective_rules_snapshot,
    DROP COLUMN IF EXISTS policy_version_id;

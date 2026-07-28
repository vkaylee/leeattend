-- ADR-005 rollback: drop diff-monitoring view, index, and column.

DROP VIEW IF EXISTS public.cascade_diff_report_v;
DROP INDEX IF EXISTS public.idx_daily_summary_diff_monitor;
ALTER TABLE public.attendance_daily_summaries DROP COLUMN IF EXISTS legacy_rules_snapshot;

BEGIN;

-- Fresh-install-only rollback for attendance_schedule; run phases in reverse order.
-- Snapshot columns are dropped with their owning tables below; no historical evidence is retained in fresh-install rollback.

-- This is destructive and must not run against production data.

DROP TABLE IF EXISTS public.working_patterns CASCADE;

DROP TABLE IF EXISTS public.working_pattern_details CASCADE;

DROP TABLE IF EXISTS public.work_segments CASCADE;

DROP TABLE IF EXISTS public.time_entries CASCADE;

DROP TABLE IF EXISTS public.shift_versions CASCADE;

DROP TABLE IF EXISTS public.shift_templates CASCADE;

DROP TABLE IF EXISTS public.shift_swap_requests CASCADE;

DROP TABLE IF EXISTS public.shift_rosters CASCADE;

DROP TABLE IF EXISTS public.shift_default_intervals CASCADE;

DROP TABLE IF EXISTS public.shift_audit_log CASCADE;

DROP TABLE IF EXISTS public.schedule_shadow_mismatches CASCADE;

DROP TABLE IF EXISTS public.schedule_publication_batches CASCADE;

DROP TABLE IF EXISTS public.schedule_previews CASCADE;

DROP TABLE IF EXISTS public.schedule_office_policies CASCADE;

DROP TABLE IF EXISTS public.schedule_legacy_mutation_evidence CASCADE;

DROP TABLE IF EXISTS public.schedule_day_revisions CASCADE;

DROP TABLE IF EXISTS public.raw_attendance_logs CASCADE;

DROP TABLE IF EXISTS public.payroll_periods CASCADE;

DROP TABLE IF EXISTS public.holidays CASCADE;

DROP TABLE IF EXISTS public.external_attendance_events CASCADE;

DROP TABLE IF EXISTS public.employee_working_patterns CASCADE;

DROP TABLE IF EXISTS public.employee_shifts CASCADE;

DROP TABLE IF EXISTS public.attendance_staging_logs_y2026m12 CASCADE;

DROP TABLE IF EXISTS public.attendance_staging_logs_y2026m11 CASCADE;

DROP TABLE IF EXISTS public.attendance_staging_logs_y2026m10 CASCADE;

DROP TABLE IF EXISTS public.attendance_staging_logs_y2026m09 CASCADE;

DROP TABLE IF EXISTS public.attendance_staging_logs_y2026m08 CASCADE;

DROP TABLE IF EXISTS public.attendance_staging_logs_y2026m07 CASCADE;

DROP TABLE IF EXISTS public.attendance_staging_logs_y2026m06 CASCADE;

DROP TABLE IF EXISTS public.attendance_staging_logs_y2026m05 CASCADE;

DROP TABLE IF EXISTS public.attendance_staging_logs_y2026m04 CASCADE;

DROP TABLE IF EXISTS public.attendance_staging_logs_y2026m03 CASCADE;

DROP TABLE IF EXISTS public.attendance_staging_logs_y2026m02 CASCADE;

DROP TABLE IF EXISTS public.attendance_staging_logs_y2026m01 CASCADE;

DROP TABLE IF EXISTS public.attendance_staging_logs_default CASCADE;

DROP TABLE IF EXISTS public.attendance_staging_logs CASCADE;

DROP TABLE IF EXISTS public.attendance_sources CASCADE;

DROP TABLE IF EXISTS public.attendance_source_pull_states CASCADE;

DROP TABLE IF EXISTS public.attendance_periods CASCADE;

DROP TABLE IF EXISTS public.attendance_log_errors CASCADE;

DROP TABLE IF EXISTS public.attendance_import_jobs CASCADE;

DROP TABLE IF EXISTS public.attendance_daily_summaries CASCADE;

DROP TABLE IF EXISTS public.attendance_closures CASCADE;

DROP TABLE IF EXISTS public.attendance_closure_days CASCADE;

DROP INDEX IF EXISTS public.attendance_adjustment_evidence_scope_idx;
DROP INDEX IF EXISTS public.attendance_adjustment_evidence_request_idx;
DROP TABLE IF EXISTS public.attendance_adjustment_evidence CASCADE;
DROP INDEX IF EXISTS public.attendance_adjustment_requests_type_idx;
DROP TABLE IF EXISTS public.attendance_adjustment_requests CASCADE;

DROP TABLE IF EXISTS public.working_pattern_assignment_rules CASCADE;

DROP TABLE IF EXISTS public.schedule_day_overrides CASCADE;
DROP TABLE IF EXISTS public.attendance_action_idempotency CASCADE;
DROP TABLE IF EXISTS public.time_entry_breaks CASCADE;

DROP TRIGGER IF EXISTS trg_attendance_punch_events_immutable ON public.attendance_punch_events;
DROP FUNCTION IF EXISTS public.guard_attendance_punch_events_immutable();
DROP TABLE IF EXISTS public.attendance_punch_events CASCADE;

DELETE FROM public.schema_migrations WHERE version = 'fresh_attendance_schedule_v1';

COMMIT;

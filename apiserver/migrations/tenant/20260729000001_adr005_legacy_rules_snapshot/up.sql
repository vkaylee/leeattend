-- ============================================================================
-- ADR-005 §1-2: Diff rate monitoring infrastructure
--
-- Adds:
--   - legacy_rules_snapshot        — JSONB storing what the legacy resolver produced
--   - cascade_diff_report_v        — SQL view comparing cascade vs legacy per row
--
-- The cascade engine already writes effective_rules_snapshot (ADR-004) with
-- final_rules + cascade_trace + clamp_warnings.  This migration adds the
-- counterpart column so the dual-run diff can be queried directly from the DB.
--
-- The diff reporting view powers the daily `scripts/adr005_diff_report.py`
-- script and the admin dashboard diff monitoring widget.
-- ============================================================================

-- ── Column ─────────────────────────────────────────────────────────────

ALTER TABLE public.attendance_daily_summaries
    ADD COLUMN IF NOT EXISTS legacy_rules_snapshot JSONB;

COMMENT ON COLUMN public.attendance_daily_summaries.legacy_rules_snapshot IS
    'ADR-005 — legacy resolver PolicyRules serialised as JSONB during dual-run.
     Used by cascade_diff_report_v for Group A/B/C diff classification.
     NULL for rows written before the dual-run instrumented the legacy path
     (i.e. before ADR-005 operational cutover).';

-- ── Diff reporting view ────────────────────────────────────────────────
--
-- Returns one row per daily summary that has both cascade and legacy
-- snapshots.  Numeric deltas are computed inline; the Python report script
-- layers Group A/B/C classification on top using the expected-diffs manifest.
--
-- Only non-superseded (live) rows are included — superseded rows are
-- historical corrections and not relevant for dual-run monitoring.

CREATE OR REPLACE VIEW public.cascade_diff_report_v AS
SELECT
    a.id                              AS summary_id,
    a.workspace_id,
    a.employment_id,
    a.date,
    a.policy_version_id,
    -- ── Grace period ──────────────────────────────────────────────
    COALESCE(
        (a.effective_rules_snapshot -> 'final_rules' ->> 'grace_period_minutes')::int,
        0
    )                                 AS cascade_grace_minutes,
    COALESCE(
        (a.legacy_rules_snapshot ->> 'grace_period_minutes')::int,
        0
    )                                 AS legacy_grace_minutes,
    COALESCE(
        (a.effective_rules_snapshot -> 'final_rules' ->> 'grace_period_minutes')::int, 0
    ) - COALESCE(
        (a.legacy_rules_snapshot ->> 'grace_period_minutes')::int, 0
    )                                 AS grace_delta_minutes,
    -- ── Authorised early leave ────────────────────────────────────
    COALESCE(
        (a.effective_rules_snapshot -> 'final_rules' ->> 'authorized_early_leave_minutes')::int,
        0
    )                                 AS cascade_ael_minutes,
    COALESCE(
        (a.legacy_rules_snapshot ->> 'authorized_early_leave_minutes')::int,
        0
    )                                 AS legacy_ael_minutes,
    COALESCE(
        (a.effective_rules_snapshot -> 'final_rules' ->> 'authorized_early_leave_minutes')::int, 0
    ) - COALESCE(
        (a.legacy_rules_snapshot ->> 'authorized_early_leave_minutes')::int, 0
    )                                 AS ael_delta_minutes,
    -- ── Boolean flags (informational — not delta-quantifiable) ────
    (a.effective_rules_snapshot -> 'final_rules' ->> 'is_early_leave_paid')::bool
                                      AS cascade_early_leave_paid,
    (a.legacy_rules_snapshot ->> 'is_early_leave_paid')::bool
                                      AS legacy_early_leave_paid,
    (a.effective_rules_snapshot -> 'final_rules' ->> 'is_break_auto_deduct')::bool
                                      AS cascade_break_auto_deduct,
    (a.legacy_rules_snapshot ->> 'is_break_auto_deduct')::bool
                                      AS legacy_break_auto_deduct,
    -- ── Composite diff flag ───────────────────────────────────────
    (a.effective_rules_snapshot -> 'final_rules' IS DISTINCT FROM
     a.legacy_rules_snapshot)          AS has_any_diff,
    -- ── Metadata ──────────────────────────────────────────────────
    a.effective_rules_snapshot -> 'clamp_warnings' AS cascade_clamp_warnings,
    jsonb_array_length(
        COALESCE(a.effective_rules_snapshot -> 'cascade_trace', '[]'::jsonb)
    )                                 AS cascade_trace_len,
    a.created_at,
    a.updated_at
FROM public.attendance_daily_summaries a
WHERE a.effective_rules_snapshot IS NOT NULL
  AND a.legacy_rules_snapshot IS NOT NULL
  AND a.is_superseded = false;

COMMENT ON VIEW public.cascade_diff_report_v IS
    'ADR-005 — dual-run diff comparison view.
     Each row that has both cascade (effective_rules_snapshot->final_rules)
     and legacy (legacy_rules_snapshot) policy output is returned with
     per-field deltas.  The daily report script layers Group A/B/C
     classification atop this view using .attendagent/migration_expected_diffs.md.';

-- ── Index to speed up diff report queries ──────────────────────────────
CREATE INDEX IF NOT EXISTS idx_daily_summary_diff_monitor
    ON public.attendance_daily_summaries (workspace_id, date)
    WHERE effective_rules_snapshot IS NOT NULL
      AND legacy_rules_snapshot IS NOT NULL
      AND is_superseded = false;

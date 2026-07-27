-- ============================================================================
-- ADR-004: Immutable snapshot & supersede-and-recompute on attendance_daily_summaries.
--
-- Adds:
--   - policy_version_id           — which ACTIVE/FROZEN policy produced this row
--   - effective_rules_snapshot    — final_rules + cascade_trace + clamp_warnings
--   - is_superseded / superseded_by_id — historical correction chain
--
-- Plan naming note: the plan refers to `daily_attendance`; the actual table
-- in this codebase is `attendance_daily_summaries`. Same semantics.
-- ============================================================================

ALTER TABLE public.attendance_daily_summaries
    ADD COLUMN policy_version_id UUID
        REFERENCES public.attendance_policy_versions(id),
    ADD COLUMN effective_rules_snapshot JSONB,
    ADD COLUMN is_superseded BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN superseded_by_id UUID
        REFERENCES public.attendance_daily_summaries(id) ON DELETE SET NULL,
    ADD COLUMN superseded_at TIMESTAMPTZ;

-- Historical rows written before the cascade engine existed have no snapshot.
-- Backfill happens in the dual-run migration phase (ADR-005), not here.

CREATE INDEX idx_daily_summary_policy_version
    ON public.attendance_daily_summaries (policy_version_id);

-- Partial index — only the "live" (non-superseded) row per (employment, date)
-- is what payroll and reports should read.
CREATE INDEX idx_daily_summary_live
    ON public.attendance_daily_summaries (employment_id, date)
    WHERE is_superseded = false;

CREATE INDEX idx_daily_summary_superseded_chain
    ON public.attendance_daily_summaries (superseded_by_id)
    WHERE superseded_by_id IS NOT NULL;

-- ---------------------------------------------------------------------------
-- Immutability guard on FROZEN policy snapshot.
--
-- Once a daily summary has a policy_version_id whose version is FROZEN,
-- the snapshot columns must never be silently mutated. To correct history,
-- callers must go through supersede-and-recompute (insert new row, set
-- is_superseded=true on the old row).
--
-- Non-snapshot columns (manager notes, OT claim entry, etc.) remain mutable.
-- ---------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION enforce_daily_summary_snapshot_immutability()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    v_status TEXT;
BEGIN
    -- Only guard rows that actually have a cascade snapshot.
    IF OLD.policy_version_id IS NULL THEN
        RETURN NEW;
    END IF;

    SELECT status::text INTO v_status
    FROM public.attendance_policy_versions
    WHERE id = OLD.policy_version_id;

    IF v_status = 'FROZEN' THEN
        IF OLD.policy_version_id IS DISTINCT FROM NEW.policy_version_id
           OR OLD.effective_rules_snapshot IS DISTINCT FROM NEW.effective_rules_snapshot
        THEN
            RAISE EXCEPTION
                'DAILY_SUMMARY_SNAPSHOT_IMMUTABLE: row % is bound to FROZEN policy version %; use supersede-and-recompute instead',
                OLD.id, OLD.policy_version_id;
        END IF;

        -- Superseding an old row is allowed. Un-superseding is not.
        IF OLD.is_superseded = true AND NEW.is_superseded = false THEN
            RAISE EXCEPTION
                'DAILY_SUMMARY_UNSUPERSEDE_FORBIDDEN: row % already superseded, cannot revert',
                OLD.id;
        END IF;
    END IF;

    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_daily_summary_snapshot_immutability
    BEFORE UPDATE ON public.attendance_daily_summaries
    FOR EACH ROW
    EXECUTE FUNCTION enforce_daily_summary_snapshot_immutability();

COMMENT ON COLUMN public.attendance_daily_summaries.policy_version_id IS
    'ADR-004 — policy version whose cascade produced effective_rules_snapshot.';
COMMENT ON COLUMN public.attendance_daily_summaries.effective_rules_snapshot IS
    'ADR-004 — { final_rules, cascade_trace, clamp_warnings }. Immutable once bound to FROZEN version.';
COMMENT ON COLUMN public.attendance_daily_summaries.is_superseded IS
    'ADR-004 — true when a newer recomputation replaced this row. Payroll/reports read only is_superseded=false.';
COMMENT ON COLUMN public.attendance_daily_summaries.superseded_by_id IS
    'ADR-004 — id of the newer row that replaced this one during supersede-and-recompute.';

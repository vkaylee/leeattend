-- ============================================================================
-- ADR-005 §4: Legacy manual override classification
--
-- Adds columns for classifying HR-edited attendance rows from the legacy system.
-- Existing rows default to is_manual_override=false; rows flagged by the
-- migration script (scripts/adr005_migrate_manual_overrides.py) are classified
-- into:
--   Cat 1 — late approval / single-day absence → attendance_exceptions
--   Cat 2 — long-term adjustment (maternity, probation, special zone)
--           → employee_policy_modifiers
--   Cat 3 — unclassifiable → requires_hr_review = true
--
-- Cat 3 rows have a 90-day SLA for HR review (ADR-005 §4 Consequences).
-- ============================================================================

ALTER TABLE public.attendance_daily_summaries
    ADD COLUMN IF NOT EXISTS is_manual_override BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS manual_override_category SMALLINT,
    ADD COLUMN IF NOT EXISTS requires_hr_review BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS manual_override_notes TEXT;

COMMENT ON COLUMN public.attendance_daily_summaries.is_manual_override IS
    'ADR-005 §4 — true when this row was manually edited by HR in the legacy system.';
COMMENT ON COLUMN public.attendance_daily_summaries.manual_override_category IS
    'ADR-005 §4: 1=late_approval (→attendance_exceptions), 2=long_term_adjustment (→employee_policy_modifiers), 3=unclassifiable (requires_hr_review). NULL = not classified.';
COMMENT ON COLUMN public.attendance_daily_summaries.requires_hr_review IS
    'ADR-005 §4 — Cat 3 rows flagged for HR review within 90 days. Auto-flag hr_review_stale if past SLA.';
COMMENT ON COLUMN public.attendance_daily_summaries.manual_override_notes IS
    'ADR-005 §4 — HR notes or legacy comments associated with the manual edit.';

-- Partial index for Cat 3 review queue
CREATE INDEX IF NOT EXISTS idx_daily_summary_hr_review
    ON public.attendance_daily_summaries (workspace_id, date)
    WHERE requires_hr_review = true;

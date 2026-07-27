-- ============================================================================
-- ADR-001 §L3: attendance_exceptions
--
-- Ad-hoc single-day exception. Top of cascade — beats every layer below.
-- Typically 1 day (typhoon, HR discretionary approval), but a date range
-- is allowed for short events (e.g. "power outage 3 days").
--
-- Uniqueness: at most one exception per (employee_id, date). Enforced with
-- EXCLUDE on a daterange so range-typed rows still can't overlap for the
-- same employee.
-- ============================================================================

CREATE TABLE public.attendance_exceptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    employee_id UUID NOT NULL
        REFERENCES public.employees(id) ON DELETE CASCADE,
    policy_version_id UUID NOT NULL
        REFERENCES public.attendance_policy_versions(id),

    exception_from DATE NOT NULL,
    exception_to DATE NOT NULL,   -- inclusive; single-day => same as exception_from

    contributions JSONB NOT NULL DEFAULT '{}'::jsonb,

    reason TEXT NOT NULL,
    approved_by UUID,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_exception_range
        CHECK (exception_to >= exception_from),

    CONSTRAINT no_overlapping_exception
        EXCLUDE USING gist (
            employee_id WITH =,
            daterange(exception_from, exception_to, '[]') WITH &&
        )
);

CREATE INDEX idx_attendance_exception_workspace
    ON public.attendance_exceptions (workspace_id);
CREATE INDEX idx_attendance_exception_employee_range
    ON public.attendance_exceptions (employee_id, exception_from);
CREATE INDEX idx_attendance_exception_policy_version
    ON public.attendance_exceptions (policy_version_id);

COMMENT ON TABLE public.attendance_exceptions IS
    'ADR-001 L3 — ad-hoc single-day exception. Top of cascade, wins over every lower layer.';
COMMENT ON COLUMN public.attendance_exceptions.contributions IS
    'PartialPolicyRules JSON. Option<T> fields. Override values reset any delta chain from lower layers.';

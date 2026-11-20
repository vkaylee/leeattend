BEGIN;

ALTER TABLE public.attendance_adjustment_requests
    ADD COLUMN adjustment_type text NOT NULL DEFAULT 'attendance_correction',
    ADD COLUMN details jsonb NOT NULL DEFAULT '{}'::jsonb;

ALTER TABLE public.attendance_adjustment_requests
    ADD CONSTRAINT attendance_adjustment_requests_type_check
        CHECK (adjustment_type IN ('break_worked', 'missing_punch', 'attendance_correction')),
    ADD CONSTRAINT attendance_adjustment_requests_details_check
        CHECK (jsonb_typeof(details) = 'object');

CREATE INDEX attendance_adjustment_requests_type_idx
    ON public.attendance_adjustment_requests (adjustment_type, status, created_at DESC);

CREATE TABLE public.attendance_adjustment_evidence (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    adjustment_request_id uuid NOT NULL,
    employment_id uuid NOT NULL,
    business_date date NOT NULL,
    attendance_period_id uuid,
    closure_id uuid,
    before_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
    proposed_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
    reason text NOT NULL,
    created_by uuid NOT NULL,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT attendance_adjustment_evidence_request_fkey
        FOREIGN KEY (adjustment_request_id) REFERENCES public.attendance_adjustment_requests(id) ON DELETE RESTRICT,
    CONSTRAINT attendance_adjustment_evidence_employment_fkey
        FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT,
    CONSTRAINT attendance_adjustment_evidence_period_fkey
        FOREIGN KEY (attendance_period_id) REFERENCES public.attendance_periods(id) ON DELETE RESTRICT,
    CONSTRAINT attendance_adjustment_evidence_closure_fkey
        FOREIGN KEY (closure_id) REFERENCES public.attendance_closures(id) ON DELETE RESTRICT,
    CONSTRAINT attendance_adjustment_evidence_creator_fkey
        FOREIGN KEY (created_by) REFERENCES public.users(id) ON DELETE RESTRICT,
    CONSTRAINT attendance_adjustment_evidence_before_check
        CHECK (jsonb_typeof(before_snapshot) = 'object'),
    CONSTRAINT attendance_adjustment_evidence_proposed_check
        CHECK (jsonb_typeof(proposed_snapshot) = 'object'),
    CONSTRAINT attendance_adjustment_evidence_reason_check
        CHECK (length(btrim(reason)) BETWEEN 1 AND 2000)
);

CREATE UNIQUE INDEX attendance_adjustment_evidence_request_idx
    ON public.attendance_adjustment_evidence (adjustment_request_id);
CREATE INDEX attendance_adjustment_evidence_scope_idx
    ON public.attendance_adjustment_evidence (employment_id, business_date, created_at DESC);

INSERT INTO public.schema_migrations(version)
VALUES ('20261123000000_typed_attendance_adjustments');

COMMIT;

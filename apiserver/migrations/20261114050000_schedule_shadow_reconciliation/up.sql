BEGIN;

CREATE TABLE public.schedule_shadow_mismatches (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    employer_id uuid NOT NULL REFERENCES public.employers(id) ON DELETE RESTRICT,
    employment_id uuid NOT NULL REFERENCES public.employments(id) ON DELETE RESTRICT,
    business_date date NOT NULL,
    mismatch_kind varchar(32) NOT NULL,
    legacy_snapshot jsonb NOT NULL,
    resolver_snapshot jsonb NOT NULL,
    resolver_version varchar(32) NOT NULL,
    status varchar(16) NOT NULL DEFAULT 'open',
    first_seen_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_seen_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    resolved_at timestamptz,
    resolution_note text,
    CONSTRAINT schedule_shadow_mismatches_kind_check CHECK (
        mismatch_kind IN ('intervals', 'overnight', 'explicit_off', 'missing', 'effective_version', 'timezone', 'source', 'other')
    ),
    CONSTRAINT schedule_shadow_mismatches_status_check CHECK (status IN ('open', 'resolved', 'waived')),
    CONSTRAINT schedule_shadow_mismatches_snapshot_check CHECK (
        jsonb_typeof(legacy_snapshot) = 'object'
        AND jsonb_typeof(resolver_snapshot) = 'object'
    ),
    CONSTRAINT schedule_shadow_mismatches_resolution_check CHECK (
        (status = 'open' AND resolved_at IS NULL)
        OR (status IN ('resolved', 'waived') AND resolved_at IS NOT NULL)
    ),
    CONSTRAINT schedule_shadow_mismatches_dates_check CHECK (last_seen_at >= first_seen_at)
);

CREATE UNIQUE INDEX schedule_shadow_mismatches_open_key
    ON public.schedule_shadow_mismatches (employer_id, employment_id, business_date, mismatch_kind)
    WHERE status = 'open';
CREATE INDEX schedule_shadow_mismatches_employer_date_idx
    ON public.schedule_shadow_mismatches (employer_id, business_date, status);
CREATE INDEX schedule_shadow_mismatches_employment_date_idx
    ON public.schedule_shadow_mismatches (employment_id, business_date, last_seen_at DESC);

INSERT INTO public.schema_migrations(version)
VALUES ('20261114050000_schedule_shadow_reconciliation');

COMMIT;

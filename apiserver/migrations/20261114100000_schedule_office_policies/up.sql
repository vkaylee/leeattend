BEGIN;

CREATE TABLE public.schedule_office_policies (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    employer_id uuid NOT NULL REFERENCES public.employers(id) ON DELETE RESTRICT,
    approval_instance_id uuid NOT NULL,
    effective_from date NOT NULL,
    effective_to date,
    status varchar(16) NOT NULL DEFAULT 'active',
    created_by uuid REFERENCES public.users(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    revoked_at timestamptz,
    CONSTRAINT schedule_office_policies_status_check CHECK (status IN ('active', 'revoked')),
    CONSTRAINT schedule_office_policies_range_check CHECK (effective_to IS NULL OR effective_to >= effective_from),
    CONSTRAINT schedule_office_policies_revoked_check CHECK (
        (status = 'active' AND revoked_at IS NULL)
        OR (status = 'revoked' AND revoked_at IS NOT NULL)
    ),
    CONSTRAINT schedule_office_policies_approval_key UNIQUE (approval_instance_id)
);

ALTER TABLE public.schedule_office_policies
    ADD CONSTRAINT schedule_office_policies_effective_no_overlap
    EXCLUDE USING gist (
        employer_id WITH =,
        daterange(effective_from, COALESCE(effective_to + 1, 'infinity'::date), '[)') WITH &&
    )
    WHERE (status = 'active');

CREATE INDEX schedule_office_policies_employer_effective_idx
    ON public.schedule_office_policies (employer_id, effective_from, effective_to, status);
CREATE INDEX schedule_office_policies_approval_idx
    ON public.schedule_office_policies (approval_instance_id, status);

INSERT INTO public.schema_migrations(version)
VALUES ('20261114100000_schedule_office_policies');

COMMIT;

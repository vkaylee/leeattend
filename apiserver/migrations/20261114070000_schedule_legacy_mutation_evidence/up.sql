BEGIN;

CREATE TABLE public.schedule_legacy_mutation_evidence (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    employment_id uuid REFERENCES public.employments(id) ON DELETE RESTRICT,
    business_date date,
    operation varchar(32) NOT NULL,
    source varchar(32) NOT NULL DEFAULT 'database_trigger',
    details jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT schedule_legacy_mutation_evidence_operation_check CHECK (
        operation IN ('insert', 'update', 'delete', 'guard_blocked')
    ),
    CONSTRAINT schedule_legacy_mutation_evidence_details_check CHECK (jsonb_typeof(details) = 'object')
);

CREATE INDEX schedule_legacy_mutation_evidence_scope_idx
    ON public.schedule_legacy_mutation_evidence (employment_id, business_date, created_at DESC);

INSERT INTO public.schema_migrations(version)
VALUES ('20261114070000_schedule_legacy_mutation_evidence');

COMMIT;

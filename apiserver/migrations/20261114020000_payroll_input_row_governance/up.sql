BEGIN;

ALTER TABLE public.payroll_inputs
    ADD COLUMN status text NOT NULL DEFAULT 'accepted',
    ADD COLUMN validation_message text,
    ADD COLUMN corrected_at timestamptz,
    ADD COLUMN corrected_by uuid REFERENCES public.users(id);

ALTER TABLE public.payroll_inputs
    ADD CONSTRAINT payroll_inputs_status_check
    CHECK (status IN ('accepted', 'warning', 'rejected', 'corrected'));

CREATE TABLE public.payroll_input_corrections (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    input_id uuid NOT NULL REFERENCES public.payroll_inputs(id),
    old_value numeric(20,6) NOT NULL,
    new_value numeric(20,6) NOT NULL,
    old_unit text NOT NULL,
    new_unit text NOT NULL,
    reason text NOT NULL,
    corrected_by uuid NOT NULL REFERENCES public.users(id),
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT payroll_input_corrections_reason_check CHECK (length(btrim(reason)) BETWEEN 1 AND 500)
);

CREATE INDEX payroll_input_corrections_input_idx
    ON public.payroll_input_corrections (input_id, created_at DESC);

INSERT INTO public.schema_migrations(version)
VALUES ('20261114020000_payroll_input_row_governance');

COMMIT;

BEGIN;

CREATE UNIQUE INDEX payroll_calculation_runs_formula_payslip_unique
    ON public.payroll_calculation_runs (payslip_id, formula_version_id)
    WHERE payslip_id IS NOT NULL AND formula_version_id IS NOT NULL;

ALTER TABLE public.payroll_formula_rule_versions
    ADD CONSTRAINT payroll_formula_published_effective_no_overlap
    EXCLUDE USING gist (
        rule_id WITH =,
        daterange(effective_from, 'infinity'::date, '[)') WITH &&
    ) WHERE (status = 'published');

INSERT INTO public.schema_migrations(version)
VALUES ('20261114010000_payroll_calculation_run_idempotency');

COMMIT;

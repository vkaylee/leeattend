BEGIN;

ALTER TABLE public.payroll_formula_rule_versions
    ADD COLUMN retired_at timestamptz;

ALTER TABLE public.payroll_calculation_runs
    ADD COLUMN payslip_id uuid REFERENCES public.payslips(id);

CREATE INDEX payroll_calculation_runs_payslip_idx
    ON public.payroll_calculation_runs (payslip_id, created_at DESC);

INSERT INTO public.schema_migrations(version)
VALUES ('20261113050000_payroll_calculation_run_links');

COMMIT;

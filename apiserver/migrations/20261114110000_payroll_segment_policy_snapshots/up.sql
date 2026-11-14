BEGIN;

ALTER TABLE public.payslip_segments
    ADD COLUMN labor_policy_group_version_id uuid REFERENCES public.labor_policy_group_versions(id);

CREATE INDEX payslip_segments_labor_policy_version_idx
    ON public.payslip_segments (labor_policy_group_version_id)
    WHERE labor_policy_group_version_id IS NOT NULL;

INSERT INTO public.schema_migrations(version)
VALUES ('20261114110000_payroll_segment_policy_snapshots');

COMMIT;

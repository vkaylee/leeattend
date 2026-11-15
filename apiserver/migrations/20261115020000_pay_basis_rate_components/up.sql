BEGIN;

ALTER TABLE public.pay_basis_policies
    ADD COLUMN rate_component_definition_id uuid
        REFERENCES public.salary_component_definitions(id);

ALTER TABLE public.pay_basis_policies
    ADD CONSTRAINT pay_basis_policies_rate_component_check CHECK (
        (method = 'monthly' AND rate_component_definition_id IS NULL)
        OR (method IN ('hourly', 'shift') AND rate_component_definition_id IS NOT NULL)
    );

INSERT INTO public.schema_migrations(version)
VALUES ('20261115020000_pay_basis_rate_components');

COMMIT;

BEGIN;

ALTER TABLE public.pay_basis_policies
    DROP CONSTRAINT pay_basis_policies_rate_component_check;

ALTER TABLE public.pay_basis_policies
    DROP COLUMN rate_component_definition_id;

DELETE FROM public.schema_migrations
WHERE version = '20261115020000_pay_basis_rate_components';

COMMIT;

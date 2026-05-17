-- Add down migration script here
DROP TABLE IF EXISTS public.employee_contracts;

DROP TYPE IF EXISTS public.contract_status_enum;

DROP TYPE IF EXISTS public.contract_type_enum;
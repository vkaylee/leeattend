ALTER TABLE public.salary_component_definitions
    DROP COLUMN IF EXISTS is_taxable,
    DROP COLUMN IF EXISTS is_base_for_ot,
    DROP COLUMN IF EXISTS is_insurance_base;

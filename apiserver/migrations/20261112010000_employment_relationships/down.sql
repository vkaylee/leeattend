BEGIN;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM public.employments WHERE relationship_id IS NOT NULL)
       OR EXISTS (SELECT 1 FROM public.employee_contracts WHERE relationship_id IS NOT NULL)
       OR EXISTS (SELECT 1 FROM public.employment_relationships) THEN
        RAISE EXCEPTION 'cannot rollback employment relationships: relationship data is present; restore backup or roll forward';
    END IF;
END
$$;

DROP TRIGGER IF EXISTS trg_validate_segment_relationship_change ON public.employments;
DROP TRIGGER IF EXISTS trg_validate_contract_relationship_link ON public.employee_contracts;
DROP FUNCTION IF EXISTS public.fn_validate_segment_relationship_change();
DROP FUNCTION IF EXISTS public.fn_validate_contract_relationship_link();

DROP INDEX IF EXISTS public.idx_employee_contracts_relationship_id;
DROP INDEX IF EXISTS public.idx_employments_relationship_id;

ALTER TABLE ONLY public.employments
    DROP CONSTRAINT IF EXISTS employments_relationship_identity_fkey;
ALTER TABLE ONLY public.employee_contracts
    DROP CONSTRAINT IF EXISTS employee_contracts_relationship_id_fkey;
ALTER TABLE ONLY public.employments
    DROP CONSTRAINT IF EXISTS employments_relationship_id_fkey;

ALTER TABLE ONLY public.employee_contracts
    DROP COLUMN IF EXISTS relationship_id;
ALTER TABLE ONLY public.employments
    DROP COLUMN IF EXISTS relationship_id;

DROP TRIGGER IF EXISTS trg_updated_at ON public.employment_relationships;
DROP INDEX IF EXISTS public.idx_employment_relationships_employer;
DROP INDEX IF EXISTS public.idx_employment_relationships_employee;
DROP TABLE IF EXISTS public.employment_relationships;

DELETE FROM public.schema_migrations
WHERE version = '20261112010000_employment_relationships';

COMMIT;

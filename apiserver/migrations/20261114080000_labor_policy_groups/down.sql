BEGIN;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM public.labor_policy_memberships) THEN
        RAISE EXCEPTION 'labor policy groups rollback blocked: memberships exist';
    END IF;
    IF EXISTS (SELECT 1 FROM public.labor_policy_group_versions) THEN
        RAISE EXCEPTION 'labor policy groups rollback blocked: group versions exist';
    END IF;
    IF EXISTS (SELECT 1 FROM public.labor_policy_groups) THEN
        RAISE EXCEPTION 'labor policy groups rollback blocked: groups exist';
    END IF;
    IF EXISTS (SELECT 1 FROM public.work_credit_policies) OR EXISTS (SELECT 1 FROM public.pay_basis_policies) THEN
        RAISE EXCEPTION 'labor policy groups rollback blocked: policy definitions exist';
    END IF;
END;
$$;

DROP TABLE public.labor_policy_memberships;
DROP TABLE public.labor_policy_group_versions;
DROP TABLE public.labor_policy_groups;
DROP TABLE public.pay_basis_policies;
DROP TABLE public.work_credit_policies;

DELETE FROM public.schema_migrations
WHERE version = '20261114080000_labor_policy_groups';

COMMIT;

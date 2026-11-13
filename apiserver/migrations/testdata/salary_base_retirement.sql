\set ON_ERROR_STOP on
-- Run only on a disposable database with the baseline applied, before retirement.
-- The fixture commits because the migration scripts own their transactions.
BEGIN;
INSERT INTO public.employers (id, name)
VALUES ('e0000000-0000-0000-0000-000000000001', 'Retirement fixture employer');
INSERT INTO public.employees (id, employee_id_number, full_name, hire_date, original_hire_date)
VALUES ('e0000000-0000-0000-0000-000000000002', 'retirement-fixture', 'Retirement fixture', '2026-01-01', '2026-01-01');
INSERT INTO public.employments (id, employee_id, employer_id, status, start_date)
VALUES ('e0000000-0000-0000-0000-000000000003', 'e0000000-0000-0000-0000-000000000002', 'e0000000-0000-0000-0000-000000000001', 'active', '2026-01-01');
INSERT INTO public.salary_component_definitions (id, name, code, component_type)
VALUES ('e0000000-0000-0000-0000-000000000004', 'Retained allowance', 'RETIREMENT_FIXTURE', 'earning');
INSERT INTO public.employee_salary_components (component_definition_id, employment_id, amount, effective_date)
VALUES
    ('e0000000-0000-0000-0000-000000000004', 'e0000000-0000-0000-0000-000000000003', 12345678.90, '2026-01-01'),
    ('e0000000-0000-0000-0000-000000000004', 'e0000000-0000-0000-0000-000000000003', 23456789.01, '2026-02-01');
INSERT INTO public.salary_base_groups (id, name, slug, description, is_system, created_at, updated_at)
VALUES
    ('e0000000-0000-0000-0000-000000000005', 'Gross fixture', 'retirement_gross', 'System gross', true, '2026-01-01', '2026-02-01'),
    ('e0000000-0000-0000-0000-000000000006', 'NET fixture', 'retirement_net', NULL, true, '2026-01-02', '2026-02-02'),
    ('e0000000-0000-0000-0000-000000000007', 'Custom fixture', 'retirement_custom', 'Non-reconstructable custom metadata', false, '2026-01-03', '2026-02-03');
INSERT INTO public.salary_component_base_members (component_definition_id, base_group_id, created_at)
SELECT 'e0000000-0000-0000-0000-000000000004', id, '2026-03-01'::timestamptz
FROM public.salary_base_groups
WHERE id IN ('e0000000-0000-0000-0000-000000000005', 'e0000000-0000-0000-0000-000000000006', 'e0000000-0000-0000-0000-000000000007');
CREATE TEMP TABLE expected_definitions AS TABLE public.salary_component_definitions;
CREATE TEMP TABLE expected_amounts AS TABLE public.employee_salary_components;
CREATE TEMP TABLE expected_groups AS TABLE public.salary_base_groups;
CREATE TEMP TABLE expected_members AS TABLE public.salary_component_base_members;
COMMIT;

\ir ../20261113020000_retire_salary_base_groups/up.sql

DO $$
BEGIN
    IF to_regclass('public.salary_base_groups') IS NOT NULL
       OR to_regclass('public.salary_component_base_members') IS NOT NULL THEN
        RAISE EXCEPTION 'retired salary tables still exist';
    END IF;
    IF EXISTS ((TABLE expected_definitions EXCEPT TABLE public.salary_component_definitions)
               UNION ALL (TABLE public.salary_component_definitions EXCEPT TABLE expected_definitions))
       OR EXISTS ((TABLE expected_amounts EXCEPT TABLE public.employee_salary_components)
                  UNION ALL (TABLE public.employee_salary_components EXCEPT TABLE expected_amounts)) THEN
        RAISE EXCEPTION 'retirement changed component definitions or assigned amounts';
    END IF;
    IF EXISTS ((TABLE expected_groups EXCEPT TABLE salary_base_retirement_backup.groups)
               UNION ALL (TABLE salary_base_retirement_backup.groups EXCEPT TABLE expected_groups))
       OR EXISTS ((TABLE expected_members EXCEPT TABLE salary_base_retirement_backup.members)
                  UNION ALL (TABLE salary_base_retirement_backup.members EXCEPT TABLE expected_members)) THEN
        RAISE EXCEPTION 'retirement snapshot lost group or member data';
    END IF;
END $$;

-- Retained writes must continue without either group table.
INSERT INTO public.salary_component_definitions (id, name, code, component_type)
VALUES ('e0000000-0000-0000-0000-000000000008', 'Post-retirement allowance', 'POST_RETIREMENT', 'earning');
INSERT INTO public.employee_salary_components (component_definition_id, employment_id, amount, effective_date)
VALUES ('e0000000-0000-0000-0000-000000000008', 'e0000000-0000-0000-0000-000000000003', 34567890.12, '2026-03-01');
INSERT INTO expected_definitions SELECT * FROM public.salary_component_definitions WHERE code = 'POST_RETIREMENT';
INSERT INTO expected_amounts SELECT * FROM public.employee_salary_components WHERE component_definition_id = 'e0000000-0000-0000-0000-000000000008';

\ir ../20261113020000_retire_salary_base_groups/down.sql

DO $$
BEGIN
    IF EXISTS ((TABLE expected_groups EXCEPT TABLE public.salary_base_groups)
               UNION ALL (TABLE public.salary_base_groups EXCEPT TABLE expected_groups))
       OR EXISTS ((TABLE expected_members EXCEPT TABLE public.salary_component_base_members)
                  UNION ALL (TABLE public.salary_component_base_members EXCEPT TABLE expected_members)) THEN
        RAISE EXCEPTION 'rollback did not restore exact group/member data';
    END IF;
    IF EXISTS ((TABLE expected_definitions EXCEPT TABLE public.salary_component_definitions)
               UNION ALL (TABLE public.salary_component_definitions EXCEPT TABLE expected_definitions))
       OR EXISTS ((TABLE expected_amounts EXCEPT TABLE public.employee_salary_components)
                  UNION ALL (TABLE public.employee_salary_components EXCEPT TABLE expected_amounts)) THEN
        RAISE EXCEPTION 'rollback changed retained payroll data';
    END IF;
    IF EXISTS (SELECT 1 FROM public.schema_migrations WHERE version = '20261113020000_retire_salary_base_groups')
       OR to_regnamespace('salary_base_retirement_backup') IS NOT NULL THEN
        RAISE EXCEPTION 'rollback left migration marker or snapshots';
    END IF;
END $$;

-- Verify snapshots can be recreated after a successful rollback.
\ir ../20261113020000_retire_salary_base_groups/up.sql
\ir ../20261113020000_retire_salary_base_groups/down.sql

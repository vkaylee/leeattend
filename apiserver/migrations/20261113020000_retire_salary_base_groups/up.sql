BEGIN;

-- Contract migration: deploy only after all salary-base API consumers are retired.
-- Keep an exact rollback snapshot; payroll components and assigned amounts are untouched.
LOCK TABLE public.salary_base_groups, public.salary_component_base_members IN ACCESS EXCLUSIVE MODE;
CREATE SCHEMA salary_base_retirement_backup;
REVOKE ALL ON SCHEMA salary_base_retirement_backup FROM PUBLIC;
CREATE TABLE salary_base_retirement_backup.groups
    (LIKE public.salary_base_groups INCLUDING ALL);
CREATE TABLE salary_base_retirement_backup.members
    (LIKE public.salary_component_base_members INCLUDING ALL);
INSERT INTO salary_base_retirement_backup.groups SELECT * FROM public.salary_base_groups;
INSERT INTO salary_base_retirement_backup.members SELECT * FROM public.salary_component_base_members;
REVOKE ALL ON ALL TABLES IN SCHEMA salary_base_retirement_backup FROM PUBLIC;

-- No CASCADE: unexpected dependencies abort the whole migration, including snapshots.
DROP TABLE public.salary_component_base_members;
DROP TABLE public.salary_base_groups;

INSERT INTO public.schema_migrations(version)
VALUES ('20261113020000_retire_salary_base_groups');

COMMIT;

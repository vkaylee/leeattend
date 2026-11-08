BEGIN;
INSERT INTO public.schema_migrations(version)
VALUES ('20261108000000_single_organization_cutover');
\ir ../20261108000000_single_organization_cutover/up.sql

BEGIN;
INSERT INTO public.schema_migrations(version)
VALUES ('20261106000000_remove_system_roles');
\ir ../fresh_baseline.sql

BEGIN;
INSERT INTO public.schema_migrations(version)
VALUES ('20261103000000_init');
\ir ../20261103000000_init/up.sql
COMMIT;

BEGIN;

DELETE FROM public.feature_flags
WHERE flag_key = 'ff_econtracts';

DELETE FROM public.schema_migrations
WHERE version = '20261113010000_feature_flags_runtime';

COMMIT;

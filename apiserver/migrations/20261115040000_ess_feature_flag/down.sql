BEGIN;

DELETE FROM public.feature_flags
WHERE flag_key = 'ff_ess';

DELETE FROM public.schema_migrations
WHERE version = '20261115040000_ess_feature_flag';

COMMIT;

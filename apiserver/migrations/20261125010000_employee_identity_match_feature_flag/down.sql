BEGIN;

DELETE FROM public.feature_flags
WHERE flag_key = 'ff_employee_identity_match';

DELETE FROM public.schema_migrations
WHERE version = '20261125010000_employee_identity_match_feature_flag';

COMMIT;

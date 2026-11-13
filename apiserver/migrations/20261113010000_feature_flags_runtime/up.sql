BEGIN;

INSERT INTO public.feature_flags (flag_key, enabled, description, metadata)
VALUES (
    'ff_econtracts',
    false,
    'Enable the e-Contracts workflow and settings.',
    '{"owner":"platform","removal_target":"2026-12-13"}'::jsonb
)
ON CONFLICT (flag_key) DO NOTHING;

INSERT INTO public.schema_migrations(version)
VALUES ('20261113010000_feature_flags_runtime');

COMMIT;

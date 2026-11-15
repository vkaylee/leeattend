BEGIN;

INSERT INTO public.feature_flags (flag_key, enabled, description, metadata)
VALUES (
    'ff_ess',
    false,
    'Enable Employee Self-Service workflows and routes.',
    '{"owner":"platform","created":"2026-11-15","removal_target":"2026-12-15"}'::jsonb
)
ON CONFLICT (flag_key) DO NOTHING;

INSERT INTO public.schema_migrations(version)
VALUES ('20261115040000_ess_feature_flag');

COMMIT;

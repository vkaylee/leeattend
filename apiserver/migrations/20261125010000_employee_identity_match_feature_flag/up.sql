BEGIN;

INSERT INTO public.feature_flags (flag_key, enabled, description, metadata)
VALUES (
    'ff_employee_identity_match',
    false,
    'Enable employee identity preflight matching and explicit tenure reuse.',
    '{"owner":"platform","created":"2026-11-25","removal_target":"2026-12-25","pilot_roles":["admin","hr_manager"]}'::jsonb
)
ON CONFLICT (flag_key) DO NOTHING;

INSERT INTO public.schema_migrations(version)
VALUES ('20261125010000_employee_identity_match_feature_flag');

COMMIT;

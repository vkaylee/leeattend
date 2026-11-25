-- Fresh-install runtime seeds.
-- Run after all four schema phases. Idempotent. No user data or secrets.
-- Runner must provide psql variable: --set=seed_effective_date=YYYY-MM-DD.
BEGIN;

INSERT INTO public.feature_flags (flag_key, enabled, description, metadata)
VALUES
    ('ff_econtracts', false, 'Enable the e-Contracts workflow and settings.', '{"owner":"platform","removal_target":"2026-12-13"}'::jsonb),
    ('ff_ess', false, 'Enable Employee Self-Service workflows and routes.', '{"owner":"platform","created":"2026-11-15","removal_target":"2026-12-15"}'::jsonb),
    ('ff_employee_identity_match', false, 'Enable employee identity preflight matching and explicit tenure reuse.', '{"owner":"platform","created":"2026-11-25","removal_target":"2026-12-25","pilot_roles":["admin","hr_manager"]}'::jsonb)
ON CONFLICT (flag_key) DO NOTHING;

INSERT INTO public.payroll_metric_definitions (code, name, unit, scope, source)
VALUES
    ('production.output', 'Production output', 'product', 'employee', 'production'),
    ('production.target_output', 'Production target output', 'product', 'employee', 'production'),
    ('production.quality_rate', 'Production quality rate', 'ratio', 'employee', 'production')
ON CONFLICT (code) DO NOTHING;

-- Config defaults use installation date as effective date. Existing callers
-- resolve latest value at or before requested date. Never delete later edits
-- during rollback; fresh-install rollback recreates the disposable database.
INSERT INTO public.config_entries (scope_type, scope_id, config_key, value_decimal, effective_from, created_at, updated_at)
VALUES
    ('organization', NULL, 'payroll.kpi.output_weight', 0.6000, :'seed_effective_date'::date, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    ('organization', NULL, 'payroll.kpi.quality_weight', 0.4000, :'seed_effective_date'::date, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    ('organization', NULL, 'payroll.kpi.target_bonus', 0.0000, :'seed_effective_date'::date, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    ('organization', NULL, 'payroll.kpi.score_cap', 1.2000, :'seed_effective_date'::date, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    ('organization', NULL, 'payroll.kpi.bonus_cap', 0.0000, :'seed_effective_date'::date, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT (scope_type, scope_id, config_key, effective_from) DO NOTHING;

-- KPI formula is intentionally absent: standalone bootstrap creates it after
-- administrator exists, using that administrator as created_by/published_by.

INSERT INTO public.schema_migrations(version)
VALUES ('fresh_runtime_seeds_v1');

COMMIT;

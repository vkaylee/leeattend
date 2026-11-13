BEGIN;

INSERT INTO public.config_entries (scope_type, scope_id, config_key, value_decimal, effective_from, created_at, updated_at)
VALUES
    ('organization', NULL, 'payroll.kpi.output_weight', 0.6000, CURRENT_DATE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    ('organization', NULL, 'payroll.kpi.quality_weight', 0.4000, CURRENT_DATE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    ('organization', NULL, 'payroll.kpi.target_bonus', 0.0000, CURRENT_DATE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    ('organization', NULL, 'payroll.kpi.score_cap', 1.2000, CURRENT_DATE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    ('organization', NULL, 'payroll.kpi.bonus_cap', 0.0000, CURRENT_DATE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT (scope_type, scope_id, config_key, effective_from) DO NOTHING;

INSERT INTO public.schema_migrations(version)
VALUES ('20261114000000_payroll_policy_parameters');

COMMIT;

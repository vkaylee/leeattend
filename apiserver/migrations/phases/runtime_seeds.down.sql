-- Fresh-install-only runtime seed rollback. Do not run against production data.
BEGIN;

DELETE FROM public.schema_migrations
WHERE version = 'fresh_runtime_seeds_v1';

DELETE FROM public.feature_flags
WHERE flag_key IN ('ff_econtracts', 'ff_ess', 'ff_employee_identity_match');

DELETE FROM public.payroll_metric_definitions
WHERE code IN ('production.output', 'production.target_output', 'production.quality_rate');

DELETE FROM public.config_entries
WHERE scope_type = 'organization'
  AND scope_id IS NULL
  AND config_key IN (
      'payroll.kpi.output_weight',
      'payroll.kpi.quality_weight',
      'payroll.kpi.target_bonus',
      'payroll.kpi.score_cap',
      'payroll.kpi.bonus_cap'
  );

COMMIT;

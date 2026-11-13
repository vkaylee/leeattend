BEGIN;

DELETE FROM public.config_entries
WHERE scope_type = 'organization'
  AND scope_id IS NULL
  AND config_key IN ('payroll.kpi.output_weight', 'payroll.kpi.quality_weight', 'payroll.kpi.target_bonus', 'payroll.kpi.score_cap', 'payroll.kpi.bonus_cap')
  AND effective_from = CURRENT_DATE;

DELETE FROM public.schema_migrations
WHERE version = '20261114000000_payroll_policy_parameters';

COMMIT;

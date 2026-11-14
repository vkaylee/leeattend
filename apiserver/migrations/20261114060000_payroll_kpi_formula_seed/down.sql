BEGIN;

DELETE FROM public.schema_migrations
WHERE version = '20261114060000_payroll_kpi_formula_seed';

DELETE FROM public.payroll_formula_rule_versions
WHERE rule_id = (SELECT id FROM public.payroll_formula_rules WHERE code = 'kpi.bonus')
  AND version_number = 1
  AND expression = 'min(target_bonus * (min(actual / target, score_cap) * output_weight + quality * quality_weight), bonus_cap)'
  AND NOT EXISTS (
      SELECT 1
      FROM public.payroll_calculation_runs cr
      WHERE cr.formula_version_id = payroll_formula_rule_versions.id
  );

DELETE FROM public.payroll_formula_rules
WHERE code = 'kpi.bonus'
  AND NOT EXISTS (
      SELECT 1
      FROM public.payroll_formula_rule_versions v
      WHERE v.rule_id = payroll_formula_rules.id
  );

COMMIT;

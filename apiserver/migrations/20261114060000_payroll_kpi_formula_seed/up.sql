BEGIN;

INSERT INTO public.payroll_formula_rules (code, name, component_code, created_by)
SELECT 'kpi.bonus', 'KPI Bonus', 'KPI_BONUS', u.id
FROM public.users u
ORDER BY u.id
LIMIT 1
ON CONFLICT (code) DO NOTHING;

INSERT INTO public.payroll_formula_rule_versions (
    rule_id,
    version_number,
    expression,
    effective_from,
    status,
    created_by,
    published_by,
    published_at
)
SELECT
    r.id,
    1,
    'min(target_bonus * (min(actual / target, score_cap) * output_weight + quality * quality_weight), bonus_cap)',
    CURRENT_DATE,
    'published',
    r.created_by,
    r.created_by,
    CURRENT_TIMESTAMP
FROM public.payroll_formula_rules r
WHERE r.code = 'kpi.bonus'
  AND NOT EXISTS (
      SELECT 1
      FROM public.payroll_formula_rule_versions v
      WHERE v.rule_id = r.id
  );

INSERT INTO public.schema_migrations(version)
VALUES ('20261114060000_payroll_kpi_formula_seed');

COMMIT;

BEGIN;

CREATE VIEW public.department_payroll_reconciliation AS
SELECT
    d.id AS department_id,
    d.name AS department_name,
    d.std_working_days AS legacy_std_working_days,
    latest.effective_from AS canonical_effective_from,
    latest.value_decimal AS canonical_std_working_days,
    CASE
        WHEN d.std_working_days IS NULL THEN 'missing_legacy_value'
        WHEN latest.department_id IS NULL THEN 'missing_canonical_config'
        WHEN latest.value_decimal = d.std_working_days THEN 'matched'
        ELSE 'value_mismatch'
    END AS reconciliation_status
FROM public.departments d
LEFT JOIN LATERAL (
    SELECT ce.scope_id AS department_id, ce.effective_from, ce.value_decimal
    FROM public.config_entries ce
    WHERE ce.scope_type = 'department'
      AND ce.scope_id = d.id
      AND ce.config_key = 'payroll.fixed_std_days'
    ORDER BY ce.effective_from DESC, ce.created_at DESC, ce.id DESC
    LIMIT 1
) latest ON TRUE
WHERE d.deleted_at IS NULL;

INSERT INTO public.schema_migrations(version)
VALUES ('20261115010000_department_payroll_reconciliation');

COMMIT;

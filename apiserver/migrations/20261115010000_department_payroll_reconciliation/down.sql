BEGIN;

DROP VIEW public.department_payroll_reconciliation;

DELETE FROM public.schema_migrations
WHERE version = '20261115010000_department_payroll_reconciliation';

COMMIT;

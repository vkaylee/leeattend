BEGIN;

DELETE FROM public.schema_migrations
WHERE version = '20261114020000_payroll_input_row_governance';

DROP INDEX IF EXISTS public.payroll_input_corrections_input_idx;
DROP TABLE IF EXISTS public.payroll_input_corrections;
ALTER TABLE public.payroll_inputs DROP CONSTRAINT IF EXISTS payroll_inputs_status_check;
ALTER TABLE public.payroll_inputs
    DROP COLUMN IF EXISTS status,
    DROP COLUMN IF EXISTS validation_message,
    DROP COLUMN IF EXISTS corrected_at,
    DROP COLUMN IF EXISTS corrected_by;

COMMIT;

BEGIN;

-- Fresh-install-only rollback for payroll; run phases in reverse order.

-- This is destructive and must not run against production data.

DROP VIEW IF EXISTS public.department_payroll_reconciliation;

DROP TABLE IF EXISTS public.work_credit_policies CASCADE;

DROP TABLE IF EXISTS public.payslips CASCADE;

DROP TABLE IF EXISTS public.payslip_segments CASCADE;

DROP TABLE IF EXISTS public.payslip_rule_evaluations CASCADE;

DROP TABLE IF EXISTS public.payslip_incentives CASCADE;

DROP TABLE IF EXISTS public.payroll_rule_tiers CASCADE;

DROP TABLE IF EXISTS public.payroll_rule_definitions CASCADE;

DROP TABLE IF EXISTS public.payroll_rule_assignments CASCADE;

DROP TABLE IF EXISTS public.payroll_metric_definitions CASCADE;

DROP TABLE IF EXISTS public.payroll_inputs CASCADE;

DROP TABLE IF EXISTS public.payroll_input_corrections CASCADE;

DROP TABLE IF EXISTS public.payroll_input_batches CASCADE;

DROP TABLE IF EXISTS public.payroll_incentive_tiers CASCADE;

DROP TABLE IF EXISTS public.payroll_incentive_policies CASCADE;

DROP TABLE IF EXISTS public.payroll_incentive_assignments CASCADE;

DROP TABLE IF EXISTS public.payroll_formula_rules CASCADE;

DROP TABLE IF EXISTS public.payroll_formula_rule_versions CASCADE;

DROP TABLE IF EXISTS public.payroll_calculation_runs CASCADE;

DROP TABLE IF EXISTS public.pay_basis_policies CASCADE;

DROP TABLE IF EXISTS public.labor_policy_memberships CASCADE;

DROP TABLE IF EXISTS public.labor_policy_groups CASCADE;

DROP TABLE IF EXISTS public.labor_policy_group_versions CASCADE;

DELETE FROM public.schema_migrations WHERE version = 'fresh_payroll_v1';

COMMIT;

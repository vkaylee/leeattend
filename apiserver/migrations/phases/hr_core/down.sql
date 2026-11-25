BEGIN;

-- Fresh-install-only rollback for hr_core; run phases in reverse order.

-- This is destructive and must not run against production data.

DROP TABLE IF EXISTS public.salary_component_definitions CASCADE;

DROP TABLE IF EXISTS public.recruitment_sources CASCADE;

DROP TABLE IF EXISTS public.overtime_rules CASCADE;

DROP TABLE IF EXISTS public.overtime_requests CASCADE;

DROP TABLE IF EXISTS public.overtime_request_employees CASCADE;

DROP TABLE IF EXISTS public.overtime_policy_definitions CASCADE;

DROP TABLE IF EXISTS public.overtime_policy_assignments CASCADE;

DROP TABLE IF EXISTS public.leave_types CASCADE;

DROP TABLE IF EXISTS public.leave_requests CASCADE;

DROP TABLE IF EXISTS public.leave_policy_definitions CASCADE;

DROP TABLE IF EXISTS public.leave_policy_assignments CASCADE;

DROP TABLE IF EXISTS public.leave_balances CASCADE;

DROP TABLE IF EXISTS public.leave_accrual_logs CASCADE;

DROP TABLE IF EXISTS public.employments CASCADE;

DROP TABLE IF EXISTS public.employment_relationships CASCADE;

DROP TABLE IF EXISTS public.employment_phase_definitions CASCADE;

DROP TABLE IF EXISTS public.employers CASCADE;

DROP TABLE IF EXISTS public.employer_econtract_routing_rules CASCADE;

DROP TABLE IF EXISTS public.employer_econtract_configs CASCADE;

DROP TABLE IF EXISTS public.employees CASCADE;

DROP TABLE IF EXISTS public.employee_tenures CASCADE;

DROP TABLE IF EXISTS public.employee_salary_components CASCADE;

DROP TABLE IF EXISTS public.employee_profiles CASCADE;

DROP TABLE IF EXISTS public.employee_phases CASCADE;

DROP TABLE IF EXISTS public.employee_emergency_contacts CASCADE;

DROP TABLE IF EXISTS public.employee_documents CASCADE;

DROP TABLE IF EXISTS public.employee_contracts CASCADE;

DROP TABLE IF EXISTS public.employee_assignments CASCADE;

DROP TABLE IF EXISTS public.econtract_envelopes CASCADE;

DROP TABLE IF EXISTS public.departments CASCADE;

DELETE FROM public.schema_migrations WHERE version = 'fresh_hr_core_v1';

COMMIT;

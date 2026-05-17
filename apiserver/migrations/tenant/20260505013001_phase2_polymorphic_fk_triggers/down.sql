-- ═══════════════════════════════════════════════════════════════════
-- Rollback: Phase 2 polymorphic FK triggers
-- ═══════════════════════════════════════════════════════════════════

-- Drop supporting indexes
DROP INDEX IF EXISTS public.idx_attendance_policy_assignments_target;

DROP INDEX IF EXISTS public.idx_overtime_policy_assignments_assignee;

DROP INDEX IF EXISTS public.idx_leave_policy_assignments_assignee;

DROP INDEX IF EXISTS public.idx_payroll_incentive_assignments_assignee;

DROP INDEX IF EXISTS public.idx_payroll_rule_assignments_assignee;

-- Drop cascade triggers on parent tables
DROP TRIGGER IF EXISTS trg_cascade_delete_emp_assignments ON public.employees;

DROP TRIGGER IF EXISTS trg_cascade_delete_dept_assignments ON public.departments;

-- Drop validation triggers on assignment tables
DROP TRIGGER IF EXISTS trg_validate_target_fk ON public.attendance_policy_assignments;

DROP TRIGGER IF EXISTS trg_validate_assignee_fk ON public.overtime_policy_assignments;

DROP TRIGGER IF EXISTS trg_validate_assignee_fk ON public.leave_policy_assignments;

DROP TRIGGER IF EXISTS trg_validate_assignee_fk ON public.payroll_incentive_assignments;

DROP TRIGGER IF EXISTS trg_validate_assignee_fk ON public.payroll_rule_assignments;

-- Drop functions
DROP FUNCTION IF EXISTS public.fn_cascade_delete_employee_assignments();

DROP FUNCTION IF EXISTS public.fn_cascade_delete_department_assignments();

DROP FUNCTION IF EXISTS public.fn_validate_target_fk();

DROP FUNCTION IF EXISTS public.fn_validate_assignee_fk();
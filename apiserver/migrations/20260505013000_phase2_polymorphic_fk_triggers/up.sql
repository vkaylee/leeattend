-- ═══════════════════════════════════════════════════════════════════
-- Phase 2: Polymorphic FK Integrity via Triggers
--
-- Problem: assignee_type/assignee_id cannot have traditional FKs
--          because they reference different tables based on type.
--          This means orphaned records can appear when a department
--          or employee is deleted.
--
-- Solution: Two trigger layers:
--   Layer 1: VALIDATE on INSERT/UPDATE — reject invalid assignee_id
--   Layer 2: CASCADE on DELETE from parent — clean up assignments
--
-- Risk: ZERO application code changes required
-- ═══════════════════════════════════════════════════════════════════

-- ───────────────────────────────────────────────────────────────────
-- Layer 1: Validation trigger for assignee_type/assignee_id tables
-- Reusable function that validates the FK based on assignee_type
-- ───────────────────────────────────────────────────────────────────
CREATE OR REPLACE FUNCTION public.fn_validate_assignee_fk()
RETURNS TRIGGER AS $$
BEGIN
    -- 'workspace' type uses workspace_id (already has real FK), no extra check
    IF NEW.assignee_type = 'workspace' THEN
        RETURN NEW;
    END IF;

    -- assignee_id must not be null for department/employee
    IF NEW.assignee_id IS NULL THEN
        RAISE EXCEPTION 'assignee_id cannot be NULL when assignee_type is %', NEW.assignee_type;
    END IF;

    IF NEW.assignee_type = 'department' THEN
        IF NOT EXISTS (SELECT 1 FROM public.departments WHERE id = NEW.assignee_id) THEN
            RAISE EXCEPTION 'Invalid assignee_id: department % does not exist', NEW.assignee_id;
        END IF;
    ELSIF NEW.assignee_type = 'employee' THEN
        IF NOT EXISTS (SELECT 1 FROM public.employees WHERE id = NEW.assignee_id) THEN
            RAISE EXCEPTION 'Invalid assignee_id: employee % does not exist', NEW.assignee_id;
        END IF;
    ELSE
        RAISE EXCEPTION 'Unknown assignee_type: %', NEW.assignee_type;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ───────────────────────────────────────────────────────────────────
-- Layer 1b: Validation trigger for target_type/target_id tables
-- (attendance_policy_assignments uses different column names)
-- ───────────────────────────────────────────────────────────────────
CREATE OR REPLACE FUNCTION public.fn_validate_target_fk()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.target_id IS NULL THEN
        RAISE EXCEPTION 'target_id cannot be NULL';
    END IF;

    IF NEW.target_type = 'USER' THEN
        IF NOT EXISTS (SELECT 1 FROM public.employees WHERE id = NEW.target_id) THEN
            RAISE EXCEPTION 'Invalid target_id: employee % does not exist', NEW.target_id;
        END IF;
    ELSIF NEW.target_type = 'GROUP' THEN
        IF NOT EXISTS (SELECT 1 FROM public.departments WHERE id = NEW.target_id) THEN
            RAISE EXCEPTION 'Invalid target_id: department % does not exist', NEW.target_id;
        END IF;
    ELSE
        RAISE EXCEPTION 'Unknown target_type: %', NEW.target_type;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ───────────────────────────────────────────────────────────────────
-- Apply validation triggers to all 5 assignment tables
-- ───────────────────────────────────────────────────────────────────

-- payroll_rule_assignments
CREATE TRIGGER trg_validate_assignee_fk
    BEFORE INSERT OR UPDATE OF assignee_type, assignee_id
    ON public.payroll_rule_assignments
    FOR EACH ROW EXECUTE FUNCTION public.fn_validate_assignee_fk();

-- payroll_incentive_assignments
CREATE TRIGGER trg_validate_assignee_fk
    BEFORE INSERT OR UPDATE OF assignee_type, assignee_id
    ON public.payroll_incentive_assignments
    FOR EACH ROW EXECUTE FUNCTION public.fn_validate_assignee_fk();

-- leave_policy_assignments
CREATE TRIGGER trg_validate_assignee_fk
    BEFORE INSERT OR UPDATE OF assignee_type, assignee_id
    ON public.leave_policy_assignments
    FOR EACH ROW EXECUTE FUNCTION public.fn_validate_assignee_fk();

-- overtime_policy_assignments
CREATE TRIGGER trg_validate_assignee_fk
    BEFORE INSERT OR UPDATE OF assignee_type, assignee_id
    ON public.overtime_policy_assignments
    FOR EACH ROW EXECUTE FUNCTION public.fn_validate_assignee_fk();

-- attendance_policy_assignments (uses target_type/target_id)
CREATE TRIGGER trg_validate_target_fk
    BEFORE INSERT OR UPDATE OF target_type, target_id
    ON public.attendance_policy_assignments
    FOR EACH ROW EXECUTE FUNCTION public.fn_validate_target_fk();

-- ───────────────────────────────────────────────────────────────────
-- Layer 2: Cascade DELETE triggers on parent tables
-- When a department or employee is deleted, clean up all their
-- polymorphic assignments across all 5 tables.
-- ───────────────────────────────────────────────────────────────────
CREATE OR REPLACE FUNCTION public.fn_cascade_delete_department_assignments()
RETURNS TRIGGER AS $$
BEGIN
    DELETE FROM public.payroll_rule_assignments
        WHERE assignee_type = 'department' AND assignee_id = OLD.id;
    DELETE FROM public.payroll_incentive_assignments
        WHERE assignee_type = 'department' AND assignee_id = OLD.id;
    DELETE FROM public.leave_policy_assignments
        WHERE assignee_type = 'department' AND assignee_id = OLD.id;
    DELETE FROM public.overtime_policy_assignments
        WHERE assignee_type = 'department' AND assignee_id = OLD.id;
    DELETE FROM public.attendance_policy_assignments
        WHERE target_type = 'GROUP' AND target_id = OLD.id;
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION public.fn_cascade_delete_employee_assignments()
RETURNS TRIGGER AS $$
BEGIN
    DELETE FROM public.payroll_rule_assignments
        WHERE assignee_type = 'employee' AND assignee_id = OLD.id;
    DELETE FROM public.payroll_incentive_assignments
        WHERE assignee_type = 'employee' AND assignee_id = OLD.id;
    DELETE FROM public.leave_policy_assignments
        WHERE assignee_type = 'employee' AND assignee_id = OLD.id;
    DELETE FROM public.overtime_policy_assignments
        WHERE assignee_type = 'employee' AND assignee_id = OLD.id;
    DELETE FROM public.attendance_policy_assignments
        WHERE target_type = 'USER' AND target_id = OLD.id;
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

-- Apply cascade triggers to parent tables
CREATE TRIGGER trg_cascade_delete_dept_assignments
    BEFORE DELETE ON public.departments
    FOR EACH ROW EXECUTE FUNCTION public.fn_cascade_delete_department_assignments();

CREATE TRIGGER trg_cascade_delete_emp_assignments
    BEFORE DELETE ON public.employees
    FOR EACH ROW EXECUTE FUNCTION public.fn_cascade_delete_employee_assignments();

-- ───────────────────────────────────────────────────────────────────
-- Supporting indexes for the cascade DELETE performance
-- These ensure the DELETE in triggers uses index scans, not seq scans
-- ───────────────────────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_payroll_rule_assignments_assignee
    ON public.payroll_rule_assignments (assignee_type, assignee_id);

CREATE INDEX IF NOT EXISTS idx_payroll_incentive_assignments_assignee
    ON public.payroll_incentive_assignments (assignee_type, assignee_id);

CREATE INDEX IF NOT EXISTS idx_leave_policy_assignments_assignee
    ON public.leave_policy_assignments (assignee_type, assignee_id);

CREATE INDEX IF NOT EXISTS idx_overtime_policy_assignments_assignee
    ON public.overtime_policy_assignments (assignee_type, assignee_id);

CREATE INDEX IF NOT EXISTS idx_attendance_policy_assignments_target
    ON public.attendance_policy_assignments (target_type, target_id);

BEGIN;

-- =============================================================================
-- Work Policy v2 — Pre-Go-Live HRM Standardization
-- Single source of truth on work_policy_groups / work_policy_versions /
-- work_policy_memberships (anchored to employment_relationships) and immutable
-- overtime_policy_versions. Removes legacy competing policy tables and keys.
-- =============================================================================

-- 1. Update cascade-delete trigger functions before dropping legacy assignment tables
CREATE OR REPLACE FUNCTION public.fn_cascade_delete_department_assignments() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    DELETE FROM public.payroll_rule_assignments
        WHERE assignee_type = 'department' AND assignee_id = OLD.id;

    DELETE FROM public.payroll_incentive_assignments
        WHERE assignee_type = 'department' AND assignee_id = OLD.id;

    DELETE FROM public.leave_policy_assignments
        WHERE assignee_type = 'department' AND assignee_id = OLD.id;

    RETURN OLD;
END;
$$;

CREATE OR REPLACE FUNCTION public.fn_cascade_delete_employee_assignments() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    DELETE FROM public.payroll_rule_assignments
        WHERE assignee_type = 'employee' AND assignee_id = OLD.id;

    DELETE FROM public.payroll_incentive_assignments
        WHERE assignee_type = 'employee' AND assignee_id = OLD.id;

    DELETE FROM public.leave_policy_assignments
        WHERE assignee_type = 'employee' AND assignee_id = OLD.id;

    RETURN OLD;
END;
$$;

-- 2. Immutable versioned overtime policy table & immutability guard
CREATE TABLE public.overtime_policy_versions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    policy_id uuid NOT NULL,
    version_no integer NOT NULL,
    status text DEFAULT 'draft'::text NOT NULL,
    row_version bigint DEFAULT 1 NOT NULL,
    daily_cap_minutes integer,
    monthly_cap_hours numeric(8,2),
    yearly_cap_hours numeric(8,2),
    rules jsonb DEFAULT '[]'::jsonb NOT NULL,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    published_by uuid,
    published_at timestamp with time zone,
    CONSTRAINT overtime_policy_versions_pkey PRIMARY KEY (id),
    CONSTRAINT overtime_policy_versions_policy_id_fkey
        FOREIGN KEY (policy_id) REFERENCES public.overtime_policy_definitions(id) ON DELETE RESTRICT,
    CONSTRAINT overtime_policy_versions_created_by_fkey
        FOREIGN KEY (created_by) REFERENCES public.users(id),
    CONSTRAINT overtime_policy_versions_published_by_fkey
        FOREIGN KEY (published_by) REFERENCES public.users(id),
    CONSTRAINT overtime_policy_versions_policy_version_unique UNIQUE (policy_id, version_no),
    CONSTRAINT overtime_policy_versions_version_no_check CHECK (version_no >= 1),
    CONSTRAINT overtime_policy_versions_row_version_check CHECK (row_version >= 1),
    CONSTRAINT overtime_policy_versions_status_check CHECK (status IN ('draft', 'published', 'archived')),
    CONSTRAINT overtime_policy_versions_caps_check CHECK (
        (daily_cap_minutes IS NULL OR daily_cap_minutes >= 0)
        AND (monthly_cap_hours IS NULL OR monthly_cap_hours >= 0)
        AND (yearly_cap_hours IS NULL OR yearly_cap_hours >= 0)
    ),
    CONSTRAINT overtime_policy_versions_rules_array_check CHECK (jsonb_typeof(rules) = 'array'),
    CONSTRAINT overtime_policy_versions_published_meta_check CHECK (
        (status = 'draft' AND published_by IS NULL AND published_at IS NULL)
        OR (status IN ('published', 'archived') AND published_by IS NOT NULL AND published_at IS NOT NULL)
    )
);

CREATE INDEX idx_overtime_policy_versions_policy_status
    ON public.overtime_policy_versions USING btree (policy_id, status, version_no DESC);

CREATE OR REPLACE FUNCTION public.fn_guard_overtime_policy_version_immutability() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        IF OLD.status IN ('published', 'archived') THEN
            RAISE EXCEPTION 'published or archived overtime policy version cannot be deleted'
                USING ERRCODE = '23514';
        END IF;
        RETURN OLD;
    END IF;

    IF OLD.status = 'archived' THEN
        RAISE EXCEPTION 'archived overtime policy version is immutable'
            USING ERRCODE = '23514';
    END IF;

    IF OLD.status = 'published' THEN
        IF NEW.status NOT IN ('published', 'archived') THEN
            RAISE EXCEPTION 'published overtime policy version cannot revert to draft'
                USING ERRCODE = '23514';
        END IF;

        IF NEW.policy_id IS DISTINCT FROM OLD.policy_id
           OR NEW.version_no IS DISTINCT FROM OLD.version_no
           OR NEW.daily_cap_minutes IS DISTINCT FROM OLD.daily_cap_minutes
           OR NEW.monthly_cap_hours IS DISTINCT FROM OLD.monthly_cap_hours
           OR NEW.yearly_cap_hours IS DISTINCT FROM OLD.yearly_cap_hours
           OR NEW.rules IS DISTINCT FROM OLD.rules
           OR NEW.created_by IS DISTINCT FROM OLD.created_by
           OR NEW.created_at IS DISTINCT FROM OLD.created_at
           OR NEW.published_by IS DISTINCT FROM OLD.published_by
           OR NEW.published_at IS DISTINCT FROM OLD.published_at THEN
            RAISE EXCEPTION 'published overtime policy version rules are immutable'
                USING ERRCODE = '23514';
        END IF;
    END IF;

    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.overtime_policy_versions
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_guard_overtime_policy_version_immutability
    BEFORE UPDATE OR DELETE ON public.overtime_policy_versions
    FOR EACH ROW EXECUTE FUNCTION public.fn_guard_overtime_policy_version_immutability();

-- 3. Unified Work Policy Groups, Versions & Relationship-Anchored Memberships
CREATE TABLE public.work_policy_groups (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    code text NOT NULL,
    name text NOT NULL,
    description text,
    archived_at timestamp with time zone,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT work_policy_groups_pkey PRIMARY KEY (id),
    CONSTRAINT work_policy_groups_code_key UNIQUE (code),
    CONSTRAINT work_policy_groups_code_check CHECK (length(btrim(code)) BETWEEN 1 AND 64),
    CONSTRAINT work_policy_groups_name_check CHECK (length(btrim(name)) BETWEEN 1 AND 255),
    CONSTRAINT work_policy_groups_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id)
);

CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.work_policy_groups
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TABLE public.work_policy_versions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    group_id uuid NOT NULL,
    version_no integer NOT NULL,
    status text DEFAULT 'draft'::text NOT NULL,
    row_version bigint DEFAULT 1 NOT NULL,
    effective_from date NOT NULL,
    effective_to date,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    published_by uuid,
    published_at timestamp with time zone,
    publish_idempotency_key text,
    -- Attendance & work credit rules
    minutes_per_credit integer DEFAULT 480 NOT NULL,
    work_unit_minutes integer DEFAULT 1 NOT NULL,
    rounding_policy text DEFAULT 'none'::text NOT NULL,
    early_arrival_strategy text DEFAULT 'clamp'::text NOT NULL,
    late_arrival_grace_minutes integer DEFAULT 0 NOT NULL,
    early_ot_enabled boolean DEFAULT false NOT NULL,
    early_ot_qualification_grace_minutes integer DEFAULT 0 NOT NULL,
    early_ot_max_minutes integer,
    break_auto_deduct boolean DEFAULT true NOT NULL,
    break_is_overtime boolean DEFAULT false NOT NULL,
    ot_aggregation_mode text DEFAULT 'aggregate'::text NOT NULL,
    -- Schedule link
    working_pattern_id uuid,
    -- Pay basis rules
    pay_method text NOT NULL,
    monthly_denominator text,
    monthly_standard_days numeric(12,4),
    rate_component_definition_id uuid,
    paid_leave_credit_mode text DEFAULT 'standard_day'::text NOT NULL,
    -- Overtime link
    overtime_policy_version_id uuid NOT NULL,
    CONSTRAINT work_policy_versions_pkey PRIMARY KEY (id),
    CONSTRAINT work_policy_versions_group_id_fkey
        FOREIGN KEY (group_id) REFERENCES public.work_policy_groups(id) ON DELETE RESTRICT,
    CONSTRAINT work_policy_versions_created_by_fkey
        FOREIGN KEY (created_by) REFERENCES public.users(id),
    CONSTRAINT work_policy_versions_published_by_fkey
        FOREIGN KEY (published_by) REFERENCES public.users(id),
    CONSTRAINT work_policy_versions_working_pattern_id_fkey
        FOREIGN KEY (working_pattern_id) REFERENCES public.working_patterns(id) ON DELETE RESTRICT,
    CONSTRAINT work_policy_versions_rate_component_definition_id_fkey
        FOREIGN KEY (rate_component_definition_id) REFERENCES public.salary_component_definitions(id) ON DELETE RESTRICT,
    CONSTRAINT work_policy_versions_overtime_policy_version_id_fkey
        FOREIGN KEY (overtime_policy_version_id) REFERENCES public.overtime_policy_versions(id) ON DELETE RESTRICT,
    CONSTRAINT work_policy_versions_group_version_unique UNIQUE (group_id, version_no),
    CONSTRAINT work_policy_versions_version_no_check CHECK (version_no >= 1),
    CONSTRAINT work_policy_versions_row_version_check CHECK (row_version >= 1),
    CONSTRAINT work_policy_versions_status_check CHECK (status IN ('draft', 'published', 'archived')),
    CONSTRAINT work_policy_versions_effective_range_check CHECK (effective_to IS NULL OR effective_to > effective_from),
    CONSTRAINT work_policy_versions_published_meta_check CHECK (
        (status = 'draft' AND published_by IS NULL AND published_at IS NULL)
        OR (status IN ('published', 'archived') AND published_by IS NOT NULL AND published_at IS NOT NULL)
    ),
    CONSTRAINT work_policy_versions_minutes_check CHECK (minutes_per_credit > 0 AND work_unit_minutes > 0),
    CONSTRAINT work_policy_versions_rounding_check CHECK (rounding_policy IN ('none', 'floor', 'round', 'ceil')),
    CONSTRAINT work_policy_versions_early_arrival_check CHECK (early_arrival_strategy IN ('allow', 'clamp', 'block')),
    CONSTRAINT work_policy_versions_late_grace_check CHECK (late_arrival_grace_minutes >= 0),
    CONSTRAINT work_policy_versions_early_grace_check CHECK (early_ot_qualification_grace_minutes >= 0),
    CONSTRAINT work_policy_versions_early_max_check CHECK (early_ot_max_minutes IS NULL OR early_ot_max_minutes >= 0),
    CONSTRAINT work_policy_versions_ot_aggregation_check CHECK (ot_aggregation_mode IN ('aggregate', 'per_segment')),
    CONSTRAINT work_policy_versions_pay_method_check CHECK (pay_method IN ('monthly', 'hourly', 'shift')),
    CONSTRAINT work_policy_versions_paid_leave_credit_mode_check CHECK (
        paid_leave_credit_mode IN ('standard_day', 'scheduled_minutes', 'unpaid')
    ),
    CONSTRAINT work_policy_versions_pay_basis_check CHECK (
        (pay_method = 'monthly' AND monthly_denominator = 'fixed' AND monthly_standard_days IS NOT NULL AND monthly_standard_days > 0 AND rate_component_definition_id IS NULL)
        OR (pay_method = 'monthly' AND monthly_denominator = 'schedule' AND monthly_standard_days IS NULL AND rate_component_definition_id IS NULL)
        OR (pay_method IN ('hourly', 'shift') AND monthly_denominator IS NULL AND monthly_standard_days IS NULL AND rate_component_definition_id IS NOT NULL)
    ),
    CONSTRAINT work_policy_versions_published_no_overlap EXCLUDE USING gist (
        group_id WITH =,
        daterange(effective_from, COALESCE(effective_to, 'infinity'::date), '[)') WITH &&
    ) WHERE (status = 'published')
);

CREATE INDEX work_policy_versions_effective_idx
    ON public.work_policy_versions USING btree (group_id, effective_from DESC, version_no DESC, id DESC)
    WHERE (status = 'published');

CREATE INDEX work_policy_versions_group_status_idx
    ON public.work_policy_versions USING btree (group_id, status, version_no DESC);

CREATE UNIQUE INDEX work_policy_versions_publish_idempotency_idx
    ON public.work_policy_versions USING btree (group_id, publish_idempotency_key)
    WHERE (publish_idempotency_key IS NOT NULL);

CREATE OR REPLACE FUNCTION public.fn_guard_work_policy_version_immutability() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        IF OLD.status IN ('published', 'archived') THEN
            RAISE EXCEPTION 'published or archived work policy version cannot be deleted'
                USING ERRCODE = '23514';
        END IF;
        RETURN OLD;
    END IF;

    IF TG_OP = 'INSERT' OR (TG_OP = 'UPDATE' AND OLD.status = 'draft' AND NEW.status = 'published') THEN
        IF NEW.status = 'published' AND NOT EXISTS (
            SELECT 1
            FROM public.overtime_policy_versions ov
            WHERE ov.id = NEW.overtime_policy_version_id
              AND ov.status = 'published'
        ) THEN
            RAISE EXCEPTION 'published work policy version requires a published overtime policy version'
                USING ERRCODE = '23514';
        END IF;
    END IF;

    IF TG_OP = 'INSERT' THEN
        RETURN NEW;
    END IF;

    IF OLD.status = 'archived' THEN
        RAISE EXCEPTION 'archived work policy version is immutable'
            USING ERRCODE = '23514';
    END IF;

    IF OLD.status = 'published' THEN
        IF NEW.status NOT IN ('published', 'archived') THEN
            RAISE EXCEPTION 'published work policy version cannot revert to draft'
                USING ERRCODE = '23514';
        END IF;

        IF NEW.group_id IS DISTINCT FROM OLD.group_id
           OR NEW.version_no IS DISTINCT FROM OLD.version_no
           OR NEW.effective_from IS DISTINCT FROM OLD.effective_from
           OR NEW.created_by IS DISTINCT FROM OLD.created_by
           OR NEW.created_at IS DISTINCT FROM OLD.created_at
           OR NEW.published_by IS DISTINCT FROM OLD.published_by
           OR NEW.published_at IS DISTINCT FROM OLD.published_at
           OR NEW.publish_idempotency_key IS DISTINCT FROM OLD.publish_idempotency_key
           OR NEW.minutes_per_credit IS DISTINCT FROM OLD.minutes_per_credit
           OR NEW.work_unit_minutes IS DISTINCT FROM OLD.work_unit_minutes
           OR NEW.rounding_policy IS DISTINCT FROM OLD.rounding_policy
           OR NEW.early_arrival_strategy IS DISTINCT FROM OLD.early_arrival_strategy
           OR NEW.late_arrival_grace_minutes IS DISTINCT FROM OLD.late_arrival_grace_minutes
           OR NEW.early_ot_enabled IS DISTINCT FROM OLD.early_ot_enabled
           OR NEW.early_ot_qualification_grace_minutes IS DISTINCT FROM OLD.early_ot_qualification_grace_minutes
           OR NEW.early_ot_max_minutes IS DISTINCT FROM OLD.early_ot_max_minutes
           OR NEW.break_auto_deduct IS DISTINCT FROM OLD.break_auto_deduct
           OR NEW.break_is_overtime IS DISTINCT FROM OLD.break_is_overtime
           OR NEW.ot_aggregation_mode IS DISTINCT FROM OLD.ot_aggregation_mode
           OR NEW.working_pattern_id IS DISTINCT FROM OLD.working_pattern_id
           OR NEW.pay_method IS DISTINCT FROM OLD.pay_method
           OR NEW.monthly_denominator IS DISTINCT FROM OLD.monthly_denominator
           OR NEW.monthly_standard_days IS DISTINCT FROM OLD.monthly_standard_days
           OR NEW.rate_component_definition_id IS DISTINCT FROM OLD.rate_component_definition_id
           OR NEW.paid_leave_credit_mode IS DISTINCT FROM OLD.paid_leave_credit_mode
           OR NEW.overtime_policy_version_id IS DISTINCT FROM OLD.overtime_policy_version_id THEN
            RAISE EXCEPTION 'published work policy version rules are immutable'
                USING ERRCODE = '23514';
        END IF;
    END IF;

    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.work_policy_versions
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_guard_work_policy_version_immutability
    BEFORE INSERT OR UPDATE OR DELETE ON public.work_policy_versions
    FOR EACH ROW EXECUTE FUNCTION public.fn_guard_work_policy_version_immutability();

CREATE TABLE public.work_policy_memberships (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employment_relationship_id uuid NOT NULL,
    group_id uuid NOT NULL,
    effective_from date NOT NULL,
    effective_to date,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT work_policy_memberships_pkey PRIMARY KEY (id),
    CONSTRAINT work_policy_memberships_relationship_id_fkey
        FOREIGN KEY (employment_relationship_id) REFERENCES public.employment_relationships(id) ON DELETE RESTRICT,
    CONSTRAINT work_policy_memberships_group_id_fkey
        FOREIGN KEY (group_id) REFERENCES public.work_policy_groups(id) ON DELETE RESTRICT,
    CONSTRAINT work_policy_memberships_created_by_fkey
        FOREIGN KEY (created_by) REFERENCES public.users(id),
    CONSTRAINT work_policy_memberships_effective_range_check CHECK (effective_to IS NULL OR effective_to > effective_from),
    CONSTRAINT work_policy_memberships_effective_no_overlap EXCLUDE USING gist (
        employment_relationship_id WITH =,
        daterange(effective_from, COALESCE(effective_to, 'infinity'::date), '[)') WITH &&
    )
);

CREATE INDEX work_policy_memberships_rel_effective_idx
    ON public.work_policy_memberships USING btree (employment_relationship_id, effective_from DESC, id DESC);

CREATE INDEX work_policy_memberships_group_effective_idx
    ON public.work_policy_memberships USING btree (group_id, effective_from DESC, id DESC);

-- 4. Temporal containment triggers between employment_relationships and work_policy_memberships
CREATE OR REPLACE FUNCTION public.fn_validate_work_policy_membership_rel_bounds() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_rel_start date;
    v_rel_end date;
    v_rel_deleted_at timestamp with time zone;
BEGIN
    SELECT start_date, end_date, deleted_at
    INTO v_rel_start, v_rel_end, v_rel_deleted_at
    FROM public.employment_relationships
    WHERE id = NEW.employment_relationship_id;

    IF NOT FOUND THEN
        RETURN NEW;
    END IF;

    IF v_rel_deleted_at IS NOT NULL THEN
        RAISE EXCEPTION 'work policy membership requires non-deleted employment relationship'
            USING ERRCODE = '23514';
    END IF;

    IF NEW.effective_from < v_rel_start
       OR (v_rel_end IS NOT NULL AND (NEW.effective_to IS NULL OR NEW.effective_to > (v_rel_end + 1))) THEN
        RAISE EXCEPTION 'work policy membership dates must fit within parent employment relationship bounds'
            USING ERRCODE = '23514';
    END IF;

    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION public.fn_validate_relationship_policy_bounds_guard() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.deleted_at IS NULL
       AND (NEW.start_date IS DISTINCT FROM OLD.start_date OR NEW.end_date IS DISTINCT FROM OLD.end_date)
       AND EXISTS (
           SELECT 1
           FROM public.work_policy_memberships m
           WHERE m.employment_relationship_id = NEW.id
             AND (
                 m.effective_from < NEW.start_date
                 OR (NEW.end_date IS NOT NULL AND (m.effective_to IS NULL OR m.effective_to > (NEW.end_date + 1)))
             )
       ) THEN
        RAISE EXCEPTION 'employment relationship bounds cannot exclude existing work policy memberships'
            USING ERRCODE = '23514';
    END IF;

    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_work_policy_memberships_rel_bounds
    BEFORE INSERT OR UPDATE OF employment_relationship_id, effective_from, effective_to
    ON public.work_policy_memberships
    FOR EACH ROW EXECUTE FUNCTION public.fn_validate_work_policy_membership_rel_bounds();

CREATE TRIGGER trg_employment_relationships_policy_bounds_guard
    BEFORE UPDATE OF start_date, end_date, deleted_at
    ON public.employment_relationships
    FOR EACH ROW EXECUTE FUNCTION public.fn_validate_relationship_policy_bounds_guard();

-- 5. Update provenance FKs on attendance_daily_summaries & payslip_segments
ALTER TABLE public.attendance_daily_summaries
    DROP CONSTRAINT IF EXISTS attendance_daily_summaries_work_policy_version_id_fkey;

UPDATE public.attendance_daily_summaries
SET work_policy_version_id = NULL
WHERE work_policy_version_id IS NOT NULL;

ALTER TABLE public.attendance_daily_summaries
    ADD CONSTRAINT attendance_daily_summaries_work_policy_version_id_fkey
        FOREIGN KEY (work_policy_version_id) REFERENCES public.work_policy_versions(id) ON DELETE RESTRICT;

DROP INDEX IF EXISTS public.payslip_segments_labor_policy_version_idx;

ALTER TABLE public.payslip_segments
    DROP CONSTRAINT IF EXISTS payslip_segments_labor_policy_group_version_id_fkey,
    DROP COLUMN IF EXISTS labor_policy_group_version_id,
    ADD COLUMN IF NOT EXISTS work_policy_version_id uuid;

ALTER TABLE public.payslip_segments
    ADD CONSTRAINT payslip_segments_work_policy_version_id_fkey
        FOREIGN KEY (work_policy_version_id) REFERENCES public.work_policy_versions(id) ON DELETE RESTRICT;

CREATE INDEX IF NOT EXISTS payslip_segments_work_policy_version_idx
    ON public.payslip_segments USING btree (work_policy_version_id)
    WHERE (work_policy_version_id IS NOT NULL);

-- 6. Drop legacy competing policy tables, reconciliation view, unused trigger fn, and config_entries keys
DROP VIEW IF EXISTS public.department_payroll_reconciliation;

DROP TABLE IF EXISTS public.labor_policy_memberships CASCADE;
DROP TABLE IF EXISTS public.labor_policy_group_versions CASCADE;
DROP TABLE IF EXISTS public.labor_policy_groups CASCADE;
DROP TABLE IF EXISTS public.work_credit_policies CASCADE;
DROP TABLE IF EXISTS public.pay_basis_policies CASCADE;
DROP TABLE IF EXISTS public.attendance_policy_assignments CASCADE;
DROP TABLE IF EXISTS public.attendance_policy_versions CASCADE;
DROP TABLE IF EXISTS public.attendance_policies CASCADE;
DROP TABLE IF EXISTS public.overtime_policy_assignments CASCADE;
DROP TABLE IF EXISTS public.overtime_rules CASCADE;

DROP FUNCTION IF EXISTS public.fn_validate_target_fk();

DELETE FROM public.config_entries
WHERE config_key IN (
    'payroll.std_working_day_mode',
    'payroll.fixed_std_days',
    'payroll.working_days_per_week',
    'payroll.hours_per_day',
    'payroll.max_ot_hours_per_day',
    'payroll.max_ot_hours_per_month',
    'overtime.max_hours_per_month'
);

INSERT INTO public.schema_migrations(version)
VALUES ('20261127000000_work_policy_v2_standardization');

COMMIT;

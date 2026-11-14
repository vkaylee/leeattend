BEGIN;

CREATE TABLE public.work_credit_policies (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL,
    minutes_per_credit integer NOT NULL,
    attendance_policy_id uuid NOT NULL REFERENCES public.attendance_policies(id),
    working_pattern_id uuid NOT NULL REFERENCES public.working_patterns(id),
    created_by uuid NOT NULL REFERENCES public.users(id),
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT work_credit_policies_name_check CHECK (length(btrim(name)) BETWEEN 1 AND 255),
    CONSTRAINT work_credit_policies_minutes_check CHECK (minutes_per_credit > 0)
);

CREATE TABLE public.pay_basis_policies (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL,
    method text NOT NULL,
    monthly_denominator text,
    monthly_standard_days numeric(12,4),
    created_by uuid NOT NULL REFERENCES public.users(id),
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT pay_basis_policies_name_check CHECK (length(btrim(name)) BETWEEN 1 AND 255),
    CONSTRAINT pay_basis_policies_method_check CHECK (method IN ('monthly', 'hourly', 'shift')),
    CONSTRAINT pay_basis_policies_denominator_check CHECK (
        (method = 'monthly' AND monthly_denominator IN ('fixed', 'schedule'))
        OR (method IN ('hourly', 'shift') AND monthly_denominator IS NULL AND monthly_standard_days IS NULL)
    ),
    CONSTRAINT pay_basis_policies_days_check CHECK (monthly_standard_days IS NULL OR monthly_standard_days > 0),
    CONSTRAINT pay_basis_policies_fixed_days_check CHECK (
        method <> 'monthly' OR monthly_denominator <> 'fixed' OR monthly_standard_days IS NOT NULL
    ),
    CONSTRAINT pay_basis_policies_schedule_days_check CHECK (
        method <> 'monthly' OR monthly_denominator <> 'schedule' OR monthly_standard_days IS NULL
    )
);

CREATE TABLE public.labor_policy_groups (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL,
    created_by uuid NOT NULL REFERENCES public.users(id),
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT labor_policy_groups_name_check CHECK (length(btrim(name)) BETWEEN 1 AND 255)
);

CREATE TABLE public.labor_policy_group_versions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id uuid NOT NULL REFERENCES public.labor_policy_groups(id),
    work_credit_policy_id uuid NOT NULL REFERENCES public.work_credit_policies(id),
    pay_basis_policy_id uuid NOT NULL REFERENCES public.pay_basis_policies(id),
    overtime_policy_id uuid NOT NULL REFERENCES public.overtime_policy_definitions(id),
    effective_from date NOT NULL,
    effective_to date,
    created_by uuid NOT NULL REFERENCES public.users(id),
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT labor_policy_group_versions_effective_range_check CHECK (effective_to IS NULL OR effective_to > effective_from)
);

CREATE TABLE public.labor_policy_memberships (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    employment_id uuid NOT NULL REFERENCES public.employments(id),
    group_id uuid NOT NULL REFERENCES public.labor_policy_groups(id),
    effective_from date NOT NULL,
    effective_to date,
    created_by uuid NOT NULL REFERENCES public.users(id),
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT labor_policy_memberships_effective_range_check CHECK (effective_to IS NULL OR effective_to > effective_from)
);

ALTER TABLE public.labor_policy_group_versions
    ADD CONSTRAINT labor_policy_group_versions_effective_no_overlap
    EXCLUDE USING gist (
        group_id WITH =,
        daterange(effective_from, COALESCE(effective_to, 'infinity'::date), '[)') WITH &&
    );

ALTER TABLE public.labor_policy_memberships
    ADD CONSTRAINT labor_policy_memberships_effective_no_overlap
    EXCLUDE USING gist (
        employment_id WITH =,
        daterange(effective_from, COALESCE(effective_to, 'infinity'::date), '[)') WITH &&
    );

CREATE INDEX labor_policy_group_versions_effective_idx
    ON public.labor_policy_group_versions (group_id, effective_from DESC, id DESC);
CREATE INDEX labor_policy_memberships_employment_effective_idx
    ON public.labor_policy_memberships (employment_id, effective_from DESC, id DESC);

INSERT INTO public.schema_migrations(version)
VALUES ('20261114080000_labor_policy_groups');

COMMIT;

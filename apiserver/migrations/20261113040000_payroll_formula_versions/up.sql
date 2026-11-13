BEGIN;

CREATE TABLE public.payroll_formula_rules (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    code text NOT NULL UNIQUE,
    name text NOT NULL,
    component_code text NOT NULL,
    created_by uuid NOT NULL REFERENCES public.users(id),
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    retired_at timestamptz,
    CONSTRAINT payroll_formula_rules_code_check CHECK (code ~ '^[a-z][a-z0-9_.]{1,99}$'),
    CONSTRAINT payroll_formula_rules_component_check CHECK (component_code ~ '^[A-Z][A-Z0-9_.]{1,99}$')
);

CREATE TABLE public.payroll_formula_rule_versions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id uuid NOT NULL REFERENCES public.payroll_formula_rules(id),
    version_number integer NOT NULL,
    expression text NOT NULL,
    effective_from date NOT NULL,
    status text NOT NULL DEFAULT 'draft',
    created_by uuid NOT NULL REFERENCES public.users(id),
    published_by uuid REFERENCES public.users(id),
    published_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT payroll_formula_rule_versions_status_check CHECK (status IN ('draft', 'published', 'retired')),
    CONSTRAINT payroll_formula_rule_versions_expression_check CHECK (length(btrim(expression)) BETWEEN 1 AND 4096),
    CONSTRAINT payroll_formula_rule_versions_version_check CHECK (version_number >= 1),
    UNIQUE (rule_id, version_number)
);

CREATE TABLE public.payroll_calculation_runs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    employment_id uuid NOT NULL REFERENCES public.employments(id),
    period_year integer NOT NULL,
    period_month integer NOT NULL,
    formula_version_id uuid REFERENCES public.payroll_formula_rule_versions(id),
    engine_version text NOT NULL DEFAULT 'legacy-v1',
    input_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
    output_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
    calculation_trace jsonb NOT NULL DEFAULT '[]'::jsonb,
    created_by uuid,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT payroll_calculation_runs_period_check CHECK (period_year BETWEEN 2000 AND 2100 AND period_month BETWEEN 1 AND 12)
);

CREATE INDEX payroll_formula_versions_effective_idx
    ON public.payroll_formula_rule_versions (rule_id, effective_from DESC, version_number DESC);
CREATE INDEX payroll_calculation_runs_employment_period_idx
    ON public.payroll_calculation_runs (employment_id, period_year, period_month, created_at DESC);

INSERT INTO public.schema_migrations(version)
VALUES ('20261113040000_payroll_formula_versions');

COMMIT;

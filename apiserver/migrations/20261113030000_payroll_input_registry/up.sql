BEGIN;

CREATE TABLE public.payroll_metric_definitions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    code text NOT NULL UNIQUE,
    name text NOT NULL,
    data_type text NOT NULL DEFAULT 'decimal',
    unit text NOT NULL,
    scope text NOT NULL DEFAULT 'employee',
    source text NOT NULL DEFAULT 'external',
    is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT payroll_metric_definitions_data_type_check CHECK (data_type IN ('decimal')),
    CONSTRAINT payroll_metric_definitions_scope_check CHECK (scope IN ('employee')),
    CONSTRAINT payroll_metric_definitions_code_check CHECK (code ~ '^[a-z][a-z0-9_.]{1,99}$')
);

CREATE TABLE public.payroll_input_batches (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    period_year integer NOT NULL,
    period_month integer NOT NULL,
    source text NOT NULL,
    idempotency_key text NOT NULL UNIQUE,
    request_hash text NOT NULL,
    status text NOT NULL DEFAULT 'submitted',
    submitted_by uuid NOT NULL REFERENCES public.users(id),
    approved_by uuid REFERENCES public.users(id),
    locked_by uuid REFERENCES public.users(id),
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT payroll_input_batches_period_check CHECK (period_year BETWEEN 2000 AND 2100 AND period_month BETWEEN 1 AND 12),
    CONSTRAINT payroll_input_batches_status_check CHECK (status IN ('submitted', 'approved', 'locked')),
    CONSTRAINT payroll_input_batches_source_check CHECK (length(btrim(source)) BETWEEN 1 AND 100)
);

CREATE TABLE public.payroll_inputs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    batch_id uuid NOT NULL REFERENCES public.payroll_input_batches(id),
    employment_id uuid NOT NULL REFERENCES public.employments(id),
    metric_definition_id uuid NOT NULL REFERENCES public.payroll_metric_definitions(id),
    value numeric(20,6) NOT NULL,
    unit text NOT NULL,
    source_reference text,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT payroll_inputs_unit_check CHECK (length(btrim(unit)) BETWEEN 1 AND 40),
    CONSTRAINT payroll_inputs_unique_metric UNIQUE (batch_id, employment_id, metric_definition_id)
);

CREATE INDEX payroll_input_batches_period_status_idx
    ON public.payroll_input_batches (period_year, period_month, status);
CREATE INDEX payroll_inputs_employment_metric_idx
    ON public.payroll_inputs (employment_id, metric_definition_id, created_at DESC);
CREATE INDEX payroll_inputs_batch_idx
    ON public.payroll_inputs (batch_id);

INSERT INTO public.payroll_metric_definitions (code, name, unit, scope, source)
VALUES
    ('production.output', 'Production output', 'product', 'employee', 'production'),
    ('production.target_output', 'Production target output', 'product', 'employee', 'production'),
    ('production.quality_rate', 'Production quality rate', 'ratio', 'employee', 'production')
ON CONFLICT (code) DO NOTHING;

INSERT INTO public.schema_migrations(version)
VALUES ('20261113030000_payroll_input_registry');

COMMIT;

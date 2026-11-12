BEGIN;

CREATE TABLE public.payroll_periods (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employer_id uuid NOT NULL,
    period_year integer NOT NULL,
    period_month integer NOT NULL,
    status character varying(16) NOT NULL DEFAULT 'open',
    version bigint NOT NULL DEFAULT 1,
    locked_at timestamp with time zone,
    locked_by uuid,
    reopened_at timestamp with time zone,
    reopened_by uuid,
    reopen_reason text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT payroll_periods_pkey PRIMARY KEY (id),
    CONSTRAINT payroll_periods_employer_fkey FOREIGN KEY (employer_id) REFERENCES public.employers(id),
    CONSTRAINT payroll_periods_locked_by_fkey FOREIGN KEY (locked_by) REFERENCES public.users(id),
    CONSTRAINT payroll_periods_reopened_by_fkey FOREIGN KEY (reopened_by) REFERENCES public.users(id),
    CONSTRAINT payroll_periods_year_check CHECK (period_year BETWEEN 2000 AND 2100),
    CONSTRAINT payroll_periods_month_check CHECK (period_month BETWEEN 1 AND 12),
    CONSTRAINT payroll_periods_status_check CHECK (status IN ('open', 'locked')),
    CONSTRAINT payroll_periods_unique_period UNIQUE (employer_id, period_year, period_month)
);

CREATE INDEX payroll_periods_status_idx
    ON public.payroll_periods (status, period_year, period_month);

CREATE OR REPLACE FUNCTION public.fn_payroll_periods_updated_at()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_payroll_periods_updated_at
BEFORE UPDATE ON public.payroll_periods
FOR EACH ROW EXECUTE FUNCTION public.fn_payroll_periods_updated_at();

INSERT INTO public.schema_migrations(version)
VALUES ('20261112030000_payroll_periods');

COMMIT;

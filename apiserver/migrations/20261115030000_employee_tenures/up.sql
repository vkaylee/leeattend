BEGIN;

CREATE TABLE public.employee_tenures (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employee_id uuid NOT NULL,
    employee_number character varying(50) NOT NULL,
    user_id uuid,
    start_date date NOT NULL,
    end_date date,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone,
    CONSTRAINT employee_tenures_pkey PRIMARY KEY (id),
    CONSTRAINT employee_tenures_employee_id_fkey
        FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE RESTRICT,
    CONSTRAINT employee_tenures_user_id_fkey
        FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE RESTRICT,
    CONSTRAINT employee_tenures_identity_key UNIQUE (id, employee_id),
    CONSTRAINT employee_tenures_number_check CHECK (length(btrim(employee_number)) > 0),
    CONSTRAINT employee_tenures_dates_check CHECK (end_date IS NULL OR start_date <= end_date),
    CONSTRAINT employee_tenures_no_overlap_person EXCLUDE USING gist (
        employee_id WITH =,
        daterange(start_date, COALESCE(end_date + 1, 'infinity'::date), '[)') WITH &&
    ) WHERE (deleted_at IS NULL),
    CONSTRAINT employee_tenures_no_overlap_number EXCLUDE USING gist (
        employee_number WITH =,
        daterange(start_date, COALESCE(end_date + 1, 'infinity'::date), '[)') WITH &&
    ) WHERE (deleted_at IS NULL)
);

CREATE INDEX idx_employee_tenures_employee
    ON public.employee_tenures (employee_id, start_date DESC, id DESC)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_employee_tenures_number
    ON public.employee_tenures (employee_number, start_date DESC, id DESC)
    WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX idx_employee_tenures_active_user
    ON public.employee_tenures (user_id)
    WHERE user_id IS NOT NULL AND deleted_at IS NULL;

CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.employee_tenures
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TABLE public.employee_tenure_reconciliation (
    employee_id uuid PRIMARY KEY,
    status character varying(30) DEFAULT 'pending' NOT NULL,
    reason character varying(100) NOT NULL,
    notes text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT employee_tenure_reconciliation_employee_id_fkey
        FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE RESTRICT,
    CONSTRAINT employee_tenure_reconciliation_status_check
        CHECK (status IN ('pending', 'resolved', 'blocked'))
);

CREATE INDEX idx_employee_tenure_reconciliation_status
    ON public.employee_tenure_reconciliation (status, updated_at, employee_id);

CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.employee_tenure_reconciliation
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

-- Do not infer tenure boundaries from legacy employee or segment dates. Record each
-- legacy employee for explicit reconciliation before any tenure-aware writer runs.
INSERT INTO public.employee_tenure_reconciliation (employee_id, reason)
SELECT id, 'legacy_tenure_mapping_required'
FROM public.employees
ON CONFLICT (employee_id) DO NOTHING;

ALTER TABLE public.employment_relationships
    ADD COLUMN tenure_id uuid;

ALTER TABLE ONLY public.employment_relationships
    ADD CONSTRAINT employment_relationships_tenure_identity_fkey
    FOREIGN KEY (tenure_id, employee_id)
    REFERENCES public.employee_tenures(id, employee_id)
    ON DELETE RESTRICT;

CREATE INDEX idx_employment_relationships_tenure_id
    ON public.employment_relationships (tenure_id, start_date DESC, id DESC)
    WHERE tenure_id IS NOT NULL AND deleted_at IS NULL;

INSERT INTO public.schema_migrations(version)
VALUES ('20261115030000_employee_tenures');

COMMIT;

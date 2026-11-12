BEGIN;

CREATE TABLE public.employment_relationships (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employee_id uuid NOT NULL,
    employer_id uuid NOT NULL,
    status character varying(50) DEFAULT 'active'::character varying NOT NULL,
    start_date date NOT NULL,
    end_date date,
    termination_reason character varying(50),
    notes text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone,
    CONSTRAINT employment_relationships_pkey PRIMARY KEY (id),
    CONSTRAINT employment_relationships_dates_check CHECK (end_date IS NULL OR start_date <= end_date),
    CONSTRAINT employment_relationships_employee_id_fkey FOREIGN KEY (employee_id) REFERENCES public.employees(id) ON DELETE RESTRICT,
    CONSTRAINT employment_relationships_employer_id_fkey FOREIGN KEY (employer_id) REFERENCES public.employers(id) ON DELETE RESTRICT,
    CONSTRAINT employment_relationships_identity_key UNIQUE (id, employee_id, employer_id),
    CONSTRAINT employment_relationships_no_overlap EXCLUDE USING gist (
        employee_id WITH =,
        employer_id WITH =,
        daterange(start_date, COALESCE(end_date + 1, 'infinity'::date), '[)') WITH &&
    ) WHERE (deleted_at IS NULL)
);

CREATE INDEX idx_employment_relationships_employee
    ON public.employment_relationships (employee_id, start_date DESC, id DESC)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_employment_relationships_employer
    ON public.employment_relationships (employer_id, start_date DESC, id DESC)
    WHERE deleted_at IS NULL;

CREATE TRIGGER trg_updated_at
    BEFORE UPDATE ON public.employment_relationships
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

ALTER TABLE ONLY public.employments
    ADD COLUMN relationship_id uuid;

ALTER TABLE ONLY public.employee_contracts
    ADD COLUMN relationship_id uuid;

ALTER TABLE ONLY public.employments
    ADD CONSTRAINT employments_relationship_id_fkey
    FOREIGN KEY (relationship_id) REFERENCES public.employment_relationships(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.employee_contracts
    ADD CONSTRAINT employee_contracts_relationship_id_fkey
    FOREIGN KEY (relationship_id) REFERENCES public.employment_relationships(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.employments
    ADD CONSTRAINT employments_relationship_identity_fkey
    FOREIGN KEY (relationship_id, employee_id, employer_id)
    REFERENCES public.employment_relationships(id, employee_id, employer_id)
    ON DELETE RESTRICT;

CREATE FUNCTION public.fn_validate_contract_relationship_link()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF NEW.relationship_id IS NULL THEN
        RETURN NEW;
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM public.employments segment
        WHERE segment.id = NEW.employment_id
          AND segment.relationship_id = NEW.relationship_id
          AND segment.deleted_at IS NULL
    ) THEN
        RAISE EXCEPTION
            'employee contract relationship_id must match its employment segment'
            USING ERRCODE = '23514';
    END IF;

    RETURN NEW;
END;
$$;

CREATE FUNCTION public.fn_validate_segment_relationship_change()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF NEW.relationship_id IS DISTINCT FROM OLD.relationship_id
       AND EXISTS (
           SELECT 1
           FROM public.employee_contracts contract
           WHERE contract.employment_id = NEW.id
             AND contract.relationship_id IS NOT NULL
             AND contract.relationship_id IS DISTINCT FROM NEW.relationship_id
             AND contract.deleted_at IS NULL
       ) THEN
        RAISE EXCEPTION
            'employment segment relationship_id cannot change while linked contracts exist'
            USING ERRCODE = '23514';
    END IF;

    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_validate_contract_relationship_link
    BEFORE INSERT OR UPDATE OF employment_id, relationship_id ON public.employee_contracts
    FOR EACH ROW EXECUTE FUNCTION public.fn_validate_contract_relationship_link();

CREATE TRIGGER trg_validate_segment_relationship_change
    BEFORE UPDATE OF relationship_id ON public.employments
    FOR EACH ROW EXECUTE FUNCTION public.fn_validate_segment_relationship_change();

CREATE INDEX idx_employments_relationship_id
    ON public.employments (relationship_id)
    WHERE relationship_id IS NOT NULL AND deleted_at IS NULL;

CREATE INDEX idx_employee_contracts_relationship_id
    ON public.employee_contracts (relationship_id)
    WHERE relationship_id IS NOT NULL AND deleted_at IS NULL;

INSERT INTO public.schema_migrations(version)
VALUES ('20261112010000_employment_relationships');

COMMIT;

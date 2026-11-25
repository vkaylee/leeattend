BEGIN;

-- =============================================================================
-- Employee Tenures v2 — Pre-Go-Live HRM Standardization
-- Single source of truth on employee_tenures, strict parent links, temporal
-- containment triggers, permanent per-person code ownership, ESS tenure_id.
-- =============================================================================

-- 1. Temporal containment trigger functions
CREATE OR REPLACE FUNCTION public.fn_validate_relationship_tenure_bounds() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_tenure_start date;
    v_tenure_end date;
    v_tenure_deleted_at timestamp with time zone;
BEGIN
    IF NEW.deleted_at IS NULL THEN
        SELECT start_date, end_date, deleted_at
        INTO v_tenure_start, v_tenure_end, v_tenure_deleted_at
        FROM public.employee_tenures
        WHERE id = NEW.tenure_id
          AND employee_id = NEW.employee_id;

        IF NOT FOUND THEN
            RETURN NEW;
        END IF;

        IF v_tenure_deleted_at IS NOT NULL THEN
            RAISE EXCEPTION
                'employment relationship requires non-deleted tenure belonging to the same employee'
                USING ERRCODE = '23514';
        END IF;

        IF NEW.start_date < v_tenure_start
           OR (v_tenure_end IS NOT NULL AND (NEW.end_date IS NULL OR NEW.end_date > v_tenure_end)) THEN
            RAISE EXCEPTION
                'employment relationship dates must fit within parent tenure bounds'
                USING ERRCODE = '23514';
        END IF;

        IF TG_OP = 'UPDATE'
           AND (NEW.start_date IS DISTINCT FROM OLD.start_date OR NEW.end_date IS DISTINCT FROM OLD.end_date)
           AND EXISTS (
               SELECT 1
               FROM public.employments segment
               WHERE segment.relationship_id = NEW.id
                 AND segment.deleted_at IS NULL
                 AND (
                     segment.start_date < NEW.start_date
                     OR (NEW.end_date IS NOT NULL AND (segment.end_date IS NULL OR segment.end_date > NEW.end_date))
                 )
           ) THEN
            RAISE EXCEPTION
                'employment relationship bounds cannot exclude existing employment segments'
                USING ERRCODE = '23514';
        END IF;
    END IF;

    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION public.fn_validate_segment_relationship_bounds() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_rel_start date;
    v_rel_end date;
    v_rel_deleted_at timestamp with time zone;
BEGIN
    IF NEW.deleted_at IS NULL THEN
        SELECT start_date, end_date, deleted_at
        INTO v_rel_start, v_rel_end, v_rel_deleted_at
        FROM public.employment_relationships
        WHERE id = NEW.relationship_id
          AND employee_id = NEW.employee_id
          AND employer_id = NEW.employer_id;

        IF NOT FOUND THEN
            RETURN NEW;
        END IF;

        IF v_rel_deleted_at IS NOT NULL THEN
            RAISE EXCEPTION
                'employment segment requires non-deleted relationship belonging to the same employee and employer'
                USING ERRCODE = '23514';
        END IF;

        IF NEW.start_date < v_rel_start
           OR (v_rel_end IS NOT NULL AND (NEW.end_date IS NULL OR NEW.end_date > v_rel_end)) THEN
            RAISE EXCEPTION
                'employment segment dates must fit within parent relationship bounds'
                USING ERRCODE = '23514';
        END IF;
    END IF;

    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION public.fn_validate_tenure_bounds_guard() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.deleted_at IS NULL
       AND (NEW.start_date IS DISTINCT FROM OLD.start_date OR NEW.end_date IS DISTINCT FROM OLD.end_date)
       AND EXISTS (
           SELECT 1
           FROM public.employment_relationships rel
           WHERE rel.tenure_id = NEW.id
             AND rel.deleted_at IS NULL
             AND (
                 rel.start_date < NEW.start_date
                 OR (NEW.end_date IS NOT NULL AND (rel.end_date IS NULL OR rel.end_date > NEW.end_date))
             )
       ) THEN
        RAISE EXCEPTION
            'employee tenure bounds cannot exclude existing employment relationships'
            USING ERRCODE = '23514';
    END IF;

    RETURN NEW;
END;
$$;

-- 2. Enhance employee_tenures with normalized code, permanent ownership, and seniority metadata
ALTER TABLE public.employee_tenures
    ADD COLUMN IF NOT EXISTS employee_number_normalized text
        GENERATED ALWAYS AS (upper(btrim((employee_number)::text))) STORED NOT NULL,
    ADD COLUMN IF NOT EXISTS hire_type character varying(20) DEFAULT 'initial'::character varying NOT NULL,
    ADD COLUMN IF NOT EXISTS prior_service_credit_days integer DEFAULT 0 NOT NULL,
    ADD COLUMN IF NOT EXISTS termination_reason text,
    ADD COLUMN IF NOT EXISTS rehire_eligible boolean DEFAULT true NOT NULL,
    ADD COLUMN IF NOT EXISTS notes text;

ALTER TABLE public.employee_tenures
    DROP CONSTRAINT IF EXISTS employee_tenures_no_overlap_number;

ALTER TABLE public.employee_tenures
    ADD CONSTRAINT employee_tenures_hire_type_check
        CHECK (((hire_type)::text = ANY ((ARRAY['initial'::character varying, 'rehire'::character varying])::text[]))),
    ADD CONSTRAINT employee_tenures_prior_service_check
        CHECK ((prior_service_credit_days >= 0)),
    ADD CONSTRAINT employee_tenures_permanent_code_owner
        EXCLUDE USING gist (employee_number_normalized WITH =, employee_id WITH <>);

DROP INDEX IF EXISTS public.idx_employee_tenures_active_user;
CREATE UNIQUE INDEX idx_employee_tenures_active_user
    ON public.employee_tenures USING btree (user_id)
    WHERE ((user_id IS NOT NULL) AND (end_date IS NULL) AND (deleted_at IS NULL));

CREATE INDEX IF NOT EXISTS idx_employee_tenures_number_normalized
    ON public.employee_tenures USING btree (employee_number_normalized, start_date DESC, id DESC)
    WHERE (deleted_at IS NULL);

CREATE INDEX IF NOT EXISTS idx_employee_tenures_number_search
    ON public.employee_tenures USING gin (employee_number public.gin_trgm_ops);

CREATE TRIGGER trg_roster_cache_invalidation_employee_tenures
    AFTER INSERT OR DELETE OR UPDATE ON public.employee_tenures
    FOR EACH ROW EXECUTE FUNCTION public.enqueue_roster_cache_invalidation('direct');

CREATE TRIGGER trg_validate_tenure_bounds_guard
    BEFORE UPDATE OF start_date, end_date, deleted_at ON public.employee_tenures
    FOR EACH ROW EXECUTE FUNCTION public.fn_validate_tenure_bounds_guard();

-- 3. Strict parent links & temporal bounds on relationships and segments
-- Backfill any pre-v2 relationships missing tenure_id
UPDATE public.employment_relationships er
SET tenure_id = (
    SELECT et.id
    FROM public.employee_tenures et
    WHERE et.employee_id = er.employee_id
      AND et.deleted_at IS NULL
      AND er.start_date >= et.start_date
      AND (et.end_date IS NULL OR er.start_date <= et.end_date)
    ORDER BY et.start_date DESC, et.id DESC
    LIMIT 1
)
WHERE er.tenure_id IS NULL;

-- Backfill missing employment_relationships for pre-v2 employments with NULL relationship_id
WITH orphan_groups AS (
    SELECT
        em.employee_id,
        em.employer_id,
        t.id AS tenure_id,
        MIN(em.start_date) AS start_date,
        CASE WHEN bool_or(em.end_date IS NULL) THEN NULL ELSE MAX(em.end_date) END AS end_date,
        CASE WHEN bool_or(em.status = 'active') THEN 'active' ELSE 'terminated' END AS status
    FROM public.employments em
    JOIN LATERAL (
        SELECT et.id
        FROM public.employee_tenures et
        WHERE et.employee_id = em.employee_id
          AND et.deleted_at IS NULL
          AND em.start_date >= et.start_date
          AND (et.end_date IS NULL OR em.start_date <= et.end_date)
        ORDER BY et.start_date DESC, et.id DESC
        LIMIT 1
    ) t ON TRUE
    WHERE em.relationship_id IS NULL
      AND em.deleted_at IS NULL
      AND NOT EXISTS (
          SELECT 1
          FROM public.employment_relationships er
          WHERE er.employee_id = em.employee_id
            AND er.employer_id = em.employer_id
            AND er.tenure_id = t.id
            AND er.deleted_at IS NULL
      )
    GROUP BY em.employee_id, em.employer_id, t.id
)
INSERT INTO public.employment_relationships (
    employee_id,
    employer_id,
    tenure_id,
    start_date,
    end_date,
    status
)
SELECT employee_id, employer_id, tenure_id, start_date, end_date, status
FROM orphan_groups;

UPDATE public.employments em
SET relationship_id = (
    SELECT r.id
    FROM public.employment_relationships r
    WHERE r.employee_id = em.employee_id
      AND r.employer_id = em.employer_id
      AND r.deleted_at IS NULL
      AND em.start_date >= r.start_date
      AND (r.end_date IS NULL OR em.start_date <= r.end_date)
    ORDER BY r.start_date DESC, r.id DESC
    LIMIT 1
)
WHERE em.relationship_id IS NULL;

ALTER TABLE public.employment_relationships
    ALTER COLUMN tenure_id SET NOT NULL;

DROP INDEX IF EXISTS public.idx_employment_relationships_tenure_id;
CREATE INDEX idx_employment_relationships_tenure_id
    ON public.employment_relationships USING btree (tenure_id, start_date DESC, id DESC)
    WHERE (deleted_at IS NULL);

CREATE TRIGGER trg_validate_relationship_tenure_bounds
    BEFORE INSERT OR UPDATE OF tenure_id, employee_id, start_date, end_date, deleted_at ON public.employment_relationships
    FOR EACH ROW EXECUTE FUNCTION public.fn_validate_relationship_tenure_bounds();

ALTER TABLE public.employments
    ALTER COLUMN relationship_id SET NOT NULL,
    ADD CONSTRAINT employments_dates_check CHECK (((end_date IS NULL) OR (start_date <= end_date)));

CREATE TRIGGER trg_validate_segment_relationship_bounds
    BEFORE INSERT OR UPDATE OF relationship_id, employee_id, employer_id, start_date, end_date, deleted_at ON public.employments
    FOR EACH ROW EXECUTE FUNCTION public.fn_validate_segment_relationship_bounds();

-- 4. ESS document scoping via tenure_id
ALTER TABLE public.employee_documents
    ADD COLUMN IF NOT EXISTS tenure_id uuid;

ALTER TABLE public.employee_documents
    ADD CONSTRAINT employee_documents_tenure_identity_fkey
    FOREIGN KEY (tenure_id, employee_id)
    REFERENCES public.employee_tenures(id, employee_id)
    ON DELETE RESTRICT;

CREATE INDEX IF NOT EXISTS idx_employee_documents_tenure_id
    ON public.employee_documents USING btree (tenure_id)
    WHERE (tenure_id IS NOT NULL);

ALTER TABLE public.file_acknowledgements
    ADD COLUMN IF NOT EXISTS tenure_id uuid;

ALTER TABLE public.file_acknowledgements
    ADD CONSTRAINT file_acknowledgements_tenure_identity_fkey
    FOREIGN KEY (tenure_id, employee_id)
    REFERENCES public.employee_tenures(id, employee_id)
    ON DELETE RESTRICT;

CREATE INDEX IF NOT EXISTS idx_file_acknowledgements_tenure_id
    ON public.file_acknowledgements USING btree (tenure_id)
    WHERE (tenure_id IS NOT NULL);

-- 5. Remove transitional reconciliation table and legacy dual-write columns on employees
DROP TABLE IF EXISTS public.employee_tenure_reconciliation CASCADE;

DROP INDEX IF EXISTS public.idx_employees_id_number_search;
DROP INDEX IF EXISTS public.idx_employees_organization_employee_id;
DROP INDEX IF EXISTS public.idx_employees_organization_user_id;
DROP INDEX IF EXISTS public.idx_employees_user_id;

ALTER TABLE public.employees
    DROP COLUMN IF EXISTS user_id,
    DROP COLUMN IF EXISTS employee_id_number,
    DROP COLUMN IF EXISTS hire_date,
    DROP COLUMN IF EXISTS original_hire_date,
    DROP COLUMN IF EXISTS termination_date,
    DROP COLUMN IF EXISTS is_active;

INSERT INTO public.schema_migrations(version)
VALUES ('20261126010000_employee_tenures_v2_standardization');

COMMIT;

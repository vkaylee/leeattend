BEGIN;

CREATE TABLE public.schedule_previews (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    requested_by uuid,
    actor_type character varying(16) NOT NULL,
    actor_name character varying(64),
    operation character varying(16) NOT NULL,
    request_hash character(64) NOT NULL,
    result_hash character(64) NOT NULL,
    scope_snapshot jsonb NOT NULL,
    result_snapshot jsonb NOT NULL,
    resolver_version character varying(32) NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT schedule_previews_pkey PRIMARY KEY (id),
    CONSTRAINT schedule_previews_requested_by_fkey FOREIGN KEY (requested_by)
        REFERENCES public.users(id) ON DELETE RESTRICT,
    CONSTRAINT schedule_previews_actor_check CHECK (
        (actor_type = 'human' AND requested_by IS NOT NULL AND actor_name IS NULL)
        OR (actor_type = 'system' AND requested_by IS NULL AND actor_name = 'schedule-publisher' AND operation = 'publish')
    ),
    CONSTRAINT schedule_previews_operation_check CHECK (operation IN ('publish', 'replan')),
    CONSTRAINT schedule_previews_hash_check CHECK (
        request_hash::text ~ '^[0-9a-f]{64}$'
        AND result_hash::text ~ '^[0-9a-f]{64}$'
    ),
    CONSTRAINT schedule_previews_snapshot_check CHECK (
        jsonb_typeof(scope_snapshot) = 'object'
        AND jsonb_typeof(result_snapshot) = 'object'
    ),
    CONSTRAINT schedule_previews_expiry_check CHECK (
        expires_at > created_at
        AND expires_at <= created_at + interval '15 minutes'
    )
);

CREATE INDEX schedule_previews_requested_by_created_at_idx
    ON public.schedule_previews (requested_by, created_at DESC);
CREATE INDEX schedule_previews_expires_at_idx
    ON public.schedule_previews (expires_at);

CREATE TABLE public.schedule_publication_batches (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    idempotency_key character varying(128) NOT NULL,
    request_hash character(64) NOT NULL,
    mode character varying(16) DEFAULT 'office' NOT NULL,
    status character varying(16) NOT NULL,
    requested_by uuid,
    operation character varying(16) NOT NULL,
    response_snapshot jsonb NOT NULL,
    actor_type character varying(16) DEFAULT 'human' NOT NULL,
    actor_name character varying(64),
    approval_instance_id uuid,
    reason text NOT NULL,
    timezone character varying(64) NOT NULL,
    range_start date NOT NULL,
    range_end date NOT NULL,
    published_at timestamp with time zone,
    affected_days integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT schedule_publication_batches_pkey PRIMARY KEY (id),
    CONSTRAINT schedule_publication_batches_requested_by_fkey FOREIGN KEY (requested_by)
        REFERENCES public.users(id) ON DELETE RESTRICT,
    CONSTRAINT schedule_publication_batches_idempotency_key_key UNIQUE (idempotency_key),
    CONSTRAINT schedule_publication_batches_request_hash_check CHECK (request_hash::text ~ '^[0-9a-f]{64}$'),
    CONSTRAINT schedule_publication_batches_operation_check CHECK (operation IN ('publish', 'replan')),
    CONSTRAINT schedule_publication_batches_response_check CHECK (jsonb_typeof(response_snapshot) = 'object'),
    CONSTRAINT schedule_publication_batches_values_check CHECK (
        mode IN ('office', 'rotating')
        AND status = 'published'
        AND actor_type IN ('human', 'system')
        AND range_start <= range_end
        AND char_length(trim(reason)) BETWEEN 1 AND 2000
        AND affected_days >= 0
        AND (
            (actor_type = 'human' AND requested_by IS NOT NULL AND actor_name IS NULL)
            OR (actor_type = 'system' AND requested_by IS NULL AND actor_name = 'schedule-publisher')
        )
        AND (
            (mode = 'office' AND approval_instance_id IS NOT NULL)
            OR (mode = 'rotating' AND approval_instance_id IS NULL)
        )
    )
);

CREATE INDEX schedule_publication_batches_range_mode_idx
    ON public.schedule_publication_batches (range_start, range_end, mode);
CREATE INDEX schedule_publication_batches_created_at_idx
    ON public.schedule_publication_batches (created_at DESC, id DESC);

CREATE TABLE public.schedule_day_revisions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employment_id uuid NOT NULL,
    employer_id uuid NOT NULL,
    business_date date NOT NULL,
    timezone character varying(64) NOT NULL,
    revision_number bigint NOT NULL,
    state character varying(16) NOT NULL,
    previous_revision_id uuid,
    publication_batch_id uuid,
    source character varying(24) NOT NULL,
    source_snapshot jsonb DEFAULT '{}'::jsonb NOT NULL,
    resolver_version character varying(32) NOT NULL,
    reason text NOT NULL,
    published_by uuid,
    published_actor_type character varying(16) DEFAULT 'human' NOT NULL,
    published_actor_name character varying(64),
    published_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT schedule_day_revisions_pkey PRIMARY KEY (id),
    CONSTRAINT schedule_day_revisions_employment_fkey FOREIGN KEY (employment_id)
        REFERENCES public.employments(id) ON DELETE RESTRICT,
    CONSTRAINT schedule_day_revisions_employer_fkey FOREIGN KEY (employer_id)
        REFERENCES public.employers(id) ON DELETE RESTRICT,
    CONSTRAINT schedule_day_revisions_previous_fkey FOREIGN KEY (previous_revision_id)
        REFERENCES public.schedule_day_revisions(id) ON DELETE RESTRICT,
    CONSTRAINT schedule_day_revisions_batch_fkey FOREIGN KEY (publication_batch_id)
        REFERENCES public.schedule_publication_batches(id) ON DELETE RESTRICT,
    CONSTRAINT schedule_day_revisions_published_by_fkey FOREIGN KEY (published_by)
        REFERENCES public.users(id) ON DELETE SET NULL,
    CONSTRAINT schedule_day_revisions_revision_key UNIQUE (employment_id, business_date, revision_number),
    CONSTRAINT schedule_day_revisions_state_check CHECK (state IN ('draft', 'published', 'superseded', 'explicit_off', 'legacy_unverified')),
    CONSTRAINT schedule_day_revisions_source_check CHECK (source IN ('roster', 'pattern', 'assignment_rule', 'fixed', 'manual', 'auto', 'legacy_unverified')),
    CONSTRAINT schedule_day_revisions_revision_number_check CHECK (revision_number >= 1),
    CONSTRAINT schedule_day_revisions_snapshot_check CHECK (jsonb_typeof(source_snapshot) = 'object'),
    CONSTRAINT schedule_day_revisions_publication_check CHECK (
        (state IN ('published', 'explicit_off', 'superseded') AND published_at IS NOT NULL)
        OR (state IN ('draft', 'legacy_unverified') AND published_at IS NULL)
    ),
    CONSTRAINT schedule_day_revisions_batch_check CHECK (
        (state = 'legacy_unverified' AND publication_batch_id IS NULL AND source_snapshot->>'provenance' = 'legacy_unverified')
        OR (state <> 'legacy_unverified' AND publication_batch_id IS NOT NULL)
    ),
    CONSTRAINT schedule_day_revisions_actor_check CHECK (published_actor_type IN ('human', 'system'))
);

CREATE UNIQUE INDEX schedule_day_revisions_current_uq
    ON public.schedule_day_revisions (employment_id, business_date)
    WHERE state IN ('published', 'explicit_off');
CREATE UNIQUE INDEX schedule_day_revisions_legacy_uq
    ON public.schedule_day_revisions (employment_id, business_date)
    WHERE state = 'legacy_unverified';
CREATE INDEX schedule_day_revisions_employment_date_revision_idx
    ON public.schedule_day_revisions (employment_id, business_date, revision_number DESC);
CREATE INDEX schedule_day_revisions_employer_date_idx
    ON public.schedule_day_revisions (employer_id, business_date);

CREATE TABLE public.attendance_periods (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employer_id uuid NOT NULL,
    period_year integer NOT NULL,
    period_month integer NOT NULL,
    status character varying(16) DEFAULT 'open' NOT NULL,
    version bigint DEFAULT 1 NOT NULL,
    current_closure_id uuid,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT attendance_periods_pkey PRIMARY KEY (id),
    CONSTRAINT attendance_periods_employer_fkey FOREIGN KEY (employer_id)
        REFERENCES public.employers(id) ON DELETE RESTRICT,
    CONSTRAINT attendance_periods_period_key UNIQUE (employer_id, period_year, period_month),
    CONSTRAINT attendance_periods_values_check CHECK (
        period_year BETWEEN 2000 AND 2100
        AND period_month BETWEEN 1 AND 12
        AND status IN ('open', 'closed', 'reopened')
        AND version >= 1
    )
);

CREATE INDEX attendance_periods_employer_period_idx
    ON public.attendance_periods (employer_id, period_year, period_month);

CREATE TABLE public.attendance_closures (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    period_id uuid NOT NULL,
    employer_id uuid NOT NULL,
    closure_number integer NOT NULL,
    status character varying(16) DEFAULT 'closed' NOT NULL,
    closed_by uuid,
    closed_actor_type character varying(16) DEFAULT 'human' NOT NULL,
    closed_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    snapshot_hash character(64) NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT attendance_closures_pkey PRIMARY KEY (id),
    CONSTRAINT attendance_closures_period_fkey FOREIGN KEY (period_id)
        REFERENCES public.attendance_periods(id) ON DELETE RESTRICT,
    CONSTRAINT attendance_closures_employer_fkey FOREIGN KEY (employer_id)
        REFERENCES public.employers(id) ON DELETE RESTRICT,
    CONSTRAINT attendance_closures_closed_by_fkey FOREIGN KEY (closed_by)
        REFERENCES public.users(id) ON DELETE SET NULL,
    CONSTRAINT attendance_closures_period_number_key UNIQUE (period_id, closure_number),
    CONSTRAINT attendance_closures_values_check CHECK (
        closure_number >= 1
        AND status = 'closed'
        AND closed_actor_type IN ('human', 'system')
        AND snapshot_hash::text ~ '^[0-9a-f]{64}$'
    )
);

ALTER TABLE ONLY public.attendance_periods
    ADD CONSTRAINT attendance_periods_current_closure_fkey
    FOREIGN KEY (current_closure_id) REFERENCES public.attendance_closures(id) ON DELETE RESTRICT;

CREATE INDEX attendance_closures_employer_closed_at_idx
    ON public.attendance_closures (employer_id, closed_at DESC);

CREATE TABLE public.attendance_closure_days (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    closure_id uuid NOT NULL,
    period_id uuid NOT NULL,
    employer_id uuid NOT NULL,
    employment_id uuid NOT NULL,
    business_date date NOT NULL,
    schedule_revision_id uuid NOT NULL,
    schedule_state character varying(16) NOT NULL,
    schedule_snapshot jsonb NOT NULL,
    attendance_snapshot jsonb NOT NULL,
    calculation_snapshot jsonb NOT NULL,
    time_entry_ids uuid[] DEFAULT '{}'::uuid[] NOT NULL,
    raw_attendance_log_ids uuid[] DEFAULT '{}'::uuid[] NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT attendance_closure_days_pkey PRIMARY KEY (id),
    CONSTRAINT attendance_closure_days_closure_fkey FOREIGN KEY (closure_id)
        REFERENCES public.attendance_closures(id) ON DELETE RESTRICT,
    CONSTRAINT attendance_closure_days_period_fkey FOREIGN KEY (period_id)
        REFERENCES public.attendance_periods(id) ON DELETE RESTRICT,
    CONSTRAINT attendance_closure_days_employer_fkey FOREIGN KEY (employer_id)
        REFERENCES public.employers(id) ON DELETE RESTRICT,
    CONSTRAINT attendance_closure_days_employment_fkey FOREIGN KEY (employment_id)
        REFERENCES public.employments(id) ON DELETE RESTRICT,
    CONSTRAINT attendance_closure_days_revision_fkey FOREIGN KEY (schedule_revision_id)
        REFERENCES public.schedule_day_revisions(id) ON DELETE RESTRICT,
    CONSTRAINT attendance_closure_days_key UNIQUE (closure_id, employment_id, business_date),
    CONSTRAINT attendance_closure_days_values_check CHECK (
        schedule_state IN ('published', 'explicit_off')
        AND schedule_revision_id IS NOT NULL
        AND jsonb_typeof(schedule_snapshot) = 'object'
        AND jsonb_typeof(attendance_snapshot) = 'object'
        AND jsonb_typeof(calculation_snapshot) = 'object'
    )
);

CREATE INDEX attendance_closure_days_employment_date_idx
    ON public.attendance_closure_days (employment_id, business_date);
CREATE INDEX attendance_closure_days_revision_idx
    ON public.attendance_closure_days (schedule_revision_id);

CREATE OR REPLACE FUNCTION public.guard_attendance_closure_immutable() RETURNS trigger
LANGUAGE plpgsql
SET search_path TO 'public'
AS $$
BEGIN
    RAISE EXCEPTION 'attendance closure evidence is immutable';
END;
$$;

CREATE TRIGGER trg_attendance_closures_immutable
BEFORE UPDATE OR DELETE ON public.attendance_closures
FOR EACH ROW EXECUTE FUNCTION public.guard_attendance_closure_immutable();

CREATE TRIGGER trg_attendance_closure_days_immutable
BEFORE UPDATE OR DELETE ON public.attendance_closure_days
FOR EACH ROW EXECUTE FUNCTION public.guard_attendance_closure_immutable();

CREATE OR REPLACE FUNCTION public.guard_schedule_day_revision() RETURNS trigger
LANGUAGE plpgsql
SET search_path TO 'public'
AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        IF OLD.state = 'draft' THEN
            RETURN OLD;
        END IF;
        RAISE EXCEPTION 'schedule day revision is immutable';
    END IF;

    IF OLD.state IN ('published', 'explicit_off')
       AND NEW.state = 'superseded'
       AND NEW.id = OLD.id
       AND NEW.employment_id = OLD.employment_id
       AND NEW.employer_id = OLD.employer_id
       AND NEW.business_date = OLD.business_date
       AND NEW.timezone = OLD.timezone
       AND NEW.revision_number = OLD.revision_number
       AND NEW.previous_revision_id IS NOT DISTINCT FROM OLD.previous_revision_id
       AND NEW.publication_batch_id IS NOT DISTINCT FROM OLD.publication_batch_id
       AND NEW.source = OLD.source
       AND NEW.source_snapshot = OLD.source_snapshot
       AND NEW.resolver_version = OLD.resolver_version
       AND NEW.reason = OLD.reason
       AND NEW.published_by IS NOT DISTINCT FROM OLD.published_by
       AND NEW.published_actor_type = OLD.published_actor_type
       AND NEW.published_actor_name IS NOT DISTINCT FROM OLD.published_actor_name
       AND NEW.published_at = OLD.published_at
       AND NEW.created_at = OLD.created_at THEN
        RETURN NEW;
    END IF;

    IF OLD.state = 'draft'
       AND NEW.state IN ('published', 'explicit_off')
       AND NEW.id = OLD.id
       AND NEW.employment_id = OLD.employment_id
       AND NEW.employer_id = OLD.employer_id
       AND NEW.business_date = OLD.business_date
       AND NEW.timezone = OLD.timezone
       AND NEW.revision_number = OLD.revision_number
       AND NEW.previous_revision_id IS NOT DISTINCT FROM OLD.previous_revision_id
       AND NEW.publication_batch_id IS NOT DISTINCT FROM OLD.publication_batch_id
       AND NEW.source = OLD.source
       AND NEW.source_snapshot = OLD.source_snapshot
       AND NEW.resolver_version = OLD.resolver_version
       AND NEW.reason = OLD.reason
       AND NEW.published_by IS NOT DISTINCT FROM OLD.published_by
       AND NEW.published_actor_type = OLD.published_actor_type
       AND NEW.published_actor_name IS NOT DISTINCT FROM OLD.published_actor_name
       AND NEW.published_at IS NOT NULL
       AND NEW.created_at = OLD.created_at THEN
        RETURN NEW;
    END IF;

    RAISE EXCEPTION 'schedule day revision is immutable outside publication transition';
END;
$$;

CREATE TRIGGER trg_schedule_day_revision_guard
BEFORE UPDATE OR DELETE ON public.schedule_day_revisions
FOR EACH ROW EXECUTE FUNCTION public.guard_schedule_day_revision();

ALTER TABLE ONLY public.work_segments
    DROP CONSTRAINT work_segments_source_check,
    ADD CONSTRAINT work_segments_source_check CHECK (
        (source)::text = ANY (ARRAY['roster'::text, 'pattern'::text, 'rule'::text, 'assignment_rule'::text, 'fixed'::text, 'manual'::text, 'auto'::text])
    ),
    ADD COLUMN schedule_revision_id uuid,
    ADD COLUMN selected_shift_version_id uuid,
    ADD COLUMN scheduled_start_at timestamp with time zone,
    ADD COLUMN scheduled_end_at timestamp with time zone,
    ADD COLUMN breaks_snapshot jsonb DEFAULT '[]'::jsonb NOT NULL;

ALTER TABLE ONLY public.work_segments
    ADD CONSTRAINT work_segments_schedule_revision_fkey FOREIGN KEY (schedule_revision_id)
        REFERENCES public.schedule_day_revisions(id) ON DELETE RESTRICT,
    ADD CONSTRAINT work_segments_selected_shift_version_fkey FOREIGN KEY (selected_shift_version_id)
        REFERENCES public.shift_versions(id) ON DELETE RESTRICT,
    ADD CONSTRAINT work_segments_schedule_values_check CHECK (
        schedule_revision_id IS NULL
        OR (
            selected_shift_version_id IS NOT NULL
            AND scheduled_start_at IS NOT NULL
            AND scheduled_end_at IS NOT NULL
            AND scheduled_end_at > scheduled_start_at
        )
    ),
    ADD CONSTRAINT work_segments_breaks_snapshot_check CHECK (jsonb_typeof(breaks_snapshot) = 'array');

DROP INDEX public.idx_work_segments_interval_unique;
CREATE UNIQUE INDEX work_segments_legacy_interval_uq
    ON public.work_segments (employment_id, date, interval_id)
    WHERE schedule_revision_id IS NULL AND interval_id IS NOT NULL;
CREATE INDEX work_segments_schedule_revision_sort_idx
    ON public.work_segments (schedule_revision_id, sort_order, id);
CREATE UNIQUE INDEX work_segments_official_interval_uq
    ON public.work_segments (schedule_revision_id, interval_id)
    WHERE schedule_revision_id IS NOT NULL AND interval_id IS NOT NULL;

CREATE OR REPLACE FUNCTION public.guard_schedule_work_segment() RETURNS trigger
LANGUAGE plpgsql
SET search_path TO 'public'
AS $$
DECLARE
    revision_state character varying(16);
BEGIN
    IF TG_OP IN ('UPDATE', 'DELETE') AND OLD.schedule_revision_id IS NOT NULL THEN
        RAISE EXCEPTION 'official work segment is immutable';
    END IF;

    IF TG_OP = 'DELETE' THEN
        IF EXISTS (
            SELECT 1
            FROM public.schedule_day_revisions
            WHERE employment_id = OLD.employment_id
              AND business_date = OLD.date
              AND state IN ('published', 'explicit_off')
        ) THEN
            RAISE EXCEPTION 'legacy work segment mutation blocked for official schedule date';
        END IF;
        RETURN OLD;
    END IF;

    IF NEW.schedule_revision_id IS NOT NULL THEN
        SELECT state INTO revision_state
        FROM public.schedule_day_revisions
        WHERE id = NEW.schedule_revision_id;

        IF revision_state IS NULL THEN
            RAISE EXCEPTION 'schedule revision does not exist';
        END IF;
        IF revision_state <> 'draft'
           AND COALESCE(current_setting('schedule.lifecycle_publication', true), '') <> 'on' THEN
            RAISE EXCEPTION 'official work segment insert requires draft revision or publication transition';
        END IF;
        IF NOT EXISTS (
            SELECT 1
            FROM public.schedule_day_revisions
            WHERE id = NEW.schedule_revision_id
              AND employment_id = NEW.employment_id
              AND business_date = NEW.date
        ) THEN
            RAISE EXCEPTION 'work segment employment/date does not match schedule revision';
        END IF;
        RETURN NEW;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM public.schedule_day_revisions
        WHERE employment_id = NEW.employment_id
          AND business_date = NEW.date
          AND state IN ('published', 'explicit_off')
    ) THEN
        RAISE EXCEPTION 'legacy work segment mutation blocked for official schedule date';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_schedule_work_segment_guard
BEFORE INSERT OR UPDATE OR DELETE ON public.work_segments
FOR EACH ROW EXECUTE FUNCTION public.guard_schedule_work_segment();

ALTER TABLE ONLY public.outbox_jobs
    ADD COLUMN dedupe_key text,
    ADD COLUMN lease_token uuid,
    ADD COLUMN locked_until timestamp with time zone,
    ADD COLUMN attempts integer DEFAULT 0 NOT NULL,
    ADD COLUMN available_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    ADD COLUMN last_error text,
    ADD COLUMN dead_letter_at timestamp with time zone;

ALTER TABLE ONLY public.outbox_jobs
    ADD CONSTRAINT outbox_jobs_attempts_check CHECK (attempts >= 0);

CREATE UNIQUE INDEX outbox_jobs_dedupe_key_uq
    ON public.outbox_jobs (dedupe_key)
    WHERE dedupe_key IS NOT NULL;
CREATE INDEX outbox_jobs_type_status_available_idx
    ON public.outbox_jobs (job_type, status, available_at);

CREATE TRIGGER trg_schedule_publication_batches_updated_at
BEFORE UPDATE ON public.schedule_publication_batches
FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

CREATE TRIGGER trg_attendance_periods_updated_at
BEFORE UPDATE ON public.attendance_periods
FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'public.work_segments'::regclass
          AND conname = 'work_segments_source_check'
          AND pg_get_constraintdef(oid) LIKE '%assignment_rule%'
    ) THEN
        RAISE EXCEPTION 'schedule lifecycle migration check failed: work_segments source constraint lacks assignment_rule';
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_trigger
        WHERE tgrelid = 'public.schedule_day_revisions'::regclass
          AND tgname = 'trg_schedule_day_revision_guard'
          AND NOT tgisinternal
    ) THEN
        RAISE EXCEPTION 'schedule lifecycle migration check failed: revision immutability trigger missing';
    END IF;
END;
$$;

INSERT INTO public.schema_migrations(version)
VALUES ('20261113000000_schedule_lifecycle');

COMMIT;

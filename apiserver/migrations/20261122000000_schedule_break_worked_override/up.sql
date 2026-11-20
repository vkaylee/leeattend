BEGIN;

CREATE TABLE public.schedule_day_overrides (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employment_id uuid NOT NULL,
    business_date date NOT NULL,
    work_segment_id uuid NOT NULL,
    override_type text NOT NULL,
    start_at timestamp with time zone NOT NULL,
    end_at timestamp with time zone NOT NULL,
    minutes integer NOT NULL,
    reason text NOT NULL,
    status text NOT NULL DEFAULT 'approved',
    created_by uuid NOT NULL,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT schedule_day_overrides_pkey PRIMARY KEY (id),
    CONSTRAINT schedule_day_overrides_employment_fkey FOREIGN KEY (employment_id) REFERENCES public.employments(id) ON DELETE RESTRICT,
    CONSTRAINT schedule_day_overrides_segment_fkey FOREIGN KEY (work_segment_id) REFERENCES public.work_segments(id) ON DELETE RESTRICT,
    CONSTRAINT schedule_day_overrides_creator_fkey FOREIGN KEY (created_by) REFERENCES public.users(id) ON DELETE RESTRICT,
    CONSTRAINT schedule_day_overrides_type_check CHECK (override_type = 'break_worked'),
    CONSTRAINT schedule_day_overrides_status_check CHECK (status IN ('pending', 'approved', 'rejected')),
    CONSTRAINT schedule_day_overrides_time_check CHECK (end_at > start_at AND minutes > 0),
    CONSTRAINT schedule_day_overrides_reason_check CHECK (length(btrim(reason)) BETWEEN 1 AND 2000)
);

CREATE INDEX schedule_day_overrides_scope_idx
    ON public.schedule_day_overrides (employment_id, business_date, work_segment_id);

INSERT INTO public.schema_migrations(version)
VALUES ('20261122000000_schedule_break_worked_override');

COMMIT;

BEGIN;

CREATE TABLE public.attendance_punch_events (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    employment_id uuid NOT NULL,
    time_entry_id uuid NOT NULL,
    event_type text NOT NULL,
    occurred_at timestamp with time zone NOT NULL,
    idempotency_key text NOT NULL,
    source text NOT NULL DEFAULT 'ess',
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT attendance_punch_events_pkey PRIMARY KEY (id),
    CONSTRAINT attendance_punch_events_employment_fkey FOREIGN KEY (employment_id)
        REFERENCES public.employments(id) ON DELETE RESTRICT,
    CONSTRAINT attendance_punch_events_time_entry_fkey FOREIGN KEY (time_entry_id)
        REFERENCES public.time_entries(id) ON DELETE RESTRICT,
    CONSTRAINT attendance_punch_events_type_check CHECK (
        event_type IN ('work_in', 'break_start', 'break_end', 'work_out')
    ),
    CONSTRAINT attendance_punch_events_source_check CHECK (source IN ('ess', 'manual')),
    CONSTRAINT attendance_punch_events_key_check CHECK (length(btrim(idempotency_key)) BETWEEN 1 AND 128),
    CONSTRAINT attendance_punch_events_entry_key_key UNIQUE (employment_id, idempotency_key)
);

CREATE INDEX attendance_punch_events_entry_time_idx
    ON public.attendance_punch_events (time_entry_id, occurred_at, id);

CREATE INDEX attendance_punch_events_employment_time_idx
    ON public.attendance_punch_events (employment_id, occurred_at, id);

CREATE OR REPLACE FUNCTION public.guard_attendance_punch_events_immutable() RETURNS trigger
LANGUAGE plpgsql
SET search_path TO 'public'
AS $$
BEGIN
    RAISE EXCEPTION 'attendance punch events are immutable';
END;
$$;

CREATE TRIGGER trg_attendance_punch_events_immutable
BEFORE UPDATE OR DELETE ON public.attendance_punch_events
FOR EACH ROW EXECUTE FUNCTION public.guard_attendance_punch_events_immutable();

INSERT INTO public.schema_migrations(version)
VALUES ('20261121000000_attendance_punch_events');

COMMIT;

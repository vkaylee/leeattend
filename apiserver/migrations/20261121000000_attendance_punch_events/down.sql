BEGIN;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM public.attendance_punch_events) THEN
        RAISE EXCEPTION 'refusing rollback: attendance punch event evidence would be lost';
    END IF;
END;
$$;

DROP TRIGGER IF EXISTS trg_attendance_punch_events_immutable ON public.attendance_punch_events;
DROP FUNCTION IF EXISTS public.guard_attendance_punch_events_immutable();
DROP INDEX IF EXISTS attendance_punch_events_employment_time_idx;
DROP INDEX IF EXISTS attendance_punch_events_entry_time_idx;
DROP TABLE IF EXISTS public.attendance_punch_events;

DELETE FROM public.schema_migrations
WHERE version = '20261121000000_attendance_punch_events';

COMMIT;

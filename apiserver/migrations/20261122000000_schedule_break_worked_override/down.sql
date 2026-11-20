BEGIN;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM public.schedule_day_overrides) THEN
        RAISE EXCEPTION 'refusing rollback: approved schedule day override evidence would be lost';
    END IF;
END;
$$;

DROP INDEX IF EXISTS schedule_day_overrides_scope_idx;
DROP TABLE IF EXISTS public.schedule_day_overrides;

DELETE FROM public.schema_migrations
WHERE version = '20261122000000_schedule_break_worked_override';

COMMIT;

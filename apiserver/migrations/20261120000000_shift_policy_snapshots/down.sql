BEGIN;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM public.shift_rosters WHERE shift_version_id IS NOT NULL)
       OR EXISTS (SELECT 1 FROM public.time_entries WHERE shift_version_id IS NOT NULL OR breaks_snapshot IS NOT NULL)
       OR EXISTS (SELECT 1 FROM public.attendance_closure_days WHERE time_entry_snapshot <> '[]'::jsonb) THEN
        RAISE EXCEPTION 'refusing rollback: shift policy snapshot evidence would be lost';
    END IF;
END;
$$;

ALTER TABLE public.time_entries
    DROP CONSTRAINT IF EXISTS time_entries_shift_version_id_fkey,
    DROP COLUMN IF EXISTS breaks_snapshot,
    DROP COLUMN IF EXISTS shift_version_id;

ALTER TABLE public.attendance_closure_days
    DROP COLUMN IF EXISTS time_entry_snapshot;

ALTER TABLE public.shift_rosters
    DROP CONSTRAINT IF EXISTS shift_rosters_shift_version_id_fkey,
    DROP COLUMN IF EXISTS shift_version_id;

DROP INDEX IF EXISTS idx_shift_rosters_version_lookup;

DELETE FROM public.schema_migrations
WHERE version = '20261120000000_shift_policy_snapshots';

COMMIT;

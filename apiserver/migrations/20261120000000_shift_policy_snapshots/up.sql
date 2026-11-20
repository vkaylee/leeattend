BEGIN;

ALTER TABLE public.shift_rosters
    ADD COLUMN IF NOT EXISTS shift_version_id uuid;

ALTER TABLE public.attendance_closure_days
    ADD COLUMN IF NOT EXISTS time_entry_snapshot jsonb NOT NULL DEFAULT '[]'::jsonb;

ALTER TABLE public.time_entries
    ADD COLUMN IF NOT EXISTS shift_version_id uuid,
    ADD COLUMN IF NOT EXISTS breaks_snapshot jsonb;

COMMENT ON COLUMN public.shift_rosters.shift_version_id IS
    'NULL means legacy, ambiguous, or not-yet-materialized roster policy; never an implicit live version.';
COMMENT ON COLUMN public.time_entries.shift_version_id IS
    'NULL preserves historical entries without a materialized policy snapshot.';
COMMENT ON COLUMN public.time_entries.breaks_snapshot IS
    'NULL preserves historical entries without a materialized break snapshot.';

DO $$
DECLARE
    updated_count bigint;
    unresolved_count bigint;
BEGIN
    LOOP
        WITH candidates AS (
            SELECT r.id, (array_agg(v.id ORDER BY v.id))[1] AS version_id
            FROM public.shift_rosters r
            JOIN public.shift_versions v
              ON v.template_id = r.template_id
             AND v.effective_from <= r.date
             AND (v.effective_to IS NULL OR v.effective_to >= r.date)
            WHERE r.shift_version_id IS NULL
              AND r.template_id IS NOT NULL
            GROUP BY r.id
            HAVING count(*) = 1
            ORDER BY r.date, r.id
            LIMIT 1000
        ), updated AS (
            UPDATE public.shift_rosters r
            SET shift_version_id = c.version_id
            FROM candidates c
            WHERE r.id = c.id
            RETURNING 1
        )
        SELECT count(*) INTO updated_count FROM updated;

        EXIT WHEN updated_count = 0;
    END LOOP;

    SELECT count(*) INTO unresolved_count
    FROM public.shift_rosters
    WHERE shift_version_id IS NULL
      AND template_id IS NOT NULL;
    RAISE NOTICE 'shift policy snapshot backfill unresolved roster rows: %', unresolved_count;
END;
$$;

CREATE INDEX IF NOT EXISTS idx_shift_rosters_version_lookup
    ON public.shift_rosters (employment_id, date, shift_version_id);

ALTER TABLE public.shift_rosters
    ADD CONSTRAINT shift_rosters_shift_version_id_fkey
    FOREIGN KEY (shift_version_id)
    REFERENCES public.shift_versions(id)
    ON DELETE RESTRICT;

ALTER TABLE public.time_entries
    ADD CONSTRAINT time_entries_shift_version_id_fkey
    FOREIGN KEY (shift_version_id)
    REFERENCES public.shift_versions(id)
    ON DELETE RESTRICT;

INSERT INTO public.schema_migrations(version)
VALUES ('20261120000000_shift_policy_snapshots');

COMMIT;

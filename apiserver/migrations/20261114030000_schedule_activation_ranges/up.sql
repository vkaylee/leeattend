BEGIN;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM public.config_entries a
        JOIN public.config_entries b
          ON b.id > a.id
         AND b.scope_type = a.scope_type
         AND b.scope_id IS NOT DISTINCT FROM a.scope_id
         AND b.config_key = a.config_key
         AND daterange(b.effective_from, COALESCE(b.effective_to + 1, 'infinity'::date), '[)')
             && daterange(a.effective_from, COALESCE(a.effective_to + 1, 'infinity'::date), '[)')
        WHERE a.scope_type = 'employer'
          AND a.config_key = 'schedule.lifecycle.enabled'
    ) THEN
        RAISE EXCEPTION 'schedule lifecycle activation migration blocked: overlapping employer effective ranges exist';
    END IF;
END;
$$;

ALTER TABLE public.config_entries
    ADD CONSTRAINT config_entries_schedule_lifecycle_effective_no_overlap
    EXCLUDE USING gist (
        scope_type WITH =,
        scope_id WITH =,
        config_key WITH =,
        daterange(effective_from, COALESCE(effective_to + 1, 'infinity'::date), '[)') WITH &&
    )
    WHERE (scope_type = 'employer' AND config_key = 'schedule.lifecycle.enabled');

INSERT INTO public.schema_migrations(version)
VALUES ('20261114030000_schedule_activation_ranges');

COMMIT;

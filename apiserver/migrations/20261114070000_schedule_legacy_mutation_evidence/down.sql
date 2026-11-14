BEGIN;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM public.schedule_legacy_mutation_evidence) THEN
        RAISE EXCEPTION 'legacy mutation evidence rollback blocked: evidence exists';
    END IF;
END;
$$;

DROP INDEX public.schedule_legacy_mutation_evidence_scope_idx;
DROP TABLE public.schedule_legacy_mutation_evidence;

DELETE FROM public.schema_migrations
WHERE version = '20261114070000_schedule_legacy_mutation_evidence';

COMMIT;

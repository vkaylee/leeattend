BEGIN;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM public.attendance_adjustment_evidence) THEN
        RAISE EXCEPTION 'refusing rollback: attendance adjustment evidence would be lost';
    END IF;
END;
$$;

DROP INDEX IF EXISTS attendance_adjustment_evidence_scope_idx;
DROP INDEX IF EXISTS attendance_adjustment_evidence_request_idx;
DROP TABLE IF EXISTS public.attendance_adjustment_evidence;

DROP INDEX IF EXISTS attendance_adjustment_requests_type_idx;
ALTER TABLE public.attendance_adjustment_requests
    DROP CONSTRAINT IF EXISTS attendance_adjustment_requests_details_check,
    DROP CONSTRAINT IF EXISTS attendance_adjustment_requests_type_check,
    DROP COLUMN IF EXISTS details,
    DROP COLUMN IF EXISTS adjustment_type;

DELETE FROM public.schema_migrations
WHERE version = '20261123000000_typed_attendance_adjustments';

COMMIT;

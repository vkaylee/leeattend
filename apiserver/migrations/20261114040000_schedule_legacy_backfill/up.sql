BEGIN;

ALTER TABLE public.schedule_day_revisions
    ALTER COLUMN state TYPE varchar(24);

-- Capture legacy schedule provenance without promoting it to official evidence.
-- One row per legacy employment/day; reruns skip keys already captured.
WITH legacy_days AS (
    SELECT employment_id, date AS business_date
    FROM public.work_segments
    UNION
    SELECT employment_id, date AS business_date
    FROM public.shift_rosters
), locked_employments AS (
    SELECT e.id, e.employer_id
    FROM public.employments e
    WHERE e.deleted_at IS NULL
      AND EXISTS (
          SELECT 1
          FROM legacy_days d
          WHERE d.employment_id = e.id
      )
    FOR UPDATE
), candidates AS (
    SELECT
        d.employment_id,
        d.business_date,
        e.employer_id,
        jsonb_build_object(
            'provenance', 'legacy_unverified',
            'timezone', 'unknown',
            'timezone_unknown', true,
            'work_segment_ids', COALESCE((
                SELECT jsonb_agg(to_jsonb(ws.id) ORDER BY ws.sort_order, ws.id)
                FROM public.work_segments ws
                WHERE ws.employment_id = d.employment_id AND ws.date = d.business_date
            ), '[]'::jsonb),
            'roster_entries', COALESCE((
                SELECT jsonb_agg(jsonb_build_object(
                    'id', sr.id,
                    'template_id', sr.template_id,
                    'status', sr.status
                ) ORDER BY sr.id)
                FROM public.shift_rosters sr
                WHERE sr.employment_id = d.employment_id AND sr.date = d.business_date
            ), '[]'::jsonb)
        ) AS source_snapshot
    FROM legacy_days d
    JOIN locked_employments e ON e.id = d.employment_id
      AND NOT EXISTS (
          SELECT 1
          FROM public.schedule_day_revisions existing
          WHERE existing.employment_id = d.employment_id
            AND existing.business_date = d.business_date
            AND existing.state = 'legacy_unverified'
      )
)
INSERT INTO public.schedule_day_revisions (
    id,
    employment_id,
    employer_id,
    business_date,
    timezone,
    revision_number,
    state,
    previous_revision_id,
    publication_batch_id,
    source,
    source_snapshot,
    resolver_version,
    reason,
    published_by,
    published_actor_type,
    published_actor_name,
    published_at
)
SELECT
    gen_random_uuid(),
    c.employment_id,
    c.employer_id,
    c.business_date,
    'UTC',
    COALESCE((
        SELECT MAX(r.revision_number)
        FROM public.schedule_day_revisions r
        WHERE r.employment_id = c.employment_id
          AND r.business_date = c.business_date
    ), 0) + 1,
    'legacy_unverified',
    NULL,
    NULL,
    'legacy_unverified',
    c.source_snapshot,
    'legacy-import-v1',
    'Legacy schedule provenance capture',
    NULL,
    'system',
    'backfill',
    NULL
FROM candidates c;

INSERT INTO public.schema_migrations(version)
VALUES ('20261114040000_schedule_legacy_backfill');

COMMIT;

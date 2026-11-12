BEGIN;

-- Map only unambiguous employee-employer histories. Multiple segments may represent
-- transfer, gap, or rehire; leave those rows for explicit reconciliation.
CREATE TEMP TABLE employment_relationship_backfill_map (
    segment_id uuid PRIMARY KEY,
    relationship_id uuid NOT NULL UNIQUE
) ON COMMIT DROP;

INSERT INTO employment_relationship_backfill_map (segment_id, relationship_id)
SELECT segment.id, gen_random_uuid()
FROM public.employments AS segment
WHERE segment.relationship_id IS NULL
  AND NOT EXISTS (
      SELECT 1
      FROM public.employments AS other
      WHERE other.employee_id = segment.employee_id
        AND other.employer_id = segment.employer_id
        AND other.id <> segment.id
  );

INSERT INTO public.employment_relationships (
    id,
    employee_id,
    employer_id,
    status,
    start_date,
    end_date,
    termination_reason,
    deleted_at
)
SELECT map.relationship_id,
       segment.employee_id,
       segment.employer_id,
       segment.status,
       segment.start_date,
       segment.end_date,
       segment.termination_reason,
       segment.deleted_at
FROM employment_relationship_backfill_map AS map
JOIN public.employments AS segment ON segment.id = map.segment_id;

UPDATE public.employments AS segment
SET relationship_id = map.relationship_id
FROM employment_relationship_backfill_map AS map
WHERE segment.id = map.segment_id;

UPDATE public.employee_contracts AS contract
SET relationship_id = segment.relationship_id
FROM public.employments AS segment
WHERE contract.employment_id = segment.id
  AND contract.relationship_id IS NULL
  AND segment.relationship_id IS NOT NULL
  AND segment.deleted_at IS NULL;

INSERT INTO public.schema_migrations(version)
VALUES ('20261112020000_employment_relationship_backfill');

COMMIT;

BEGIN;

-- Keep newest write for each logical key/date before enforcing NULL-safe uniqueness.
WITH ranked AS (
    SELECT id,
           row_number() OVER (
               PARTITION BY scope_type, scope_id, config_key, effective_from
               ORDER BY updated_at DESC, created_at DESC, id DESC
           ) AS row_number
    FROM public.config_entries
)
DELETE FROM public.config_entries AS entries
USING ranked
WHERE entries.id = ranked.id
  AND ranked.row_number > 1;

ALTER TABLE ONLY public.config_entries
    DROP CONSTRAINT organization_config_entries_organization_scope_type_scope_id_c_;

ALTER TABLE ONLY public.config_entries
    ADD CONSTRAINT organization_config_entries_organization_scope_type_scope_id_c_
    UNIQUE NULLS NOT DISTINCT (scope_type, scope_id, config_key, effective_from);

ALTER TABLE ONLY public.config_entries
    ADD CONSTRAINT config_entries_effective_date_range_check
    CHECK (effective_to IS NULL OR effective_to >= effective_from);

INSERT INTO public.schema_migrations(version)
VALUES ('20261112000000_config_entries_integrity');

COMMIT;

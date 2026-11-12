BEGIN;

ALTER TABLE ONLY public.config_entries
    DROP CONSTRAINT config_entries_effective_date_range_check;

ALTER TABLE ONLY public.config_entries
    DROP CONSTRAINT organization_config_entries_organization_scope_type_scope_id_c_;

ALTER TABLE ONLY public.config_entries
    ADD CONSTRAINT organization_config_entries_organization_scope_type_scope_id_c_
    UNIQUE (scope_type, scope_id, config_key, effective_from);

DELETE FROM public.schema_migrations
WHERE version = '20261112000000_config_entries_integrity';

COMMIT;

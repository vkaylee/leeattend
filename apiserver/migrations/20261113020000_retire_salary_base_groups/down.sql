BEGIN;

-- Restore original IDs, metadata, membership and timestamps, never synthesized seed data.
-- Missing snapshots or deleted component definitions fail atomically; keep backup and roll forward.
CREATE TABLE public.salary_base_groups (
    id uuid DEFAULT gen_random_uuid() NOT NULL PRIMARY KEY,
    name varchar(255) NOT NULL,
    slug varchar(100) NOT NULL,
    description text,
    is_system boolean DEFAULT false NOT NULL,
    created_at timestamptz DEFAULT now() NOT NULL,
    updated_at timestamptz DEFAULT now() NOT NULL
);
CREATE UNIQUE INDEX salary_base_groups_slug_key ON public.salary_base_groups (slug);
CREATE TABLE public.salary_component_base_members (
    component_definition_id uuid NOT NULL REFERENCES public.salary_component_definitions(id) ON DELETE CASCADE,
    base_group_id uuid NOT NULL REFERENCES public.salary_base_groups(id) ON DELETE CASCADE,
    created_at timestamptz DEFAULT now() NOT NULL,
    PRIMARY KEY (component_definition_id, base_group_id)
);
INSERT INTO public.salary_base_groups SELECT * FROM salary_base_retirement_backup.groups;
INSERT INTO public.salary_component_base_members SELECT * FROM salary_base_retirement_backup.members;
CREATE TRIGGER trg_updated_at BEFORE UPDATE ON public.salary_base_groups
    FOR EACH ROW EXECUTE FUNCTION public.fn_set_updated_at();

DROP TABLE salary_base_retirement_backup.members;
DROP TABLE salary_base_retirement_backup.groups;
DROP SCHEMA salary_base_retirement_backup;
DELETE FROM public.schema_migrations
WHERE version = '20261113020000_retire_salary_base_groups';

COMMIT;

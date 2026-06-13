DROP INDEX IF EXISTS public.idx_behalf_assignments_employer;
ALTER TABLE public.behalf_assignments DROP CONSTRAINT IF EXISTS behalf_assignments_employer_id_fkey;
ALTER TABLE public.behalf_assignments DROP COLUMN employer_id;

-- Revert workspace_config_entries_scope_type_check constraint
ALTER TABLE public.workspace_config_entries
    DROP CONSTRAINT IF EXISTS workspace_config_entries_scope_type_check;

ALTER TABLE public.workspace_config_entries
    ADD CONSTRAINT workspace_config_entries_scope_type_check
    CHECK (((scope_type)::text = ANY ((ARRAY['workspace'::character varying, 'employer'::character varying, 'department'::character varying, 'employee'::character varying])::text[])));

CREATE OR REPLACE FUNCTION public.fn_validate_target_fk() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.target_id IS NULL THEN
        RAISE EXCEPTION 'target_id cannot be NULL';
    END IF;

    IF NEW.target_type = 'USER' THEN
        IF NOT EXISTS (SELECT 1 FROM public.employees WHERE id = NEW.target_id) THEN
            RAISE EXCEPTION 'Invalid target_id: employee % does not exist', NEW.target_id;
        END IF;
    ELSIF NEW.target_type = 'GROUP' THEN
        IF NOT EXISTS (SELECT 1 FROM public.departments WHERE id = NEW.target_id) THEN
            RAISE EXCEPTION 'Invalid target_id: department % does not exist', NEW.target_id;
        END IF;
    ELSE
        RAISE EXCEPTION 'Unknown target_type: %', NEW.target_type;
    END IF;

    RETURN NEW;
END;
$$;

-- Phase 3: Add employer_id to behalf_assignments
ALTER TABLE public.behalf_assignments ADD COLUMN employer_id UUID;
ALTER TABLE public.behalf_assignments
    ADD CONSTRAINT behalf_assignments_employer_id_fkey
    FOREIGN KEY (employer_id) REFERENCES public.employers(id) ON DELETE SET NULL;

CREATE INDEX idx_behalf_assignments_employer ON public.behalf_assignments USING btree (employer_id);

-- Alter workspace_config_entries_scope_type_check constraint to allow 'employment'
ALTER TABLE public.workspace_config_entries
    DROP CONSTRAINT IF EXISTS workspace_config_entries_scope_type_check;

ALTER TABLE public.workspace_config_entries
    ADD CONSTRAINT workspace_config_entries_scope_type_check
    CHECK (((scope_type)::text = ANY ((ARRAY['workspace'::character varying, 'employer'::character varying, 'department'::character varying, 'employee'::character varying, 'employment'::character varying])::text[])));

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
    ELSIF NEW.target_type = 'EMPLOYMENT' THEN
        IF NOT EXISTS (SELECT 1 FROM public.employments WHERE id = NEW.target_id) THEN
            RAISE EXCEPTION 'Invalid target_id: employment % does not exist', NEW.target_id;
        END IF;
    ELSE
        RAISE EXCEPTION 'Unknown target_type: %', NEW.target_type;
    END IF;

    RETURN NEW;
END;
$$;

-- ============================================================================
-- Phase 1: Condition Type Catalog
-- Data-driven workspace-scoped catalog for working condition types.
-- Replaces hardcoded frontend enums and backend string matching.
-- ============================================================================

-- 1. Create condition_type_definitions table
CREATE TABLE public.condition_type_definitions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    workspace_id uuid NOT NULL,
    code character varying(50) NOT NULL,
    name_i18n jsonb NOT NULL DEFAULT '{}'::jsonb,
    category character varying(50) NOT NULL DEFAULT 'general'::character varying,
    default_minute_allowance integer DEFAULT 0 NOT NULL,
    default_is_paid boolean DEFAULT false NOT NULL,
    affects_rules jsonb NOT NULL DEFAULT '{}'::jsonb,
    icon character varying(50),
    color character varying(20),
    is_system boolean DEFAULT false NOT NULL,
    sort_order integer DEFAULT 0 NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,

    CONSTRAINT condition_type_definitions_pkey PRIMARY KEY (id),
    CONSTRAINT condition_type_definitions_workspace_id_code_key UNIQUE (workspace_id, code),
    CONSTRAINT condition_type_definitions_valid_code CHECK ((code)::text ~ '^[A-Z][A-Z0-9_]*$'::text),
    CONSTRAINT condition_type_definitions_workspace_id_fkey FOREIGN KEY (workspace_id)
        REFERENCES public.workspaces(id) ON DELETE CASCADE
);

COMMENT ON COLUMN public.condition_type_definitions.code IS 'Immutable uppercase machine key (e.g., EARLY_LEAVE, MATERNITY)';
COMMENT ON COLUMN public.condition_type_definitions.name_i18n IS 'I18n names, e.g. {"en":"Early Leave","vi":"Về sớm"}';
COMMENT ON COLUMN public.condition_type_definitions.affects_rules IS 'JSON map of PolicyRules fields this type affects, e.g. {"authorized_early_leave_minutes":true,"is_early_leave_paid":true}';
COMMENT ON COLUMN public.condition_type_definitions.is_system IS 'System types cannot be deleted by users';
COMMENT ON COLUMN public.condition_type_definitions.category IS 'Grouping: working_hours, leave, general';

-- 2. Add condition_type_id FK to working_conditions (nullable, backward compat)
ALTER TABLE public.working_conditions
    ADD COLUMN condition_type_id uuid;

COMMENT ON COLUMN public.working_conditions.condition_type_id IS 'FK to condition_type_definitions. Nullable for backward compatibility with existing string-based type_ field.';

ALTER TABLE ONLY public.working_conditions
    ADD CONSTRAINT working_conditions_condition_type_id_fkey
        FOREIGN KEY (condition_type_id)
        REFERENCES public.condition_type_definitions(id) ON DELETE SET NULL;

CREATE INDEX idx_working_conditions_condition_type_id
    ON public.working_conditions USING btree (condition_type_id);

-- 3. Seed system condition types for all existing workspaces
DO $$
DECLARE
    ws RECORD;
BEGIN
    FOR ws IN SELECT id FROM public.workspaces
    LOOP
        -- EARLY_LEAVE: authorized early leave minutes, paid/unpaid
        INSERT INTO public.condition_type_definitions (workspace_id, code, name_i18n, category, default_minute_allowance, default_is_paid, affects_rules, icon, color, is_system, sort_order)
        VALUES (ws.id, 'EARLY_LEAVE',
            '{"en":"Early Leave","vi":"Về sớm"}',
            'working_hours', 60, false,
            '{"authorized_early_leave_minutes":true,"is_early_leave_paid":true}',
            'Clock', 'blue', true, 1)
        ON CONFLICT (workspace_id, code) DO NOTHING;

        -- BREASTFEEDING: extra paid break/nursing leave (always paid)
        INSERT INTO public.condition_type_definitions (workspace_id, code, name_i18n, category, default_minute_allowance, default_is_paid, affects_rules, icon, color, is_system, sort_order)
        VALUES (ws.id, 'BREASTFEEDING',
            '{"en":"Breastfeeding","vi":"Cho con bú"}',
            'working_hours', 60, true,
            '{"authorized_early_leave_minutes":true,"is_early_leave_paid":true}',
            'Baby', 'pink', true, 2)
        ON CONFLICT (workspace_id, code) DO NOTHING;

        -- MATERNITY: maternity leave (paid)
        INSERT INTO public.condition_type_definitions (workspace_id, code, name_i18n, category, default_minute_allowance, default_is_paid, affects_rules, icon, color, is_system, sort_order)
        VALUES (ws.id, 'MATERNITY',
            '{"en":"Maternity","vi":"Thai sản"}',
            'leave', 0, true,
            '{}',
            'Heart', 'purple', true, 3)
        ON CONFLICT (workspace_id, code) DO NOTHING;

        -- SICK: sick leave (paid)
        INSERT INTO public.condition_type_definitions (workspace_id, code, name_i18n, category, default_minute_allowance, default_is_paid, affects_rules, icon, color, is_system, sort_order)
        VALUES (ws.id, 'SICK',
            '{"en":"Sick","vi":"Ốm đau"}',
            'leave', 0, true,
            '{}',
            'ShieldAlert', 'red', true, 4)
        ON CONFLICT (workspace_id, code) DO NOTHING;

        -- UNPAID: unpaid leave
        INSERT INTO public.condition_type_definitions (workspace_id, code, name_i18n, category, default_minute_allowance, default_is_paid, affects_rules, icon, color, is_system, sort_order)
        VALUES (ws.id, 'UNPAID',
            '{"en":"Unpaid Leave","vi":"Nghỉ không lương"}',
            'leave', 0, false,
            '{}',
            'Ban', 'gray', true, 5)
        ON CONFLICT (workspace_id, code) DO NOTHING;
    END LOOP;
END $$;

-- 4. Backfill: set condition_type_id for existing working_conditions where "type" matches a system code
UPDATE public.working_conditions wc
SET condition_type_id = ctd.id
FROM public.condition_type_definitions ctd, public.employments emp
WHERE emp.id = wc.employment_id
  AND ctd.workspace_id = emp.workspace_id
  AND UPPER(wc."type") = ctd.code
  AND wc.condition_type_id IS NULL;

-- 5. Trigger: auto-seed system condition types when a new workspace is created
CREATE OR REPLACE FUNCTION public.fn_seed_workspace_condition_types()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    INSERT INTO public.condition_type_definitions (workspace_id, code, name_i18n, category, default_minute_allowance, default_is_paid, affects_rules, icon, color, is_system, sort_order)
    VALUES
        (NEW.id, 'EARLY_LEAVE',   '{"en":"Early Leave","vi":"Về sớm"}',              'working_hours', 60, false, '{"authorized_early_leave_minutes":true,"is_early_leave_paid":true}', 'Clock',       'blue',    true, 1),
        (NEW.id, 'BREASTFEEDING', '{"en":"Breastfeeding","vi":"Cho con bú"}',         'working_hours', 60, true,  '{"authorized_early_leave_minutes":true,"is_early_leave_paid":true}', 'Baby',        'pink',    true, 2),
        (NEW.id, 'MATERNITY',     '{"en":"Maternity","vi":"Thai sản"}',               'leave',          0, true,  '{}',                                                                   'Heart',       'purple',  true, 3),
        (NEW.id, 'SICK',          '{"en":"Sick","vi":"Ốm đau"}',                        'leave',          0, true,  '{}',                                                                   'ShieldAlert', 'red',     true, 4),
        (NEW.id, 'UNPAID',        '{"en":"Unpaid Leave","vi":"Nghỉ không lương"}',    'leave',          0, false, '{}',                                                                   'Ban',         'gray',    true, 5)
    ON CONFLICT (workspace_id, code) DO NOTHING;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_seed_workspace_condition_types ON public.workspaces;
CREATE TRIGGER trg_seed_workspace_condition_types
    AFTER INSERT ON public.workspaces
    FOR EACH ROW
    EXECUTE FUNCTION public.fn_seed_workspace_condition_types();

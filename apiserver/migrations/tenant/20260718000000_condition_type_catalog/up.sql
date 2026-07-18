-- ============================================================================
-- Phase 1: Condition Type Catalog
-- Data-driven workspace-scoped catalog for working condition types.
-- Replaces hardcoded frontend enums and backend string matching.
-- ============================================================================

-- 1. Create condition_type_definitions table
-- NOTE: workspace_id is a logical scoping column. There is no FK to a
-- workspace table in the tenant DB because workspace metadata lives in
-- the system DB (database-per-tenant architecture). System-type seeding
-- happens in application code (tenant provisioner, startup reconciler,
-- test factory) rather than in this DDL.
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
    CONSTRAINT condition_type_definitions_valid_code CHECK ((code)::text ~ '^[A-Z][A-Z0-9_]*$'::text)
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

-- 3. Backfill: set condition_type_id for existing working_conditions where
--    "type" matches a system code AND a matching catalog entry exists.
--    If the catalog is empty (no user-created types yet), this is a safe no-op.
UPDATE public.working_conditions wc
SET condition_type_id = ctd.id
FROM public.condition_type_definitions ctd, public.employments emp
WHERE emp.id = wc.employment_id
  AND ctd.workspace_id = emp.workspace_id
  AND UPPER(wc."type") = ctd.code
  AND wc.condition_type_id IS NULL;

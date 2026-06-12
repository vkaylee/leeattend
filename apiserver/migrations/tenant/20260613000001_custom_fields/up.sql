-- 1. Custom Field Definitions (metadata)
CREATE TABLE public.custom_field_definitions (
    id uuid DEFAULT gen_random_uuid() PRIMARY KEY NOT NULL,
    workspace_id uuid NOT NULL,
    entity_type character varying(50) NOT NULL,           -- 'employee', 'employment', 'department'
    field_key character varying(100) NOT NULL,            -- machine-readable key (e.g., 'tshirt_size')
    field_label text NOT NULL,                  -- display label (i18n key or plain text)
    field_type character varying(20) NOT NULL,            -- 'text', 'number', 'date', 'boolean', 'select', 'multiselect', 'url', 'email', 'phone'
    options jsonb DEFAULT '[]'::jsonb,          -- for select/multiselect: [{"value":"S","label":"Small"}, ...]
    default_value text,                         -- optional default value
    is_required boolean DEFAULT false NOT NULL,
    is_visible_to_employee boolean DEFAULT false NOT NULL,  -- ESS portal visibility
    is_filterable boolean DEFAULT false NOT NULL,           -- enable search/filter in list APIs
    is_exportable boolean DEFAULT true NOT NULL,            -- include in report exports
    display_order integer DEFAULT 0 NOT NULL,
    section_name character varying(100),                 -- UI grouping (e.g., "Personal Info", "Custom")
    validation_rules jsonb,                    -- optional: {"min": 0, "max": 100, "pattern": "..."}
    deleted_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT uq_workspace_entity_key UNIQUE(workspace_id, entity_type, field_key)
);

CREATE INDEX idx_cfd_workspace_entity ON public.custom_field_definitions(workspace_id, entity_type)
    WHERE deleted_at IS NULL;
CREATE INDEX idx_cfd_filterable ON public.custom_field_definitions(workspace_id, entity_type, is_filterable)
    WHERE deleted_at IS NULL AND is_filterable = true;

-- 2. Custom Field Values (data)
CREATE TABLE public.custom_field_values (
    id uuid DEFAULT gen_random_uuid() PRIMARY KEY NOT NULL,
    definition_id uuid NOT NULL REFERENCES public.custom_field_definitions(id) ON DELETE CASCADE,
    entity_id uuid NOT NULL,                   -- employee_id, employment_id, department_id
    value_text text,
    value_number numeric,
    value_date date,
    value_bool boolean,
    value_json jsonb,                          -- for multiselect and complex types
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT uq_definition_entity UNIQUE(definition_id, entity_id)           -- one value per field per entity
);

CREATE INDEX idx_cfv_entity ON public.custom_field_values(entity_id);
CREATE INDEX idx_cfv_definition ON public.custom_field_values(definition_id);
CREATE INDEX idx_cfv_text_search ON public.custom_field_values(definition_id, value_text)
    WHERE value_text IS NOT NULL;
CREATE INDEX idx_cfv_number_search ON public.custom_field_values(definition_id, value_number)
    WHERE value_number IS NOT NULL;

-- Generic typed EAV table for workspace-level configuration
-- Supports temporal configs (effective_from/to) and scope hierarchy
-- (workspace > department > employee)
CREATE TABLE workspace_config_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,

    -- Scope hierarchy: 'workspace' | 'department' | 'employee'
    scope_type VARCHAR(20) NOT NULL DEFAULT 'workspace'
        CHECK (scope_type IN ('workspace', 'department', 'employee')),
    scope_id UUID,  -- NULL for workspace-level; department_id or employee_id otherwise

    -- Attribute key (dot-separated namespace, e.g. 'overtime.max_hours_per_month')
    config_key VARCHAR(100) NOT NULL,

    -- Typed values — exactly one must be non-null per row
    value_int INT,
    value_decimal NUMERIC(12,4),
    value_text TEXT,
    value_bool BOOLEAN,
    value_json JSONB,

    -- Temporal validity
    effective_from DATE NOT NULL,
    effective_to DATE,  -- NULL = still active

    -- Audit
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Only one active config per (workspace, scope, key, effective_from)
    UNIQUE(workspace_id, scope_type, scope_id, config_key, effective_from),

    -- Exactly one value column must be set
    CHECK (
        (value_int IS NOT NULL)::int +
        (value_decimal IS NOT NULL)::int +
        (value_text IS NOT NULL)::int +
        (value_bool IS NOT NULL)::int +
        (value_json IS NOT NULL)::int = 1
    )
);

-- Fast key lookups with temporal range
CREATE INDEX idx_wce_lookup ON workspace_config_entries(workspace_id, config_key, effective_from);

-- Active configs (no end date)
CREATE INDEX idx_wce_active ON workspace_config_entries(workspace_id, config_key)
    WHERE effective_to IS NULL;

-- Scope-level queries
CREATE INDEX idx_wce_scope ON workspace_config_entries(workspace_id, scope_type, scope_id);

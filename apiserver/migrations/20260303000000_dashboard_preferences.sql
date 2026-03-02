-- Dashboard layout preferences (per user, per workspace)
CREATE TABLE IF NOT EXISTS dashboard_preferences (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,

    -- Active view tab: "overview" | "attendance" | "trends" | "custom"
    active_view     TEXT NOT NULL DEFAULT 'overview',

    -- Which widgets are visible (JSON object: { "widget_id": true/false })
    visible_widgets JSONB NOT NULL DEFAULT '{}',

    -- Widget display order (JSON array: ["widget_id_1", "widget_id_2", ...])
    widget_order    JSONB NOT NULL DEFAULT '[]',

    -- Which sections are collapsed (JSON object: { "section_id": true/false })
    collapsed_sections JSONB NOT NULL DEFAULT '{}',

    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (user_id, workspace_id)
);

-- Index for fast lookup by user + workspace
CREATE INDEX IF NOT EXISTS idx_dashboard_prefs_user_ws ON dashboard_preferences (user_id, workspace_id);

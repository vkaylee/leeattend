-- Runtime feature flags per workspace
CREATE TABLE IF NOT EXISTS feature_flags (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID       NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    flag_key    VARCHAR(128) NOT NULL,
    enabled     BOOLEAN      NOT NULL DEFAULT FALSE,
    description TEXT,
    metadata    JSONB        NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT now(),

    UNIQUE (workspace_id, flag_key)
);

CREATE INDEX idx_feature_flags_workspace ON feature_flags (workspace_id);
CREATE INDEX idx_feature_flags_lookup    ON feature_flags (workspace_id, flag_key);

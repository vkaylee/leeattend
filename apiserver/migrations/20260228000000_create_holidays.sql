-- Holiday calendar table for workspace-specific holiday management
CREATE TABLE holidays (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    date DATE NOT NULL,
    holiday_type VARCHAR(50) NOT NULL DEFAULT 'national',
    is_paid BOOLEAN NOT NULL DEFAULT TRUE,
    ot_multiplier DECIMAL(3,1),
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(workspace_id, date)
);

CREATE INDEX idx_holidays_lookup ON holidays(workspace_id, date);

-- Workspace-level holiday configuration
ALTER TABLE workspaces
ADD COLUMN holiday_overnight_strategy VARCHAR(30) NOT NULL DEFAULT 'split_at_midnight',
ADD COLUMN default_holiday_ot_multiplier DECIMAL(3,1) NOT NULL DEFAULT 3.0;

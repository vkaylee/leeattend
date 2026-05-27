CREATE TABLE work_locations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    address TEXT,
    timezone VARCHAR(50) NOT NULL DEFAULT 'UTC',
    is_headquarters BOOLEAN NOT NULL DEFAULT false,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX idx_work_locations_hq 
    ON work_locations(workspace_id) WHERE is_headquarters = true;

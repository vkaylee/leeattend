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

-- Add column allowing nulls first
ALTER TABLE employees ADD COLUMN primary_location_id UUID REFERENCES work_locations(id);

-- Assign existing employees to their workspace's HQ
UPDATE employees e
SET primary_location_id = wl.id
FROM work_locations wl
WHERE e.workspace_id = wl.workspace_id AND wl.is_headquarters = true;

-- Enforce NOT NULL
ALTER TABLE employees ALTER COLUMN primary_location_id SET NOT NULL;

-- Add column allowing nulls first
ALTER TABLE employees ADD COLUMN primary_location_id UUID REFERENCES work_locations(id);

-- Ensure there is a headquarters work location for every workspace that has employees
INSERT INTO work_locations (id, workspace_id, name, is_headquarters, timezone, status)
SELECT md5('hq_location_' || workspace_id::text)::uuid, workspace_id, 'Headquarters', true, 'UTC', 'active'
FROM (SELECT DISTINCT workspace_id FROM employees) w
ON CONFLICT (workspace_id) WHERE is_headquarters = true DO NOTHING;

-- Assign existing employees to their workspace's HQ
UPDATE employees e
SET primary_location_id = wl.id
FROM work_locations wl
WHERE e.workspace_id = wl.workspace_id AND wl.is_headquarters = true;

-- Enforce NOT NULL
ALTER TABLE employees ALTER COLUMN primary_location_id SET NOT NULL;

CREATE EXTENSION IF NOT EXISTS btree_gist;

CREATE TABLE attendance_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Ensure only one default policy per workspace
CREATE UNIQUE INDEX attendance_policies_workspace_default_idx 
ON attendance_policies (workspace_id) 
WHERE is_default = true;

CREATE TABLE attendance_policy_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_id UUID NOT NULL REFERENCES attendance_policies(id) ON DELETE CASCADE,
    rules JSONB NOT NULL DEFAULT '{}'::jsonb,
    valid_from TIMESTAMPTZ NOT NULL,
    valid_to TIMESTAMPTZ,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Exclusion constraint to prevent overlapping versions for the same policy
ALTER TABLE attendance_policy_versions
ADD CONSTRAINT no_overlapping_policy_versions
EXCLUDE USING GIST (
    policy_id WITH =,
    tstzrange(valid_from, COALESCE(valid_to, 'infinity'::timestamptz)) WITH &&
);

CREATE TABLE attendance_policy_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_id UUID NOT NULL REFERENCES attendance_policies(id) ON DELETE CASCADE,
    target_type VARCHAR(50) NOT NULL, -- 'GROUP', 'USER'
    target_id UUID NOT NULL,
    valid_from TIMESTAMPTZ NOT NULL,
    valid_to TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Exclusion constraint to prevent overlapping assignments for the same target
ALTER TABLE attendance_policy_assignments
ADD CONSTRAINT no_overlapping_policy_assignments
EXCLUDE USING GIST (
    target_id WITH =,
    target_type WITH =,
    tstzrange(valid_from, COALESCE(valid_to, 'infinity'::timestamptz)) WITH &&
);

-- Migrate old data from workspaces table to the new tables as default policy
DO $$
DECLARE
    ws_record RECORD;
    new_policy_id UUID;
    rules_json JSONB;
BEGIN
    FOR ws_record IN SELECT id, work_unit_minutes, rounding_policy FROM workspaces LOOP
        rules_json := json_build_object(
            'work_unit_minutes', ws_record.work_unit_minutes,
            'rounding_policy', ws_record.rounding_policy
        );
        
        new_policy_id := gen_random_uuid();
        INSERT INTO attendance_policies (id, workspace_id, name, is_default)
        VALUES (new_policy_id, ws_record.id, 'Default Attendance Policy', true);
        
        INSERT INTO attendance_policy_versions (policy_id, rules, valid_from, valid_to)
        VALUES (new_policy_id, rules_json, '2000-01-01 00:00:00+00', NULL);
    END LOOP;
END $$;

-- Drop old columns from workspaces
ALTER TABLE workspaces DROP COLUMN work_unit_minutes;
ALTER TABLE workspaces DROP COLUMN rounding_policy;

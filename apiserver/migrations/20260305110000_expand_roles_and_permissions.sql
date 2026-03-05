-- Up Migration: Expand role system with new role types and extra permissions table

-- Add new role values to the member_role_type enum
-- NOTE: ALTER TYPE ... ADD VALUE cannot run inside a transaction in Postgres.
--       sqlx runs each migration in a transaction, so we use DO blocks with
--       dynamic SQL to add values only if they don't already exist.
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_enum WHERE enumlabel = 'hr_manager' AND enumtypid = 'member_role_type'::regtype) THEN
        EXECUTE 'ALTER TYPE member_role_type ADD VALUE ''hr_manager''';
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_enum WHERE enumlabel = 'shift_manager' AND enumtypid = 'member_role_type'::regtype) THEN
        EXECUTE 'ALTER TYPE member_role_type ADD VALUE ''shift_manager''';
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_enum WHERE enumlabel = 'team_lead' AND enumtypid = 'member_role_type'::regtype) THEN
        EXECUTE 'ALTER TYPE member_role_type ADD VALUE ''team_lead''';
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_enum WHERE enumlabel = 'employee' AND enumtypid = 'member_role_type'::regtype) THEN
        EXECUTE 'ALTER TYPE member_role_type ADD VALUE ''employee''';
    END IF;
END $$;

-- Extra permissions table for fine-grained per-user permission overrides
CREATE TABLE IF NOT EXISTS workspace_member_extra_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permission TEXT NOT NULL,
    assigned_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    assigned_by UUID REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_extra_perms_lookup
    ON workspace_member_extra_permissions(workspace_id, user_id, expires_at);

-- Add scope_type to workspace_roles to replace hardcoded role name checks
-- in application code (previously: name IN ('owner', 'admin', 'hr_manager', 'shift_manager')).
-- This allows custom roles created by Workspace Admins to also carry global scope.

ALTER TABLE workspace_roles
    ADD COLUMN scope_type VARCHAR(16) NOT NULL DEFAULT 'scoped'
        CHECK (scope_type IN ('global', 'scoped'));

-- Backfill: system roles that have workspace-wide (global) authority
UPDATE workspace_roles
   SET scope_type = 'global'
 WHERE is_system = TRUE
   AND name IN ('owner', 'admin', 'hr_manager', 'shift_manager');

-- All other system roles (team_lead, employee) and new custom roles stay 'scoped' by default.

CREATE INDEX idx_workspace_roles_scope_type ON workspace_roles(scope_type);

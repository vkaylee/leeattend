-- Down Migration: Remove extra permissions table
-- NOTE: Postgres does not support DROP VALUE from enums,
-- so the new enum values remain (they are harmless).

DROP TABLE IF EXISTS workspace_member_extra_permissions;

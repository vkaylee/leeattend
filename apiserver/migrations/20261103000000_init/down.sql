-- Single-tenant baseline rollback.
-- Use only after backup/restore protection; this removes all application objects.
DROP SCHEMA IF EXISTS public CASCADE;
CREATE SCHEMA public;

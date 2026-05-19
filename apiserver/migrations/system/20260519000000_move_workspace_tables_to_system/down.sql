-- Drop workspace-related tables (moved from tenant to system)
DROP TABLE IF EXISTS public.workspace_member_extra_permissions CASCADE;
DROP TABLE IF EXISTS public.member_extra_roles CASCADE;
DROP TABLE IF EXISTS public.workspace_role_permissions CASCADE;
DROP TABLE IF EXISTS public.workspace_roles CASCADE;
DROP TABLE IF EXISTS public.workspace_members CASCADE;

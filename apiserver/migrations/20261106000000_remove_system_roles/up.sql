BEGIN;

DO $$
DECLARE
    organization_count bigint;
    admin_role_count bigint;
    admin_member_count bigint;
BEGIN
	IF to_regclass('public.system_roles') IS NULL OR to_regclass('public.user_system_roles') IS NULL OR to_regclass('public.system_db_clusters') IS NULL THEN
		RAISE EXCEPTION 'legacy platform tables are missing; migration state is inconsistent';
	END IF;

    SELECT count(*) INTO organization_count FROM public.workspaces;
    IF organization_count <> 1 THEN
        RAISE EXCEPTION 'expected exactly one standalone organization, found %', organization_count;
    END IF;

    SELECT count(*) INTO admin_role_count
    FROM public.workspace_roles
    WHERE workspace_id IS NULL AND name = 'admin' AND is_system = true;
    IF admin_role_count <> 1 THEN
        RAISE EXCEPTION 'expected exactly one global workspace admin role, found %', admin_role_count;
    END IF;

    SELECT count(*) INTO admin_member_count
    FROM public.workspace_members wm
    JOIN public.workspace_roles wr ON wr.id = wm.role_id
    WHERE wr.workspace_id IS NULL AND wr.name = 'admin' AND wr.is_system = true;
    IF admin_member_count < 1 THEN
        RAISE EXCEPTION 'no active standalone organization administrator assignment found';
    END IF;
END $$;

DROP TABLE public.user_system_roles;
DROP TABLE public.system_roles;

DROP TABLE public.system_db_clusters;

COMMIT;

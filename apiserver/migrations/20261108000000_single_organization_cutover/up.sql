BEGIN;

DO $$
DECLARE
    r record;
    c record;
    v_org_count bigint;
    v_key_columns text;
    v_include_columns text;
    v_predicate text;
    v_new_name text;
    v_constraint_name text;
    v_constraint_def text;
    v_sql text;
    v_roles_id oid;
BEGIN
    -- Freeze every organization-scoped table before preflight or DDL. The cutover
    -- is designed for a maintenance window; ACCESS EXCLUSIVE also prevents a
    -- writer racing singleton/collision checks.
    FOR r IN
        SELECT DISTINCT n.nspname, cls.relname
        FROM pg_class cls
        JOIN pg_namespace n ON n.oid = cls.relnamespace
        JOIN pg_attribute a ON a.attrelid = cls.oid AND a.attname = 'workspace_id' AND NOT a.attisdropped
        WHERE n.nspname = 'public' AND cls.relkind IN ('r', 'p') AND NOT cls.relispartition
        ORDER BY n.nspname, cls.relname
    LOOP
        EXECUTE format('LOCK TABLE %I.%I IN ACCESS EXCLUSIVE MODE', r.nspname, r.relname);
    END LOOP;

    SELECT count(*) INTO v_org_count FROM public.workspaces;
    IF v_org_count > 1 THEN
        RAISE EXCEPTION 'single-organization cutover refused: public.workspaces contains % rows; reconcile to exactly one organization before retrying', v_org_count;
    END IF;
    IF EXISTS (
        SELECT 1 FROM public.workspace_member_extra_permissions old
        JOIN public.workspace_member_extra_permissions current
          ON current.user_id = old.user_id
         AND current.permission = CASE old.permission
              WHEN 'workspace:settings:read' THEN 'organization:settings:read'
              WHEN 'workspace:settings:write' THEN 'organization:settings:write'
         END
        WHERE old.permission IN ('workspace:settings:read', 'workspace:settings:write')
    ) THEN
        RAISE EXCEPTION 'single-organization cutover refused: a user has both legacy and organization settings permissions; reconcile duplicate grants before retrying';
    END IF;

    IF EXISTS (SELECT 1 FROM public.policy_templates GROUP BY name, version HAVING count(*) > 1) THEN
        RAISE EXCEPTION 'single-organization cutover refused: policy template name/version pairs collide across former scopes; reconcile conflicting templates before retrying';
    END IF;
    IF EXISTS (SELECT 1 FROM public.salary_base_groups GROUP BY slug HAVING count(*) > 1) THEN
        RAISE EXCEPTION 'single-organization cutover refused: salary base group slugs collide across former scopes; rename conflicting groups before retrying';
    END IF;
    IF (SELECT count(*) FROM public.storage_configs) > 1 THEN
        RAISE EXCEPTION 'single-organization cutover refused: storage_configs contains multiple rows; choose one storage configuration before retrying';
    END IF;
    IF (SELECT count(*) FROM public.workspace_compliance_rules) > 1 THEN
        RAISE EXCEPTION 'single-organization cutover refused: workspace_compliance_rules contains multiple rows; reconcile compliance settings before retrying';
    END IF;

    IF EXISTS (
        SELECT 1 FROM public.workspace_roles
        GROUP BY name HAVING count(*) > 1
    ) THEN
        RAISE EXCEPTION 'single-organization cutover refused: global and organization roles share a name; rename conflicting roles before retrying';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM public.workspace_role_permissions old
        JOIN public.workspace_role_permissions current
          ON current.role_id = old.role_id
         AND current.permission_name = CASE old.permission_name
              WHEN 'workspace:settings:read' THEN 'organization:settings:read'
              WHEN 'workspace:settings:write' THEN 'organization:settings:write'
         END
        WHERE old.permission_name IN ('workspace:settings:read', 'workspace:settings:write')
    ) THEN
        RAISE EXCEPTION 'single-organization cutover refused: a role has both legacy and organization settings permissions; reconcile duplicate grants before retrying';
    END IF;

    IF to_regclass('public.organization_settings') IS NOT NULL
       OR to_regclass('public.roles') IS NOT NULL
       OR to_regclass('public.user_roles') IS NOT NULL
       OR to_regclass('public.role_permissions') IS NOT NULL
       OR to_regclass('public.files') IS NOT NULL
       OR to_regclass('public.compliance_rules') IS NOT NULL
       OR to_regclass('public.config_entries') IS NOT NULL
       OR to_regclass('public.user_extra_roles') IS NOT NULL
       OR to_regclass('public.user_extra_permissions') IS NOT NULL THEN
        RAISE EXCEPTION 'single-organization cutover refused: one or more final schema table names already exist; inspect partial migration state';
    END IF;

    -- Remove partial unique indexes whose predicates distinguish global and
    -- organization rows. Final uniqueness applies to the merged single scope.
    DROP INDEX public.idx_system_roles_unique_name;
    DROP INDEX public.idx_workspace_roles_unique_name;
    DROP INDEX public.idx_policy_templates_system_unique;
    DROP INDEX public.idx_policy_templates_workspace_unique;
    DROP INDEX public.idx_salary_base_groups_global_slug;
    DROP INDEX public.idx_salary_base_groups_workspace_slug;

    -- Rename business tables before rebuilding constraints and indexes.
    ALTER TABLE public.workspaces RENAME TO organization_settings;
    ALTER TABLE public.workspace_members RENAME TO user_roles;
    ALTER TABLE public.workspace_roles RENAME TO roles;
    ALTER TABLE public.workspace_role_permissions RENAME TO role_permissions;
    ALTER TABLE public.member_extra_roles RENAME TO user_extra_roles;
    ALTER TABLE public.workspace_member_extra_permissions RENAME TO user_extra_permissions;
    ALTER TABLE public.workspace_files RENAME TO files;
    ALTER TABLE public.workspace_compliance_rules RENAME TO compliance_rules;
    ALTER TABLE public.workspace_config_entries RENAME TO config_entries;

    -- Normalize persisted authorization and configuration values without
    -- deleting, merging, or recreating any grant rows.
    UPDATE public.role_permissions
       SET permission_name = CASE permission_name
           WHEN 'workspace:settings:read' THEN 'organization:settings:read'
           WHEN 'workspace:settings:write' THEN 'organization:settings:write'
           ELSE permission_name
       END
     WHERE permission_name IN ('workspace:settings:read', 'workspace:settings:write');
    UPDATE public.user_extra_permissions
       SET permission = CASE permission
           WHEN 'workspace:settings:read' THEN 'organization:settings:read'
           WHEN 'workspace:settings:write' THEN 'organization:settings:write'
           ELSE permission
       END
     WHERE permission IN ('workspace:settings:read', 'workspace:settings:write');
    ALTER TABLE public.config_entries DROP CONSTRAINT workspace_config_entries_scope_type_check;
    UPDATE public.config_entries SET scope_type = 'organization', scope_id = NULL WHERE scope_type = 'workspace';
    ALTER TABLE public.config_entries ALTER COLUMN scope_type SET DEFAULT 'organization';
    ALTER TABLE public.config_entries ADD CONSTRAINT config_entries_scope_type_check
        CHECK (scope_type IN ('organization', 'employer', 'department', 'employee', 'employment'));
    ALTER TABLE public.leave_policy_assignments DROP CONSTRAINT leave_policy_assignments_assignee_type_check;
    ALTER TABLE public.overtime_policy_assignments DROP CONSTRAINT overtime_policy_assignments_assignee_type_check;
    ALTER TABLE public.payroll_incentive_assignments DROP CONSTRAINT payroll_incentive_assignments_assignee_type_check;
    ALTER TABLE public.payroll_rule_assignments DROP CONSTRAINT payroll_rule_assignments_assignee_type_check;
    -- Replace assignment trigger functions before writing the new discriminator.
    CREATE OR REPLACE FUNCTION public.check_assignment_target_exists() RETURNS trigger
        LANGUAGE plpgsql AS $function$
    BEGIN
        IF NEW.assignee_type = 'organization' THEN
            RETURN NEW;
        END IF;
        IF NEW.assignee_id IS NULL THEN
            RAISE EXCEPTION 'assignee_id cannot be NULL when assignee_type is %', NEW.assignee_type;
        END IF;
        IF NEW.assignee_type = 'department' THEN
            IF NOT EXISTS (SELECT 1 FROM public.departments WHERE id = NEW.assignee_id) THEN
                RAISE EXCEPTION 'Invalid assignee_id: department % does not exist', NEW.assignee_id;
            END IF;
        ELSIF NEW.assignee_type = 'employee' THEN
            IF NOT EXISTS (SELECT 1 FROM public.employees WHERE id = NEW.assignee_id) THEN
                RAISE EXCEPTION 'Invalid assignee_id: employee % does not exist', NEW.assignee_id;
            END IF;
        ELSIF NEW.assignee_type = 'user' THEN
            RETURN NEW;
        ELSE
            RAISE EXCEPTION 'Unknown assignee_type: %', NEW.assignee_type;
        END IF;
        RETURN NEW;
    END;
    $function$;

    CREATE OR REPLACE FUNCTION public.fn_validate_assignee_fk() RETURNS trigger
        LANGUAGE plpgsql AS $function$
    BEGIN
        IF NEW.assignee_type = 'organization' THEN
            RETURN NEW;
        END IF;
        IF NEW.assignee_id IS NULL THEN
            RAISE EXCEPTION 'assignee_id cannot be NULL when assignee_type is %', NEW.assignee_type;
        END IF;
        IF NEW.assignee_type = 'department' THEN
            IF NOT EXISTS (SELECT 1 FROM public.departments WHERE id = NEW.assignee_id) THEN
                RAISE EXCEPTION 'Invalid assignee_id: department % does not exist', NEW.assignee_id;
            END IF;
        ELSIF NEW.assignee_type = 'employee' THEN
            IF NOT EXISTS (SELECT 1 FROM public.employees WHERE id = NEW.assignee_id) THEN
                RAISE EXCEPTION 'Invalid assignee_id: employee % does not exist', NEW.assignee_id;
            END IF;
        ELSE
            RAISE EXCEPTION 'Unknown assignee_type: %', NEW.assignee_type;
        END IF;
        RETURN NEW;
    END;
    $function$;

    -- Organization-wide assignment rows no longer carry a target row UUID.
    UPDATE public.leave_policy_assignments
       SET assignee_type = 'organization', assignee_id = NULL
     WHERE assignee_type = 'workspace';
    UPDATE public.overtime_policy_assignments
       SET assignee_type = 'organization', assignee_id = NULL
     WHERE assignee_type = 'workspace';
    UPDATE public.payroll_incentive_assignments
       SET assignee_type = 'organization', assignee_id = NULL
     WHERE assignee_type = 'workspace';
    UPDATE public.payroll_rule_assignments
       SET assignee_type = 'organization', assignee_id = NULL
     WHERE assignee_type = 'workspace';

    -- Preserve object keys and ownership IDs; only rename old organization labels.
    UPDATE public.files SET owner_type = 'organization' WHERE owner_type = 'workspace';
    UPDATE public.files SET category = 'organization_logo' WHERE category = 'workspace_logo';

    CREATE OR REPLACE FUNCTION public.enqueue_roster_cache_invalidation() RETURNS trigger
        LANGUAGE plpgsql SET search_path TO 'public' AS $function$
    BEGIN
        INSERT INTO public.outbox_jobs (
            id, job_type, payload, status, error_message, created_at, updated_at
        ) VALUES (
            gen_random_uuid(), 'RosterCacheInvalidation', '{}'::jsonb, 'pending', NULL,
            CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
        );
        IF TG_OP = 'DELETE' THEN
            RETURN OLD;
        END IF;
        RETURN NEW;
    END;
    $function$;

    -- Drop FKs that bind rows to an organization key; preserve all business FKs.
    FOR c IN
        SELECT con.conrelid::regclass AS table_name, con.conname
        FROM pg_constraint con
        WHERE con.contype = 'f'
          AND (con.confrelid = 'public.organization_settings'::regclass OR EXISTS (
              SELECT 1 FROM unnest(con.conkey) k(attnum)
              JOIN pg_attribute a ON a.attrelid = con.conrelid AND a.attnum = k.attnum
              WHERE a.attname = 'workspace_id'
          ))
    LOOP
        EXECUTE format('ALTER TABLE %s DROP CONSTRAINT %I', c.table_name, c.conname);
    END LOOP;

    -- Replace every unique/primary/exclusion constraint that included the
    -- organization key with the same remaining key. A singleton makes this
    -- equivalent; any unexpected collision aborts the transaction.
    FOR c IN
        SELECT con.oid, con.conrelid::regclass AS table_name, con.conname,
               con.contype, con.conkey, pg_get_constraintdef(con.oid) AS definition,
               cls.relname AS table_relname
        FROM pg_constraint con
        JOIN pg_class cls ON cls.oid = con.conrelid
        JOIN pg_namespace ns ON ns.oid = cls.relnamespace
        WHERE ns.nspname = 'public' AND con.contype IN ('p', 'u', 'x')
          AND EXISTS (
              SELECT 1 FROM unnest(con.conkey) k(attnum)
              JOIN pg_attribute a ON a.attrelid = con.conrelid AND a.attnum = k.attnum
              WHERE a.attname = 'workspace_id'
          )
        ORDER BY con.conrelid, con.conname
    LOOP
        v_constraint_name := regexp_replace(c.conname, 'workspace(_id)?', 'organization', 'gi');
        EXECUTE format('ALTER TABLE %s DROP CONSTRAINT %I', c.table_name, c.conname);

        IF c.contype = 'x' THEN
            v_constraint_def := regexp_replace(c.definition, 'workspace_id WITH =,\s*', '', 'gi');
            IF v_constraint_def = c.definition THEN
                RAISE EXCEPTION 'cannot safely rebuild exclusion constraint % on %: organization key expression not recognized', c.conname, c.table_name;
            END IF;
            EXECUTE format('ALTER TABLE %s ADD CONSTRAINT %I %s', c.table_name, v_constraint_name, v_constraint_def);
        ELSE
            SELECT string_agg(format('%I', a.attname), ', ' ORDER BY k.ordinality)
              INTO v_key_columns
              FROM unnest(c.conkey) WITH ORDINALITY k(attnum, ordinality)
              JOIN pg_attribute a ON a.attrelid = c.table_name::oid AND a.attnum = k.attnum
             WHERE a.attname <> 'workspace_id';
            IF v_key_columns IS NULL THEN
                IF c.contype = 'p' THEN
                    RAISE EXCEPTION 'cannot rebuild primary key % on % after removing organization key', c.conname, c.table_name;
                END IF;
                v_new_name := regexp_replace(c.table_relname || '_organization_singleton_key', 'workspace(_id)?', 'organization', 'gi');
                EXECUTE format('CREATE UNIQUE INDEX %I ON %s ((true))', v_new_name, c.table_name);
            ELSIF c.contype = 'p' THEN
                EXECUTE format('ALTER TABLE %s ADD CONSTRAINT %I PRIMARY KEY (%s)', c.table_name, v_constraint_name, v_key_columns);
            ELSE
                EXECUTE format('ALTER TABLE %s ADD CONSTRAINT %I UNIQUE (%s)', c.table_name, v_constraint_name, v_key_columns);
            END IF;
        END IF;
    END LOOP;

    -- Rebuild ordinary indexes with their organization key removed. Attached
    -- partition indexes are recreated by PostgreSQL when their parent index is
    -- created; processing them independently would attempt to rebuild twice.
    FOR r IN
        SELECT i.indexrelid, i.indrelid, idx.relname AS index_name,
               tbl.relname AS table_name, am.amname AS access_method,
               i.indnkeyatts, i.indnatts, i.indisunique,
               pg_get_expr(i.indpred, i.indrelid) AS predicate,
               EXISTS (SELECT 1 FROM pg_inherits inh WHERE inh.inhrelid = i.indexrelid) AS is_attached
        FROM pg_index i
        JOIN pg_class idx ON idx.oid = i.indexrelid
        JOIN pg_class tbl ON tbl.oid = i.indrelid
        JOIN pg_namespace ns ON ns.oid = tbl.relnamespace
        JOIN pg_am am ON am.oid = idx.relam
        WHERE ns.nspname = 'public'
          AND NOT EXISTS (SELECT 1 FROM pg_inherits inh WHERE inh.inhrelid = i.indexrelid)
          AND EXISTS (
              SELECT 1 FROM unnest(i.indkey) WITH ORDINALITY k(attnum, ordinality)
              JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = k.attnum
              WHERE a.attname = 'workspace_id' AND k.ordinality <= i.indnkeyatts
          )
        ORDER BY tbl.relname, idx.relname
    LOOP
        IF r.predicate IS NOT NULL AND r.predicate ~* '\mworkspace_id\M' THEN
            RAISE EXCEPTION 'cannot safely rebuild partial index % on %: predicate still distinguishes organization rows', r.index_name, r.table_name;
        END IF;
        SELECT string_agg(pg_get_indexdef(r.indexrelid, n, true), ', ' ORDER BY n)
          INTO v_key_columns
          FROM generate_series(1, r.indnkeyatts) n
         WHERE pg_get_indexdef(r.indexrelid, n, true) <> 'workspace_id';
        SELECT string_agg(pg_get_indexdef(r.indexrelid, n, true), ', ' ORDER BY n)
          INTO v_include_columns
          FROM generate_series(r.indnkeyatts + 1, r.indnatts) n
         WHERE pg_get_indexdef(r.indexrelid, n, true) <> 'workspace_id';
        v_new_name := regexp_replace(r.index_name, 'workspace(_id)?', 'organization', 'gi');
        EXECUTE format('DROP INDEX public.%I', r.index_name);
        IF v_key_columns IS NULL THEN
            IF r.indisunique THEN
                EXECUTE format('CREATE UNIQUE INDEX %I ON public.%I ((true))%s', v_new_name, r.table_name,
                    CASE WHEN r.predicate IS NULL THEN '' ELSE format(' WHERE %s', r.predicate) END);
            END IF;
        ELSE
            v_sql := format('CREATE %s INDEX %I ON public.%I USING %I (%s)',
                CASE WHEN r.indisunique THEN 'UNIQUE' ELSE '' END,
                v_new_name, r.table_name, r.access_method, v_key_columns);
            IF v_include_columns IS NOT NULL THEN
                v_sql := v_sql || format(' INCLUDE (%s)', v_include_columns);
            END IF;
            IF r.predicate IS NOT NULL THEN
                v_sql := v_sql || format(' WHERE %s', r.predicate);
            END IF;
            EXECUTE v_sql;
        END IF;
    END LOOP;

    -- Remove workspace_id from every base table; parent operations propagate to
    -- attached partitions. Re-adding these columns would invent tenant identity.
    FOR r IN
        SELECT DISTINCT n.nspname, cls.relname
        FROM pg_class cls
        JOIN pg_namespace n ON n.oid = cls.relnamespace
        JOIN pg_attribute a ON a.attrelid = cls.oid AND a.attname = 'workspace_id' AND NOT a.attisdropped
        WHERE n.nspname = 'public' AND cls.relkind IN ('r', 'p') AND NOT cls.relispartition
        ORDER BY n.nspname, cls.relname
    LOOP
        EXECUTE format('ALTER TABLE %I.%I DROP COLUMN workspace_id', r.nspname, r.relname);
    END LOOP;

    -- Assignment type constraints were dropped before normalization so legacy
    -- BEFORE UPDATE triggers and CHECK constraints never see invalid values.
    ALTER TABLE public.leave_policy_assignments ADD CONSTRAINT leave_policy_assignments_assignee_type_check
        CHECK (assignee_type IN ('organization', 'department', 'employee', 'user'));
    ALTER TABLE public.overtime_policy_assignments ADD CONSTRAINT overtime_policy_assignments_assignee_type_check
        CHECK (assignee_type IN ('organization', 'department', 'employee', 'user'));
    ALTER TABLE public.payroll_incentive_assignments ADD CONSTRAINT payroll_incentive_assignments_assignee_type_check
        CHECK (assignee_type IN ('organization', 'department', 'employee', 'user'));
    ALTER TABLE public.payroll_rule_assignments ADD CONSTRAINT payroll_rule_assignments_assignee_type_check
        CHECK (assignee_type IN ('organization', 'department', 'employee'));

    -- Merge former global/local uniqueness only after the duplicate-name
    -- preflight above. Unconditional constraints retain all role/template rows.
    CREATE UNIQUE INDEX roles_name_key ON public.roles (name);
    CREATE UNIQUE INDEX policy_templates_name_version_key ON public.policy_templates (name, version);
    CREATE UNIQUE INDEX salary_base_groups_slug_key ON public.salary_base_groups (slug);

    -- At most one settings/storage/compliance row; zero settings rows remains
    -- valid until first-run bootstrap inserts the configured organization.
    -- The former unique(workspace_id) constraints on storage_configs and
    -- compliance_rules were rebuilt as singleton partial keys above.
    CREATE UNIQUE INDEX organization_settings_singleton_key ON public.organization_settings ((true));

    -- Keep object names free of obsolete tenant terminology where table renames
    -- alone do not rename standalone indexes or triggers.
    IF to_regclass('public.idx_workspace_members_user_id') IS NOT NULL THEN
        ALTER INDEX public.idx_workspace_members_user_id RENAME TO idx_user_roles_user_id;
    END IF;
    IF to_regclass('public.idx_workspace_roles_scope_type') IS NOT NULL THEN
        ALTER INDEX public.idx_workspace_roles_scope_type RENAME TO idx_roles_scope_type;
    END IF;
    IF to_regclass('public.idx_extra_perms_lookup') IS NOT NULL THEN
        ALTER INDEX public.idx_extra_perms_lookup RENAME TO idx_user_extra_permissions_lookup;
    END IF;
    IF to_regclass('public.idx_member_extra_roles_lookup') IS NOT NULL THEN
        ALTER INDEX public.idx_member_extra_roles_lookup RENAME TO idx_user_extra_roles_lookup;
    END IF;
    IF to_regclass('public.idx_wce_active') IS NOT NULL THEN
        ALTER INDEX public.idx_wce_active RENAME TO idx_config_entries_active;
    END IF;
    IF to_regclass('public.idx_wce_lookup') IS NOT NULL THEN
        ALTER INDEX public.idx_wce_lookup RENAME TO idx_config_entries_lookup;
    END IF;
    IF to_regclass('public.idx_wce_scope') IS NOT NULL THEN
        ALTER INDEX public.idx_wce_scope RENAME TO idx_config_entries_scope;
    END IF;
    IF to_regclass('public.idx_workspace_compliance_rules_workspace') IS NOT NULL THEN
        DROP INDEX public.idx_workspace_compliance_rules_workspace;
    END IF;

    COMMENT ON TABLE public.compliance_rules IS 'Organization-wide compliance rule thresholds for shift conflict detection and rest-period validation.';
    COMMENT ON TABLE public.organization_settings IS 'Singleton organization settings; its UUID is internal and never selected by clients.';
END;
$$;

COMMIT;

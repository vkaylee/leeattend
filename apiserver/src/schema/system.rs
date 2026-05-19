// @generated automatically by Diesel CLI.

diesel::table! {
    audit_logs (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    audit_logs_default (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    audit_logs_y2026m01 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    audit_logs_y2026m02 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    audit_logs_y2026m03 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    audit_logs_y2026m04 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    audit_logs_y2026m05 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    audit_logs_y2026m06 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    audit_logs_y2026m07 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    audit_logs_y2026m08 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    audit_logs_y2026m09 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    audit_logs_y2026m10 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    audit_logs_y2026m11 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    audit_logs_y2026m12 (id, created_at) {
        id -> Uuid,
        workspace_id -> Uuid,
        actor_id -> Nullable<Uuid>,
        target_user_id -> Nullable<Uuid>,
        target_resource_id -> Nullable<Uuid>,
        #[max_length = 255]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    authorized_agents (id) {
        #[max_length = 50]
        id -> Varchar,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 255]
        token -> Varchar,
        is_trusted -> Nullable<Bool>,
        #[max_length = 45]
        last_ip -> Nullable<Varchar>,
        created_at -> Nullable<Timestamptz>,
        is_active -> Bool,
        workspace_id -> Uuid,
        last_seen_at -> Nullable<Timestamptz>,
        max_concurrent_workers -> Int4,
        success_cooldown_secs -> Int4,
        retention_days -> Int4,
    }
}

diesel::table! {
    member_extra_roles (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        user_id -> Uuid,
        assigned_at -> Nullable<Timestamptz>,
        expires_at -> Timestamptz,
        role_id -> Uuid,
    }
}

diesel::table! {
    sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
        token -> Text,
        expires_at -> Timestamptz,
        created_at -> Nullable<Timestamptz>,
        revoked_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    system_audit_logs (id) {
        id -> Uuid,
        actor_id -> Nullable<Uuid>,
        #[max_length = 100]
        action -> Varchar,
        details -> Nullable<Jsonb>,
        ip_address -> Nullable<Inet>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    system_roles (id) {
        id -> Uuid,
        name -> Text,
        description -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    user_system_roles (user_id, role_id) {
        user_id -> Uuid,
        role_id -> Uuid,
        assigned_by -> Nullable<Uuid>,
        assigned_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        password_hash -> Varchar,
        created_at -> Nullable<Timestamptz>,
        #[max_length = 100]
        full_name -> Nullable<Varchar>,
        avatar_url -> Nullable<Text>,
        bio -> Nullable<Text>,
        #[max_length = 10]
        preferred_language -> Nullable<Varchar>,
    }
}

diesel::table! {
    workspace_member_extra_permissions (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        user_id -> Uuid,
        permission -> Text,
        assigned_at -> Nullable<Timestamptz>,
        expires_at -> Timestamptz,
        assigned_by -> Nullable<Uuid>,
    }
}

diesel::table! {
    workspace_members (workspace_id, user_id) {
        workspace_id -> Uuid,
        user_id -> Uuid,
        role_id -> Uuid,
    }
}

diesel::table! {
    workspace_role_permissions (role_id, permission_name) {
        role_id -> Uuid,
        permission_name -> Text,
    }
}

diesel::table! {
    workspace_roles (id) {
        id -> Uuid,
        workspace_id -> Nullable<Uuid>,
        name -> Text,
        description -> Nullable<Text>,
        color -> Nullable<Text>,
        is_system -> Nullable<Bool>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        #[max_length = 16]
        scope_type -> Varchar,
    }
}

diesel::table! {
    workspaces (id) {
        id -> Uuid,
        #[max_length = 100]
        name -> Varchar,
        created_at -> Nullable<Timestamptz>,
        #[max_length = 20]
        date_format -> Varchar,
        #[max_length = 10]
        time_format -> Varchar,
        #[max_length = 50]
        timezone -> Varchar,
        theme_primary_color -> Nullable<Text>,
        #[max_length = 10]
        default_language -> Varchar,
        #[max_length = 30]
        holiday_overnight_strategy -> Varchar,
        default_holiday_ot_multiplier -> Numeric,
        logo_url -> Nullable<Text>,
        tagline -> Nullable<Text>,
        custom_db_url_encrypted -> Nullable<Text>,
    }
}

diesel::joinable!(audit_logs -> workspaces (workspace_id));
diesel::joinable!(audit_logs_default -> workspaces (workspace_id));
diesel::joinable!(audit_logs_y2026m01 -> workspaces (workspace_id));
diesel::joinable!(audit_logs_y2026m02 -> workspaces (workspace_id));
diesel::joinable!(audit_logs_y2026m03 -> workspaces (workspace_id));
diesel::joinable!(audit_logs_y2026m04 -> workspaces (workspace_id));
diesel::joinable!(audit_logs_y2026m05 -> workspaces (workspace_id));
diesel::joinable!(audit_logs_y2026m06 -> workspaces (workspace_id));
diesel::joinable!(audit_logs_y2026m07 -> workspaces (workspace_id));
diesel::joinable!(audit_logs_y2026m08 -> workspaces (workspace_id));
diesel::joinable!(audit_logs_y2026m09 -> workspaces (workspace_id));
diesel::joinable!(audit_logs_y2026m10 -> workspaces (workspace_id));
diesel::joinable!(audit_logs_y2026m11 -> workspaces (workspace_id));
diesel::joinable!(audit_logs_y2026m12 -> workspaces (workspace_id));
diesel::joinable!(member_extra_roles -> users (user_id));
diesel::joinable!(member_extra_roles -> workspace_roles (role_id));
diesel::joinable!(member_extra_roles -> workspaces (workspace_id));
diesel::joinable!(system_audit_logs -> users (actor_id));
diesel::joinable!(user_system_roles -> system_roles (role_id));
diesel::joinable!(workspace_member_extra_permissions -> workspaces (workspace_id));
diesel::joinable!(workspace_members -> users (user_id));
diesel::joinable!(workspace_members -> workspace_roles (role_id));
diesel::joinable!(workspace_members -> workspaces (workspace_id));
diesel::joinable!(workspace_role_permissions -> workspace_roles (role_id));
diesel::joinable!(workspace_roles -> workspaces (workspace_id));

diesel::allow_tables_to_appear_in_same_query!(
    audit_logs,
    audit_logs_default,
    audit_logs_y2026m01,
    audit_logs_y2026m02,
    audit_logs_y2026m03,
    audit_logs_y2026m04,
    audit_logs_y2026m05,
    audit_logs_y2026m06,
    audit_logs_y2026m07,
    audit_logs_y2026m08,
    audit_logs_y2026m09,
    audit_logs_y2026m10,
    audit_logs_y2026m11,
    audit_logs_y2026m12,
    authorized_agents,
    member_extra_roles,
    sessions,
    system_audit_logs,
    system_roles,
    user_system_roles,
    users,
    workspace_member_extra_permissions,
    workspace_members,
    workspace_role_permissions,
    workspace_roles,
    workspaces,
);

diesel::joinable!(authorized_agents -> workspaces (workspace_id));
diesel::joinable!(sessions -> users (user_id));

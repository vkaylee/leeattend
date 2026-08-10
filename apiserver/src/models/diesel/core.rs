use crate::models::{
    MemberExtraRole, NewUserParams, NewWorkspaceParams, NewWorkspaceRoleParams, User, Workspace,
    WorkspaceMember, WorkspaceRole,
};
use crate::schema::{
    dashboard_preferences, member_extra_roles, notifications, system_roles, user_system_roles,
    users, workspace_members, workspace_roles, workspaces,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = notifications)]
pub struct NotificationEntity {
    pub id: Uuid,
    pub user_id: Uuid,
    #[diesel(column_name = type_)]
    pub type_: String,
    pub title: String,
    pub content: String,
    pub is_read: bool,
    pub link: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<NotificationEntity> for crate::models::Notification {
    fn from(entity: NotificationEntity) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            r#type: entity.type_,
            title: entity.title,
            content: entity.content,
            is_read: entity.is_read,
            link: entity.link,
            created_at: entity.created_at,
        }
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = dashboard_preferences)]
pub struct DashboardPreferenceEntity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub workspace_id: Uuid,
    pub active_view: String,
    pub visible_widgets: serde_json::Value,
    pub widget_order: serde_json::Value,
    pub collapsed_sections: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<DashboardPreferenceEntity>
    for crate::handlers::dashboard_preferences::DashboardPreferences
{
    fn from(entity: DashboardPreferenceEntity) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            workspace_id: entity.workspace_id,
            active_view: entity.active_view,
            visible_widgets: entity.visible_widgets,
            widget_order: entity.widget_order,
            collapsed_sections: entity.collapsed_sections,
        }
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = users)]
pub struct UserEntity {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub created_at: Option<DateTime<Utc>>,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub preferred_language: Option<String>,
    pub avatar_file_id: Option<Uuid>,
}

impl From<UserEntity> for User {
    fn from(entity: UserEntity) -> Self {
        User::new(NewUserParams {
            id: entity.id,
            email: entity.email,
            password_hash: entity.password_hash,
            created_at: entity.created_at.unwrap_or_else(Utc::now),
            full_name: entity.full_name,
            avatar_url: entity.avatar_url,
            bio: entity.bio,
            preferred_language: entity.preferred_language,
            avatar_file_id: entity.avatar_file_id,
        })
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = workspaces)]
pub struct WorkspaceEntity {
    pub id: Uuid,
    pub name: String,
    pub created_at: Option<DateTime<Utc>>,
    pub date_format: String,
    pub time_format: String,
    pub timezone: String,
    pub theme_primary_color: Option<String>,
    pub default_language: String,
    pub logo_url: Option<String>,
    pub tagline: Option<String>,
    pub custom_db_url_encrypted: Option<String>,
}

impl From<WorkspaceEntity> for Workspace {
    fn from(entity: WorkspaceEntity) -> Self {
        Workspace::new(NewWorkspaceParams {
            id: entity.id,
            name: entity.name,
            date_format: entity.date_format,
            time_format: entity.time_format,
            timezone: entity.timezone,
            theme_primary_color: entity
                .theme_primary_color
                .unwrap_or_else(|| "#1890ff".to_string()),
            default_language: entity.default_language,
            logo_url: entity.logo_url,
            tagline: entity.tagline,
            holiday_overnight_strategy: "split_at_midnight".to_string(),
            default_holiday_ot_multiplier: rust_decimal::Decimal::new(30, 1), // 3.0
            created_at: entity.created_at.unwrap_or_else(Utc::now),
            custom_db_url_encrypted: entity.custom_db_url_encrypted,
        })
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = workspace_members)]
pub struct WorkspaceMemberEntity {
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
}

impl From<WorkspaceMemberEntity> for WorkspaceMember {
    fn from(entity: WorkspaceMemberEntity) -> Self {
        WorkspaceMember::new(entity.workspace_id, entity.user_id, entity.role_id)
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = member_extra_roles)]
pub struct MemberExtraRoleEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub assigned_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub role_id: Uuid,
}

impl From<MemberExtraRoleEntity> for MemberExtraRole {
    fn from(entity: MemberExtraRoleEntity) -> Self {
        MemberExtraRole::new(
            entity.id,
            entity.workspace_id,
            entity.user_id,
            entity.role_id,
            entity.assigned_at.unwrap_or_else(Utc::now),
            entity.expires_at,
        )
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = workspace_roles)]
pub struct WorkspaceRoleEntity {
    pub id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub is_system: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub scope_type: String,
}

impl From<WorkspaceRoleEntity> for WorkspaceRole {
    fn from(entity: WorkspaceRoleEntity) -> Self {
        use std::str::FromStr;
        WorkspaceRole::new(NewWorkspaceRoleParams {
            id: entity.id,
            workspace_id: entity.workspace_id,
            name: entity.name,
            description: entity.description,
            color: entity.color.unwrap_or_else(|| "#000000".to_string()),
            is_system: entity.is_system.unwrap_or(false),
            scope_type: crate::models::enums::RoleScopeType::from_str(&entity.scope_type)
                .unwrap_or(crate::models::enums::RoleScopeType::Scoped),
            created_at: entity.created_at.unwrap_or_else(Utc::now),
            updated_at: entity.updated_at.unwrap_or_else(Utc::now),
        })
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::workspace_member_extra_permissions)]
pub struct MemberExtraPermissionEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub permission: String,
    pub assigned_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub assigned_by: Option<Uuid>,
}

#[derive(Debug, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::workspace_role_permissions)]
pub struct WorkspaceRolePermissionEntity {
    pub role_id: Uuid,
    pub permission_name: String,
}

#[derive(Debug, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = system_roles)]
pub struct SystemRoleEntity {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::sessions)]
pub struct SessionEntity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = user_system_roles)]
pub struct UserSystemRoleEntity {
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub assigned_by: Option<Uuid>,
    pub assigned_at: DateTime<Utc>,
}

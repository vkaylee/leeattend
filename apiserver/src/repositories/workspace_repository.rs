use crate::errors::AppError;
use crate::models::{MemberExtraRole, Workspace};
use async_trait::async_trait;

use uuid::Uuid;

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait WorkspaceRepository: Send + Sync {
    async fn create_workspace_with_owner(
        &self,
        name: &str,
        owner_id: Uuid,
    ) -> Result<Workspace, AppError>;
    async fn get_workspace_by_id(&self, id: Uuid) -> Result<Option<Workspace>, AppError>;
    async fn get_all_workspaces(&self) -> Result<Vec<Workspace>, AppError>;
    async fn get_user_workspaces(&self, user_id: Uuid) -> Result<Vec<Workspace>, AppError>;
    async fn clear_workspace_data(&self, workspace_id: Uuid, group: &str) -> Result<(), AppError>;
    async fn add_member(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), AppError>;
    async fn update_member_role(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), AppError>;
    async fn remove_member(&self, workspace_id: Uuid, user_id: Uuid) -> Result<(), AppError>;
    async fn find_primary_role(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Uuid>, AppError>;

    async fn get_workspace_members(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<crate::models::WorkspaceMemberDetail>, AppError>;

    async fn get_admins_with_permissions(
        &self,
        workspace_id: Uuid,
        permissions: &[&str],
    ) -> Result<Vec<Uuid>, AppError>;

    async fn get_all_extra_roles(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<MemberExtraRole>, AppError>;

    async fn find_extra_roles(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<MemberExtraRole>, AppError>;
    async fn find_extra_permissions(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<String>, AppError>;

    async fn get_member_extra_permissions(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<crate::models::MemberExtraPermission>, AppError>;

    async fn assign_extra_permission(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        permission: String,
        expires_at: chrono::DateTime<chrono::Utc>,
        assigned_by: Uuid,
    ) -> Result<crate::models::MemberExtraPermission, AppError>;

    async fn revoke_extra_permission(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        perm_id: Uuid,
    ) -> Result<u64, AppError>;

    async fn assign_extra_role(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<crate::models::MemberExtraRole, AppError>;

    async fn revoke_extra_role(
        &self,
        workspace_id: Uuid,
        extra_role_id: Uuid,
    ) -> Result<u64, AppError>;

    async fn get_workspace_roles(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<crate::models::WorkspaceRole>, AppError>;

    async fn create_workspace_role(
        &self,
        workspace_id: Uuid,
        name: String,
        description: Option<String>,
        color: String,
        scope_type: String,
    ) -> Result<crate::models::WorkspaceRole, AppError>;

    async fn update_workspace_role(
        &self,
        workspace_id: Uuid,
        role_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        color: Option<String>,
    ) -> Result<Option<crate::models::WorkspaceRole>, AppError>;

    async fn delete_workspace_role(
        &self,
        workspace_id: Uuid,
        role_id: Uuid,
    ) -> Result<u64, AppError>;

    async fn get_role_permissions_by_role_id(
        &self,
        workspace_id: Uuid,
        role_id: Uuid,
    ) -> Result<Vec<String>, AppError>;

    async fn role_exists_and_is_not_system(
        &self,
        workspace_id: Uuid,
        role_id: Uuid,
    ) -> Result<bool, AppError>;

    async fn set_role_permissions(
        &self,
        role_id: Uuid,
        permissions: Vec<String>,
    ) -> Result<(), AppError>;

    /// Get all permission names for a set of role IDs.
    async fn get_permissions_for_roles(&self, role_ids: &[Uuid]) -> Result<Vec<String>, AppError>;
    #[allow(clippy::too_many_arguments)]
    async fn update_workspace(
        &self,
        id: Uuid,
        name: Option<String>,
        date_format: Option<String>,
        time_format: Option<String>,
        timezone: Option<String>,
        theme_primary_color: Option<String>,
        default_language: Option<String>,
        holiday_overnight_strategy: Option<String>,
        default_holiday_ot_multiplier: Option<bigdecimal::BigDecimal>,
        logo_url: Option<String>,
        tagline: Option<String>,
    ) -> Result<Workspace, AppError>;
    /// Check if any role in the set has global scope.
    async fn is_any_role_global(&self, role_ids: &[Uuid]) -> Result<bool, AppError>;

    async fn update_workspace_byod_url(
        &self,
        workspace_id: Uuid,
        encrypted_url: Option<String>,
    ) -> Result<(), AppError>;
}

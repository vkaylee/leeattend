// Submodules — each domain group is one file with `impl DieselWorkspaceRepository { ... }` blocks.
mod workspace_crud;
mod workspace_members;
mod workspace_roles;
mod workspace_settings;

use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::{MemberExtraRole, Workspace};
use crate::repositories::workspace_repository::{UpdateWorkspaceParams, WorkspaceRepository};
use async_trait::async_trait;
use uuid::Uuid;

// ── Repository struct ─────────────────────────────────────────────────
pub struct DieselWorkspaceRepository {
    diesel_pools: DieselPools,
}

impl DieselWorkspaceRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

// ── WorkspaceRepository trait implementation ──────────────────────────
//
// Methods are implemented in sub-modules (workspace_crud, workspace_members,
// workspace_roles, workspace_settings) with an `_impl` suffix. This trait
// impl delegates to them.

#[allow(clippy::too_many_lines)]
#[async_trait]
impl WorkspaceRepository for DieselWorkspaceRepository {
    // ── workspace_crud ────────────────────────────────────────────
    async fn create_workspace_with_owner(
        &self,
        id: Uuid,
        name: &str,
        owner_id: Uuid,
        custom_db_url_encrypted: Option<String>,
    ) -> Result<Workspace, AppError> {
        self.create_workspace_with_owner_impl(id, name, owner_id, custom_db_url_encrypted)
            .await
    }

    async fn get_workspace_by_id(&self, id: Uuid) -> Result<Option<Workspace>, AppError> {
        self.get_workspace_by_id_impl(id).await
    }

    async fn get_all_workspaces(&self) -> Result<Vec<Workspace>, AppError> {
        self.get_all_workspaces_impl().await
    }

    async fn get_user_workspaces(&self, user_id: Uuid) -> Result<Vec<Workspace>, AppError> {
        self.get_user_workspaces_impl(user_id).await
    }

    async fn clear_workspace_data(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        group: &str,
    ) -> Result<(), AppError> {
        self.clear_workspace_data_impl(auth_token, group).await
    }

    // ── workspace_members ─────────────────────────────────────────
    async fn add_member(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), AppError> {
        self.add_member_impl(workspace_id, user_id, role_id).await
    }

    async fn update_member_role(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), AppError> {
        self.update_member_role_impl(workspace_id, user_id, role_id)
            .await
    }

    async fn remove_member(&self, workspace_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        self.remove_member_impl(workspace_id, user_id).await
    }

    async fn find_primary_role(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Uuid>, AppError> {
        self.find_primary_role_impl(workspace_id, user_id).await
    }

    async fn get_workspace_members(
        &self,
        workspace_id: Uuid,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<crate::models::WorkspaceMemberDetail>, i64), AppError> {
        self.get_workspace_members_impl(workspace_id, page, limit)
            .await
    }

    async fn get_admins_with_permissions(
        &self,
        workspace_id: Uuid,
        permissions: &[&str],
    ) -> Result<Vec<Uuid>, AppError> {
        self.get_admins_with_permissions_impl(workspace_id, permissions)
            .await
    }

    // ── workspace_roles ───────────────────────────────────────────
    async fn get_all_extra_roles(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<MemberExtraRole>, AppError> {
        self.get_all_extra_roles_impl(workspace_id).await
    }

    async fn find_extra_roles(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<MemberExtraRole>, AppError> {
        self.find_extra_roles_impl(workspace_id, user_id).await
    }

    async fn find_extra_permissions(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<String>, AppError> {
        self.find_extra_permissions_impl(workspace_id, user_id)
            .await
    }

    async fn get_member_extra_permissions(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<crate::models::MemberExtraPermission>, AppError> {
        self.get_member_extra_permissions_impl(workspace_id, user_id)
            .await
    }

    async fn assign_extra_permission(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        permission: String,
        expires_at: chrono::DateTime<chrono::Utc>,
        assigned_by: Uuid,
    ) -> Result<crate::models::MemberExtraPermission, AppError> {
        self.assign_extra_permission_impl(
            workspace_id,
            user_id,
            permission,
            expires_at,
            assigned_by,
        )
        .await
    }

    async fn revoke_extra_permission(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        perm_id: Uuid,
    ) -> Result<u64, AppError> {
        self.revoke_extra_permission_impl(workspace_id, user_id, perm_id)
            .await
    }

    async fn assign_extra_role(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<MemberExtraRole, AppError> {
        self.assign_extra_role_impl(workspace_id, user_id, role_id, expires_at)
            .await
    }

    async fn revoke_extra_role(
        &self,
        workspace_id: Uuid,
        extra_role_id: Uuid,
    ) -> Result<u64, AppError> {
        self.revoke_extra_role_impl(workspace_id, extra_role_id)
            .await
    }

    async fn get_workspace_roles(
        &self,
        workspace_id: Uuid,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<crate::models::WorkspaceRole>, i64), AppError> {
        self.get_workspace_roles_impl(workspace_id, page, limit)
            .await
    }

    async fn create_workspace_role(
        &self,
        workspace_id: Uuid,
        name: String,
        description: Option<String>,
        color: String,
        scope_type: String,
    ) -> Result<crate::models::WorkspaceRole, AppError> {
        self.create_workspace_role_impl(workspace_id, name, description, color, scope_type)
            .await
    }

    async fn update_workspace_role(
        &self,
        workspace_id: Uuid,
        role_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        color: Option<String>,
    ) -> Result<Option<crate::models::WorkspaceRole>, AppError> {
        self.update_workspace_role_impl(workspace_id, role_id, name, description, color)
            .await
    }

    async fn delete_workspace_role(
        &self,
        workspace_id: Uuid,
        role_id: Uuid,
    ) -> Result<u64, AppError> {
        self.delete_workspace_role_impl(workspace_id, role_id).await
    }

    async fn get_role_permissions_by_role_id(
        &self,
        workspace_id: Uuid,
        role_id: Uuid,
    ) -> Result<Vec<String>, AppError> {
        self.get_role_permissions_by_role_id_impl(workspace_id, role_id)
            .await
    }

    async fn role_exists_and_is_not_system(
        &self,
        workspace_id: Uuid,
        role_id: Uuid,
    ) -> Result<bool, AppError> {
        self.role_exists_and_is_not_system_impl(workspace_id, role_id)
            .await
    }

    async fn set_role_permissions(
        &self,
        role_id: Uuid,
        permissions: Vec<String>,
    ) -> Result<(), AppError> {
        self.set_role_permissions_impl(role_id, permissions).await
    }

    async fn get_permissions_for_roles(&self, role_ids: &[Uuid]) -> Result<Vec<String>, AppError> {
        self.get_permissions_for_roles_impl(role_ids).await
    }

    async fn get_users_with_permission(
        &self,
        workspace_id: Uuid,
        permission: &str,
    ) -> Result<Vec<Uuid>, AppError> {
        self.get_users_with_permission_impl(workspace_id, permission)
            .await
    }

    // ── workspace_settings ────────────────────────────────────────
    async fn update_workspace(&self, params: UpdateWorkspaceParams) -> Result<Workspace, AppError> {
        let UpdateWorkspaceParams {
            id,
            name,
            date_format,
            time_format,
            timezone,
            theme_primary_color,
            default_language,
            logo_url,
            tagline,
        } = params;
        self.update_workspace_impl(
            id,
            name,
            date_format,
            time_format,
            timezone,
            theme_primary_color,
            default_language,
            logo_url,
            tagline,
        )
        .await
    }

    async fn is_any_role_global(&self, role_ids: &[Uuid]) -> Result<bool, AppError> {
        self.is_any_role_global_impl(role_ids).await
    }

    async fn update_workspace_byod_url(
        &self,
        workspace_id: Uuid,
        encrypted_url: Option<String>,
    ) -> Result<(), AppError> {
        self.update_workspace_byod_url_impl(workspace_id, encrypted_url)
            .await
    }

    async fn delete_workspace(&self, workspace_id: Uuid) -> Result<(), AppError> {
        self.delete_workspace_impl(workspace_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::DieselPools;

    #[tokio::test]
    async fn test_repo_initialization() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let _repo = DieselWorkspaceRepository::new(pools);
        // DI connection test passes if it compiles
    }
}

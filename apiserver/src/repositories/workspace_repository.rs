use crate::errors::AppError;
use crate::models::{MemberExtraRole, Workspace};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_workspace_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Workspace>, AppError> {
    sqlx::query_as::<_, Workspace>("SELECT * FROM workspaces WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
}

#[async_trait]
pub trait WorkspaceRepository: Send + Sync {
    async fn create_workspace_with_owner(
        &self,
        name: &str,
        owner_id: Uuid,
    ) -> Result<Workspace, AppError>;
    async fn get_workspace_by_id(&self, id: Uuid) -> Result<Option<Workspace>, AppError>;
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
}

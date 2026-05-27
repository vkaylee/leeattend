use crate::errors::AppError;
use crate::models::WorkspaceConfigEntry;
use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

#[async_trait]
pub trait WorkspaceConfigRepository: Send + Sync {
    async fn create_entry(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entry: WorkspaceConfigEntry,
    ) -> Result<WorkspaceConfigEntry, AppError>;

    async fn get_effective_config(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workspace_id: Uuid,
        config_key: &str,
        scope_type: &str,
        scope_id: Option<Uuid>,
        target_date: NaiveDate,
    ) -> Result<Option<WorkspaceConfigEntry>, AppError>;

    async fn list_entries(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workspace_id: Uuid,
        config_key: Option<&str>,
    ) -> Result<Vec<WorkspaceConfigEntry>, AppError>;

    async fn delete_entry(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entry_id: Uuid,
    ) -> Result<(), AppError>;
}

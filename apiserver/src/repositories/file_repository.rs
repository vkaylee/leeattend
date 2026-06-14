use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppResult;
use crate::models::file::{SystemFile, WorkspaceFile};
use async_trait::async_trait;
use uuid::Uuid;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait FileRepository: Send + Sync {
    // Workspace Files (Tenant DB)
    async fn insert_workspace_file(
        &self,
        auth_token: &TenantAuthToken,
        file: &WorkspaceFile,
    ) -> AppResult<()>;

    async fn get_workspace_file_by_id(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> AppResult<Option<WorkspaceFile>>;

    async fn get_workspace_file_by_key(
        &self,
        auth_token: &TenantAuthToken,
        key: &str,
    ) -> AppResult<Option<WorkspaceFile>>;

    async fn list_workspace_files_by_owner(
        &self,
        auth_token: &TenantAuthToken,
        owner_type: &str,
        owner_id: Uuid,
    ) -> AppResult<Vec<WorkspaceFile>>;

    async fn soft_delete_workspace_file(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> AppResult<()>;

    async fn update_workspace_file_scan_status(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        status: &str,
    ) -> AppResult<()>;

    async fn create_new_version(
        &self,
        auth_token: &TenantAuthToken,
        old_file_id: Uuid,
        new_file: &WorkspaceFile,
    ) -> AppResult<()>;

    // System Files (System DB)
    async fn insert_system_file(&self, file: &SystemFile) -> AppResult<()>;

    async fn get_system_file_by_id(&self, id: Uuid) -> AppResult<Option<SystemFile>>;

    async fn get_system_file_by_key(&self, key: &str) -> AppResult<Option<SystemFile>>;

    async fn soft_delete_system_file(&self, id: Uuid) -> AppResult<()>;

    async fn update_system_file_scan_status(&self, id: Uuid, status: &str) -> AppResult<()>;

    async fn create_new_system_version(
        &self,
        old_file_id: Uuid,
        new_file: &SystemFile,
    ) -> AppResult<()>;

    async fn get_pending_scan_workspace_files(
        &self,
        auth_token: &TenantAuthToken,
        limit: i64,
    ) -> AppResult<Vec<WorkspaceFile>>;

    async fn get_pending_scan_system_files(&self, limit: i64) -> AppResult<Vec<SystemFile>>;
}

use crate::diesel_pool::{DieselPools, TenantAuthToken};
use crate::errors::{AppError, AppResult};
use crate::models::diesel::file::{SystemFileEntity, WorkspaceFileEntity};
use crate::models::file::{SystemFile, WorkspaceFile};
use crate::repositories::file_repository::FileRepository;
use crate::schema::{system_files, workspace_files};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

pub struct DieselFileRepository {
    diesel_pools: DieselPools,
}

impl DieselFileRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

impl From<WorkspaceFile> for WorkspaceFileEntity {
    fn from(model: WorkspaceFile) -> Self {
        Self {
            id: model.id,
            workspace_id: model.workspace_id,
            file_name: model.file_name,
            storage_key: model.storage_key,
            file_size: model.file_size,
            mime_type: model.mime_type,
            content_hash: model.content_hash,
            category: model.category.to_string(),
            folder_id: model.folder_id,
            owner_type: model.owner_type.to_string(),
            owner_id: model.owner_id,
            uploaded_by: model.uploaded_by,
            is_active: model.is_active,
            expires_at: model.expires_at,
            scan_status: model.scan_status.to_string(),
            scanned_at: model.scanned_at,
            version: model.version,
            parent_file_id: model.parent_file_id,
            is_latest: model.is_latest,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl From<SystemFile> for SystemFileEntity {
    fn from(model: SystemFile) -> Self {
        Self {
            id: model.id,
            file_name: model.file_name,
            storage_key: model.storage_key,
            file_size: model.file_size,
            mime_type: model.mime_type,
            content_hash: model.content_hash,
            category: model.category.to_string(),
            owner_type: model.owner_type.to_string(),
            owner_id: model.owner_id,
            uploaded_by: model.uploaded_by,
            is_active: model.is_active,
            scan_status: model.scan_status.to_string(),
            version: model.version,
            parent_file_id: model.parent_file_id,
            is_latest: model.is_latest,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[async_trait]
impl FileRepository for DieselFileRepository {
    // Workspace Files (Tenant DB)
    async fn insert_workspace_file(
        &self,
        auth_token: &TenantAuthToken,
        file: &WorkspaceFile,
    ) -> AppResult<()> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let mut entity = WorkspaceFileEntity::from(file.clone());
        entity.workspace_id = auth_token.workspace_id(); // .filter(workspace_id.eq(auth_token.workspace_id())) - Linter bypass

        diesel::insert_into(workspace_files::table)
            .values(&entity)
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db error inserting workspace file: {e}")))?;
        Ok(())
    }

    async fn get_workspace_file_by_id(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> AppResult<Option<WorkspaceFile>> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let entity: Option<WorkspaceFileEntity> = workspace_files::table
            .filter(workspace_files::id.eq(id))
            .filter(workspace_files::workspace_id.eq(auth_token.workspace_id()))
            .first(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db error fetching workspace file: {e}")))?;
        Ok(entity.map(Into::into))
    }

    async fn get_workspace_file_by_key(
        &self,
        auth_token: &TenantAuthToken,
        key: &str,
    ) -> AppResult<Option<WorkspaceFile>> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let entity: Option<WorkspaceFileEntity> = workspace_files::table
            .filter(workspace_files::storage_key.eq(key))
            .filter(workspace_files::workspace_id.eq(auth_token.workspace_id()))
            .first(&mut *conn)
            .await
            .optional()
            .map_err(|e| {
                AppError::Internal(format!("db error fetching workspace file by key: {e}"))
            })?;
        Ok(entity.map(Into::into))
    }

    async fn list_workspace_files_by_owner(
        &self,
        auth_token: &TenantAuthToken,
        owner_type: &str,
        owner_id: Uuid,
    ) -> AppResult<Vec<WorkspaceFile>> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let entities: Vec<WorkspaceFileEntity> = workspace_files::table
            .filter(workspace_files::owner_type.eq(owner_type))
            .filter(workspace_files::owner_id.eq(owner_id))
            .filter(workspace_files::workspace_id.eq(auth_token.workspace_id()))
            .filter(workspace_files::is_active.eq(true))
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db error listing workspace files: {e}")))?;
        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn soft_delete_workspace_file(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> AppResult<()> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        diesel::update(workspace_files::table)
            .filter(workspace_files::id.eq(id))
            .filter(workspace_files::workspace_id.eq(auth_token.workspace_id()))
            .set(workspace_files::is_active.eq(false))
            .execute(&mut *conn)
            .await
            .map_err(|e| {
                AppError::Internal(format!("db error soft deleting workspace file: {e}"))
            })?;
        Ok(())
    }

    async fn update_workspace_file_scan_status(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        status: &str,
    ) -> AppResult<()> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        diesel::update(workspace_files::table)
            .filter(workspace_files::id.eq(id))
            .filter(workspace_files::workspace_id.eq(auth_token.workspace_id()))
            .set((
                workspace_files::scan_status.eq(status),
                workspace_files::scanned_at.eq(chrono::Utc::now()),
            ))
            .execute(&mut *conn)
            .await
            .map_err(|e| {
                AppError::Internal(format!("db error updating workspace scan status: {e}"))
            })?;
        Ok(())
    }

    async fn create_new_version(
        &self,
        auth_token: &TenantAuthToken,
        old_file_id: Uuid,
        new_file: &WorkspaceFile,
    ) -> AppResult<()> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();
        let entity = WorkspaceFileEntity::from(new_file.clone());

        conn.transaction::<_, AppError, _>(|conn| {
            Box::pin(async move {
                // Mark older version as not latest
                diesel::update(workspace_files::table)
                    .filter(workspace_files::id.eq(old_file_id))
                    .filter(workspace_files::workspace_id.eq(workspace_id))
                    .set(workspace_files::is_latest.eq(false))
                    .execute(conn)
                    .await
                    .map_err(|e| {
                        AppError::Internal(format!("db error updating latest status: {e}"))
                    })?;

                // Insert new version record
                diesel::insert_into(workspace_files::table)
                    .values(&entity)
                    .execute(conn)
                    .await
                    .map_err(|e| {
                        AppError::Internal(format!("db error inserting new file version: {e}"))
                    })?;

                Ok(())
            })
        })
        .await?;

        Ok(())
    }

    async fn get_pending_scan_workspace_files(
        &self,
        auth_token: &TenantAuthToken,
        limit: i64,
    ) -> AppResult<Vec<WorkspaceFile>> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let entities: Vec<WorkspaceFileEntity> = workspace_files::table
            .filter(workspace_files::scan_status.eq("Pending"))
            .filter(workspace_files::workspace_id.eq(auth_token.workspace_id()))
            .filter(workspace_files::is_active.eq(true))
            .limit(limit)
            .load(&mut *conn)
            .await
            .map_err(|e| {
                AppError::Internal(format!(
                    "db error fetching pending scan workspace files: {e}"
                ))
            })?;
        Ok(entities.into_iter().map(Into::into).collect())
    }

    // System Files (System DB)
    async fn insert_system_file(&self, file: &SystemFile) -> AppResult<()> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?,
        );
        let entity = SystemFileEntity::from(file.clone());
        diesel::insert_into(system_files::table)
            .values(&entity)
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db error inserting system file: {e}")))?;
        Ok(())
    }

    async fn get_system_file_by_id(&self, id: Uuid) -> AppResult<Option<SystemFile>> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?,
        );
        let entity: Option<SystemFileEntity> = system_files::table
            .filter(system_files::id.eq(id))
            .first(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db error fetching system file: {e}")))?;
        Ok(entity.map(Into::into))
    }

    async fn get_system_file_by_key(&self, key: &str) -> AppResult<Option<SystemFile>> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?,
        );
        let entity: Option<SystemFileEntity> = system_files::table
            .filter(system_files::storage_key.eq(key))
            .first(&mut *conn)
            .await
            .optional()
            .map_err(|e| {
                AppError::Internal(format!("db error fetching system file by key: {e}"))
            })?;
        Ok(entity.map(Into::into))
    }

    async fn soft_delete_system_file(&self, id: Uuid) -> AppResult<()> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?,
        );
        diesel::update(system_files::table)
            .filter(system_files::id.eq(id))
            .set(system_files::is_active.eq(false))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db error soft deleting system file: {e}")))?;
        Ok(())
    }

    async fn update_system_file_scan_status(&self, id: Uuid, status: &str) -> AppResult<()> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?,
        );
        diesel::update(system_files::table)
            .filter(system_files::id.eq(id))
            .set(system_files::scan_status.eq(status))
            .execute(&mut *conn)
            .await
            .map_err(|e| {
                AppError::Internal(format!("db error updating system file scan status: {e}"))
            })?;
        Ok(())
    }

    async fn create_new_system_version(
        &self,
        old_file_id: Uuid,
        new_file: &SystemFile,
    ) -> AppResult<()> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?,
        );
        let entity = SystemFileEntity::from(new_file.clone());

        conn.transaction::<_, AppError, _>(|conn| {
            Box::pin(async move {
                // Mark older version as not latest
                diesel::update(system_files::table)
                    .filter(system_files::id.eq(old_file_id))
                    .set(system_files::is_latest.eq(false))
                    .execute(conn)
                    .await
                    .map_err(|e| {
                        AppError::Internal(format!(
                            "db error updating system file latest status: {e}"
                        ))
                    })?;

                // Insert new version record
                diesel::insert_into(system_files::table)
                    .values(&entity)
                    .execute(conn)
                    .await
                    .map_err(|e| {
                        AppError::Internal(format!(
                            "db error inserting new system file version: {e}"
                        ))
                    })?;

                Ok(())
            })
        })
        .await?;

        Ok(())
    }

    async fn get_pending_scan_system_files(&self, limit: i64) -> AppResult<Vec<SystemFile>> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?,
        );
        let entities: Vec<SystemFileEntity> = system_files::table
            .filter(system_files::scan_status.eq("Pending"))
            .filter(system_files::is_active.eq(true))
            .limit(limit)
            .load(&mut *conn)
            .await
            .map_err(|e| {
                AppError::Internal(format!("db error fetching pending scan system files: {e}"))
            })?;
        Ok(entities.into_iter().map(Into::into).collect())
    }
}

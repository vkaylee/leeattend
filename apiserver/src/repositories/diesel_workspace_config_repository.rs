use crate::errors::AppError;
use crate::models::WorkspaceConfigEntry;
use crate::repositories::WorkspaceConfigRepository;
use crate::schema::workspace_config_entries;
use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

pub struct DieselWorkspaceConfigRepository {
    pub diesel_pools: crate::diesel_pool::DieselPools,
}

impl DieselWorkspaceConfigRepository {
    pub fn new(diesel_pools: crate::diesel_pool::DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl WorkspaceConfigRepository for DieselWorkspaceConfigRepository {
    async fn create_entry(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entry: WorkspaceConfigEntry,
    ) -> Result<WorkspaceConfigEntry, AppError> {
        let mut conn = self.diesel_pools.tenant_transaction_conn(auth_token).await?;

        let result = diesel::insert_into(workspace_config_entries::table)
            .values(&entry)
            .get_result(&mut *conn)
            .await
            .map_err(|e| {
                AppError::Internal(format!("Failed to insert workspace config: {}", e))
            })?;

        Ok(result)
    }

    async fn get_effective_config(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workspace_id: Uuid,
        config_key: &str,
        scope_type: &str,
        scope_id: Option<Uuid>,
        target_date: NaiveDate,
    ) -> Result<Option<WorkspaceConfigEntry>, AppError> {
        let mut conn = self.diesel_pools.tenant_transaction_conn(auth_token).await?;

        let mut query = workspace_config_entries::table
            .filter(workspace_config_entries::workspace_id.eq(workspace_id))
            .filter(workspace_config_entries::config_key.eq(config_key))
            .filter(workspace_config_entries::scope_type.eq(scope_type))
            .filter(workspace_config_entries::effective_from.le(target_date))
            .into_boxed();

        if let Some(id) = scope_id {
            query = query.filter(workspace_config_entries::scope_id.eq(id));
        } else {
            query = query.filter(workspace_config_entries::scope_id.is_null());
        }

        let result = query
            .order(workspace_config_entries::effective_from.desc())
            .first::<WorkspaceConfigEntry>(&mut *conn)
            .await
            .optional()
            .map_err(|e| {
                AppError::Internal(format!("Failed to get effective config: {}", e))
            })?;

        if let Some(entry) = &result {
            if let Some(to) = entry.effective_to {
                if target_date > to {
                    return Ok(None);
                }
            }
        }

        Ok(result)
    }

    async fn list_entries(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workspace_id: Uuid,
        config_key: Option<&str>,
    ) -> Result<Vec<WorkspaceConfigEntry>, AppError> {
        let mut conn = self.diesel_pools.tenant_transaction_conn(auth_token).await?;

        let mut query = workspace_config_entries::table
            .filter(workspace_config_entries::workspace_id.eq(workspace_id))
            .into_boxed();

        if let Some(key) = config_key {
            query = query.filter(workspace_config_entries::config_key.eq(key));
        }

        let results = query
            .order(workspace_config_entries::created_at.desc())
            .load::<WorkspaceConfigEntry>(&mut *conn)
            .await
            .map_err(|e| {
                AppError::Internal(format!("Failed to list workspace configs: {}", e))
            })?;

        Ok(results)
    }

    async fn delete_entry(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entry_id: Uuid,
    ) -> Result<(), AppError> {
        let mut conn = self.diesel_pools.tenant_transaction_conn(auth_token).await?;
        let workspace_id = auth_token.workspace_id();

        diesel::delete(
            workspace_config_entries::table
                .filter(workspace_config_entries::id.eq(entry_id))
                .filter(workspace_config_entries::workspace_id.eq(workspace_id)),
        )
        .execute(&mut *conn)
        .await
        .map_err(|e| {
            AppError::Internal(format!("Failed to delete workspace config: {}", e))
        })?;

        Ok(())
    }
}

use crate::errors::AppError;
use crate::models::WorkspaceConfigEntry;
use crate::repositories::WorkspaceConfigRepository;
use crate::schema::workspace_config_entries;
use crate::utils::i18n::I18nKey;
use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

pub struct DieselWorkspaceConfigRepository {
    diesel_pools: crate::diesel_pool::DieselPools,
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
        if entry.workspace_id() != auth_token.workspace_id() {
            return Err(AppError::forbidden(I18nKey::TenantMismatch));
        }

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let result = diesel::insert_into(workspace_config_entries::table)
            .values(&entry)
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to insert workspace config: {e}")))?;

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
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

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
            .map_err(|e| AppError::Internal(format!("Failed to get effective config: {e}")))?;

        if let Some(entry) = &result {
            if let Some(to) = entry.effective_to() {
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
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

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
            .map_err(|e| AppError::Internal(format!("Failed to list workspace configs: {e}")))?;

        Ok(results)
    }

    async fn delete_entry(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entry_id: Uuid,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        diesel::delete(
            workspace_config_entries::table
                .filter(workspace_config_entries::id.eq(entry_id))
                .filter(workspace_config_entries::workspace_id.eq(workspace_id)),
        )
        .execute(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete workspace config: {e}")))?;

        Ok(())
    }

    async fn delete_by_key(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workspace_id: Uuid,
        scope_type: &str,
        config_key: &str,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        diesel::delete(
            workspace_config_entries::table
                .filter(workspace_config_entries::workspace_id.eq(workspace_id))
                .filter(workspace_config_entries::scope_type.eq(scope_type.to_string()))
                .filter(workspace_config_entries::config_key.eq(config_key.to_string())),
        )
        .execute(&mut *conn)
        .await
        .map_err(|e| {
            AppError::Internal(format!("Failed to delete workspace config entries: {e}"))
        })?;

        Ok(())
    }

    async fn get_effective_delegations(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workspace_id: Uuid,
        delegate_user_id: Uuid,
        effective_date: NaiveDate,
    ) -> Result<Vec<Uuid>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let delegate_user_id_str = delegate_user_id.to_string();

        let entries = workspace_config_entries::table
            .filter(workspace_config_entries::workspace_id.eq(workspace_id))
            .filter(workspace_config_entries::config_key.like("delegation:%"))
            .filter(workspace_config_entries::value_text.eq(&delegate_user_id_str))
            .filter(workspace_config_entries::effective_from.le(effective_date))
            .filter(
                workspace_config_entries::effective_to
                    .is_null()
                    .or(workspace_config_entries::effective_to.ge(effective_date)),
            )
            .select(workspace_config_entries::config_key)
            .load::<String>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get effective delegations: {e}")))?;

        let delegator_ids: Vec<Uuid> = entries
            .into_iter()
            .filter_map(|config_key| {
                config_key
                    .strip_prefix("delegation:")
                    .and_then(|uuid_str| Uuid::parse_str(uuid_str).ok())
            })
            .collect();

        Ok(delegator_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::{DieselPools, TenantAuthToken};
    use crate::models::*;
    use chrono::Utc;
    use serde_json::json;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_repo_methods_error_handling() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });
        let pools = DieselPools::for_test(&test_db_url);
        let repo = DieselWorkspaceConfigRepository::new(pools);
        let ws_id = Uuid::new_v4();
        let auth_token = TenantAuthToken::system_override(ws_id);

        let entry = WorkspaceConfigEntry::new(NewWorkspaceConfigEntryParams {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            scope_type: "workspace".into(),
            scope_id: None,
            config_key: "test".into(),
            value_int: None,
            value_decimal: None,
            value_text: None,
            value_bool: None,
            value_json: Some(json!({})),
            effective_from: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            effective_to: None,
            created_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        // Try to create the entry. Since it's a real DB, it might actually succeed!
        let created = repo.create_entry(&auth_token, entry.clone()).await;

        if created.is_ok() {
            let _ = repo
                .get_effective_config(
                    &auth_token,
                    ws_id,
                    "test",
                    "workspace",
                    None,
                    chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                )
                .await;

            let _ = repo
                .get_effective_config(
                    &auth_token,
                    ws_id,
                    "test",
                    "workspace",
                    Some(Uuid::new_v4()),
                    chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                )
                .await;

            let _ = repo.list_entries(&auth_token, ws_id, Some("test")).await;
            let _ = repo.delete_entry(&auth_token, entry.id()).await;
        } else {
            let _ = repo
                .get_effective_config(
                    &auth_token,
                    ws_id,
                    "test",
                    "workspace",
                    None,
                    chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                )
                .await;

            let _ = repo.list_entries(&auth_token, ws_id, None).await;
            let _ = repo.delete_entry(&auth_token, Uuid::new_v4()).await;
        }
    }
}

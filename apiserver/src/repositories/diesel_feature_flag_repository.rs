use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::diesel_pool::{DieselPools, TenantAuthToken};
use crate::errors::AppError;
use crate::models::diesel::feature_flag::FeatureFlagEntity;
use crate::models::feature_flag::FeatureFlag;
use crate::repositories::feature_flag_repository::FeatureFlagRepository;
use crate::schema::feature_flags;

pub struct DieselFeatureFlagRepository {
    pub diesel_pools: DieselPools,
}

impl DieselFeatureFlagRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl FeatureFlagRepository for DieselFeatureFlagRepository {
    async fn list_flags(&self, auth_token: &TenantAuthToken) -> Result<Vec<FeatureFlag>, AppError> {
        let mut conn = self.diesel_pools.tenant_conn(auth_token).await?;
        let workspace_id = auth_token.workspace_id();

        feature_flags::table
            .filter(feature_flags::workspace_id.eq(workspace_id))
            .order(feature_flags::flag_key.asc())
            .select(FeatureFlagEntity::as_select())
            .load::<FeatureFlagEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))
            .map(|rows| rows.into_iter().map(Into::into).collect())
    }

    async fn get_flag_by_key(
        &self,
        auth_token: &TenantAuthToken,
        flag_key: &str,
    ) -> Result<Option<FeatureFlag>, AppError> {
        let mut conn = self.diesel_pools.tenant_conn(auth_token).await?;
        let workspace_id = auth_token.workspace_id();

        feature_flags::table
            .filter(feature_flags::workspace_id.eq(workspace_id))
            .filter(feature_flags::flag_key.eq(flag_key))
            .select(FeatureFlagEntity::as_select())
            .first::<FeatureFlagEntity>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))
            .map(|r| r.map(Into::into))
    }

    async fn get_flag_by_id(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<FeatureFlag>, AppError> {
        let mut conn = self.diesel_pools.tenant_conn(auth_token).await?;
        let workspace_id = auth_token.workspace_id();

        feature_flags::table
            .filter(feature_flags::workspace_id.eq(workspace_id))
            .filter(feature_flags::id.eq(id))
            .select(FeatureFlagEntity::as_select())
            .first::<FeatureFlagEntity>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))
            .map(|r| r.map(Into::into))
    }

    async fn is_enabled_flag(&self, auth_token: &TenantAuthToken, flag_key: &str) -> Result<bool, AppError> {
        let mut conn = self.diesel_pools.tenant_conn(auth_token).await?;
        let workspace_id = auth_token.workspace_id();

        let enabled = feature_flags::table
            .filter(feature_flags::workspace_id.eq(workspace_id))
            .filter(feature_flags::flag_key.eq(flag_key))
            .select(feature_flags::enabled)
            .first::<bool>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(enabled.unwrap_or(false))
    }

    async fn create_flag(
        &self,
        auth_token: &TenantAuthToken,
        flag_key: &str,
        enabled: bool,
        description: Option<&str>,
        metadata: &serde_json::Value,
    ) -> Result<FeatureFlag, AppError> {
        let mut conn = self.diesel_pools.tenant_conn(auth_token).await?;
        let workspace_id = auth_token.workspace_id();
        let now = Utc::now();

        let entity = FeatureFlagEntity {
            id: Uuid::new_v4(),
            workspace_id,
            flag_key: flag_key.to_string(),
            enabled,
            description: description.map(String::from),
            metadata: metadata.clone(),
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(feature_flags::table)
            .values(&entity)
            .returning(FeatureFlagEntity::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))
            .map(Into::into)
    }

    async fn update_flag(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        enabled: Option<bool>,
        description: Option<&str>,
        metadata: Option<&serde_json::Value>,
    ) -> Result<Option<FeatureFlag>, AppError> {
        let mut conn = self.diesel_pools.tenant_conn(auth_token).await?;
        let workspace_id = auth_token.workspace_id();

        let target = feature_flags::table
            .filter(feature_flags::workspace_id.eq(workspace_id))
            .filter(feature_flags::id.eq(id));

        diesel::update(target)
            .set((
                enabled.map(|v| feature_flags::enabled.eq(v)),
                description.map(|s| feature_flags::description.eq(s.to_string())),
                metadata.map(|v| feature_flags::metadata.eq(v.clone())),
                feature_flags::updated_at.eq(Utc::now()),
            ))
            .returning(FeatureFlagEntity::as_returning())
            .get_result(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))
            .map(|r| r.map(Into::into))
    }

    async fn delete_flag(&self, auth_token: &TenantAuthToken, id: Uuid) -> Result<bool, AppError> {
        let mut conn = self.diesel_pools.tenant_conn(auth_token).await?;
        let workspace_id = auth_token.workspace_id();

        diesel::delete(
            feature_flags::table
                .filter(feature_flags::workspace_id.eq(workspace_id))
                .filter(feature_flags::id.eq(id)),
        )
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))
        .map(|count| count > 0)
    }

    async fn upsert_flag(
        &self,
        auth_token: &TenantAuthToken,
        flag_key: &str,
        enabled: bool,
        description: Option<&str>,
        metadata: &serde_json::Value,
    ) -> Result<FeatureFlag, AppError> {
        let mut conn = self.diesel_pools.tenant_conn(auth_token).await?;
        let workspace_id = auth_token.workspace_id();
        let now = Utc::now();

        let entity = FeatureFlagEntity {
            id: Uuid::new_v4(),
            workspace_id,
            flag_key: flag_key.to_string(),
            enabled,
            description: description.map(String::from),
            metadata: metadata.clone(),
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(feature_flags::table)
            .values(&entity)
            .on_conflict((feature_flags::workspace_id, feature_flags::flag_key))
            .do_update()
            .set((
                feature_flags::enabled.eq(enabled),
                feature_flags::description.eq(description.map(String::from)),
                feature_flags::metadata.eq(metadata.clone()),
                feature_flags::updated_at.eq(Utc::now()),
            ))
            .returning(FeatureFlagEntity::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))
            .map(Into::into)
    }
}

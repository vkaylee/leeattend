use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::feature_flag::FeatureFlag;

// ---------------------------------------------------------------------------
// Trait: FeatureFlagRepositoryTrait — DIP abstraction
// ---------------------------------------------------------------------------
#[async_trait]
pub trait FeatureFlagRepositoryTrait: super::PoolBacked + Send + Sync {
    async fn list_flags(&self, workspace_id: Uuid) -> Result<Vec<FeatureFlag>, AppError>;
    async fn get_flag_by_key(
        &self,
        workspace_id: Uuid,
        flag_key: &str,
    ) -> Result<Option<FeatureFlag>, AppError>;
    async fn get_flag_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<FeatureFlag>, AppError>;
    async fn is_enabled_flag(
        &self,
        workspace_id: Uuid,
        flag_key: &str,
    ) -> Result<bool, AppError>;
    async fn create_flag(
        &self,
        workspace_id: Uuid,
        flag_key: &str,
        enabled: bool,
        description: Option<&str>,
        metadata: &serde_json::Value,
    ) -> Result<FeatureFlag, AppError>;
    async fn update_flag(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        enabled: Option<bool>,
        description: Option<&str>,
        metadata: Option<&serde_json::Value>,
    ) -> Result<Option<FeatureFlag>, AppError>;
    async fn delete_flag(&self, workspace_id: Uuid, id: Uuid) -> Result<bool, AppError>;
    async fn upsert_flag(
        &self,
        workspace_id: Uuid,
        flag_key: &str,
        enabled: bool,
        description: Option<&str>,
        metadata: &serde_json::Value,
    ) -> Result<FeatureFlag, AppError>;
}

/// Direct-query repository for feature flags (mirrors HolidayRepository pattern).
pub struct FeatureFlagRepository;

impl FeatureFlagRepository {
    /// List all flags for a workspace.
    pub async fn list(pool: &PgPool, workspace_id: Uuid) -> sqlx::Result<Vec<FeatureFlag>> {
        sqlx::query_as::<_, FeatureFlag>(
            r#"SELECT id, workspace_id, flag_key, enabled, description, metadata,
                      created_at, updated_at
               FROM feature_flags
              WHERE workspace_id = $1
              ORDER BY flag_key"#,
        )
        .bind(workspace_id)
        .fetch_all(pool)
        .await
    }

    /// Get a single flag by workspace + key.
    pub async fn get_by_key(
        pool: &PgPool,
        workspace_id: Uuid,
        flag_key: &str,
    ) -> sqlx::Result<Option<FeatureFlag>> {
        sqlx::query_as::<_, FeatureFlag>(
            r#"SELECT id, workspace_id, flag_key, enabled, description, metadata,
                      created_at, updated_at
               FROM feature_flags
              WHERE workspace_id = $1 AND flag_key = $2"#,
        )
        .bind(workspace_id)
        .bind(flag_key)
        .fetch_optional(pool)
        .await
    }

    /// Get a single flag by id.
    pub async fn get_by_id(
        pool: &PgPool,
        workspace_id: Uuid,
        id: Uuid,
    ) -> sqlx::Result<Option<FeatureFlag>> {
        sqlx::query_as::<_, FeatureFlag>(
            r#"SELECT id, workspace_id, flag_key, enabled, description, metadata,
                      created_at, updated_at
               FROM feature_flags
              WHERE workspace_id = $1 AND id = $2"#,
        )
        .bind(workspace_id)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// Check whether a flag is enabled (returns false if the flag doesn't exist).
    pub async fn is_enabled(
        pool: &PgPool,
        workspace_id: Uuid,
        flag_key: &str,
    ) -> sqlx::Result<bool> {
        let row: Option<(bool,)> = sqlx::query_as(
            "SELECT enabled FROM feature_flags WHERE workspace_id = $1 AND flag_key = $2",
        )
        .bind(workspace_id)
        .bind(flag_key)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| r.0).unwrap_or(false))
    }

    /// Create a new flag. Returns the created row.
    pub async fn create(
        pool: &PgPool,
        workspace_id: Uuid,
        flag_key: &str,
        enabled: bool,
        description: Option<&str>,
        metadata: &serde_json::Value,
    ) -> sqlx::Result<FeatureFlag> {
        sqlx::query_as::<_, FeatureFlag>(
            r#"INSERT INTO feature_flags (workspace_id, flag_key, enabled, description, metadata)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING id, workspace_id, flag_key, enabled, description, metadata,
                         created_at, updated_at"#,
        )
        .bind(workspace_id)
        .bind(flag_key)
        .bind(enabled)
        .bind(description)
        .bind(metadata)
        .fetch_one(pool)
        .await
    }

    /// Partial update. Only provided fields are changed.
    pub async fn update(
        pool: &PgPool,
        workspace_id: Uuid,
        id: Uuid,
        enabled: Option<bool>,
        description: Option<&str>,
        metadata: Option<&serde_json::Value>,
    ) -> sqlx::Result<Option<FeatureFlag>> {
        sqlx::query_as::<_, FeatureFlag>(
            r#"UPDATE feature_flags
               SET enabled     = COALESCE($3, enabled),
                   description = COALESCE($4, description),
                   metadata    = COALESCE($5, metadata),
                   updated_at  = now()
             WHERE workspace_id = $1 AND id = $2
             RETURNING id, workspace_id, flag_key, enabled, description, metadata,
                       created_at, updated_at"#,
        )
        .bind(workspace_id)
        .bind(id)
        .bind(enabled)
        .bind(description)
        .bind(metadata)
        .fetch_optional(pool)
        .await
    }

    /// Delete a flag. Returns true if a row was actually deleted.
    pub async fn delete(pool: &PgPool, workspace_id: Uuid, id: Uuid) -> sqlx::Result<bool> {
        let result = sqlx::query("DELETE FROM feature_flags WHERE workspace_id = $1 AND id = $2")
            .bind(workspace_id)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Upsert: create-or-update by (workspace_id, flag_key).
    pub async fn upsert(
        pool: &PgPool,
        workspace_id: Uuid,
        flag_key: &str,
        enabled: bool,
        description: Option<&str>,
        metadata: &serde_json::Value,
    ) -> sqlx::Result<FeatureFlag> {
        sqlx::query_as::<_, FeatureFlag>(
            r#"INSERT INTO feature_flags (workspace_id, flag_key, enabled, description, metadata)
               VALUES ($1, $2, $3, $4, $5)
               ON CONFLICT (workspace_id, flag_key)
               DO UPDATE SET enabled     = EXCLUDED.enabled,
                             description = EXCLUDED.description,
                             metadata    = EXCLUDED.metadata,
                             updated_at  = now()
               RETURNING id, workspace_id, flag_key, enabled, description, metadata,
                         created_at, updated_at"#,
        )
        .bind(workspace_id)
        .bind(flag_key)
        .bind(enabled)
        .bind(description)
        .bind(metadata)
        .fetch_one(pool)
        .await
    }
}

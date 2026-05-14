use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::feature_flag::FeatureFlag;

// ---------------------------------------------------------------------------
// Trait: FeatureFlagRepositoryTrait — DIP abstraction
// ---------------------------------------------------------------------------
#[async_trait]
pub trait FeatureFlagRepository: Send + Sync {
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
    async fn is_enabled_flag(&self, workspace_id: Uuid, flag_key: &str) -> Result<bool, AppError>;
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

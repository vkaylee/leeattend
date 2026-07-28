use async_trait::async_trait;
use uuid::Uuid;

use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use crate::models::feature_flag::FeatureFlag;

// ---------------------------------------------------------------------------
// Trait: FeatureFlagRepositoryTrait — DIP abstraction
// ---------------------------------------------------------------------------
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait FeatureFlagRepository: Send + Sync {
    async fn list_flags(&self, auth_token: &TenantAuthToken) -> Result<Vec<FeatureFlag>, AppError>;
    async fn get_flag_by_key(
        &self,
        auth_token: &TenantAuthToken,
        flag_key: &str,
    ) -> Result<Option<FeatureFlag>, AppError>;
    async fn get_flag_by_id(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<FeatureFlag>, AppError>;
    async fn is_enabled_flag(
        &self,
        auth_token: &TenantAuthToken,
        flag_key: &str,
    ) -> Result<bool, AppError>;
    async fn create_flag<'a>(
        &self,
        auth_token: &TenantAuthToken,
        flag_key: &'a str,
        enabled: bool,
        description: Option<&'a str>,
        metadata: &'a serde_json::Value,
    ) -> Result<FeatureFlag, AppError>;
    async fn update_flag<'a>(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        enabled: Option<bool>,
        description: Option<&'a str>,
        metadata: Option<&'a serde_json::Value>,
    ) -> Result<Option<FeatureFlag>, AppError>;
    async fn delete_flag(&self, auth_token: &TenantAuthToken, id: Uuid) -> Result<bool, AppError>;
    async fn upsert_flag<'a>(
        &self,
        auth_token: &TenantAuthToken,
        flag_key: &'a str,
        enabled: bool,
        description: Option<&'a str>,
        metadata: &'a serde_json::Value,
    ) -> Result<FeatureFlag, AppError>;
}

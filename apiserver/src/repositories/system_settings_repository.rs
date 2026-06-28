use crate::errors::AppError;
use async_trait::async_trait;
use serde_json::Value as JsonValue;

#[async_trait]
pub trait SystemSettingsRepository: Send + Sync {
    /// Read a system setting by key. Returns `None` if the key does not exist.
    async fn get(&self, key: &str) -> Result<Option<JsonValue>, AppError>;

    /// Upsert a system setting. Creates or replaces the value for the given key.
    async fn set(&self, key: &str, value: &JsonValue) -> Result<(), AppError>;
}

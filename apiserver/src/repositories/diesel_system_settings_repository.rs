use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::system_settings::NewSystemSetting;
use crate::repositories::system_settings_repository::SystemSettingsRepository;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde_json::Value as JsonValue;

pub struct DieselSystemSettingsRepository {
    pool: DieselPools,
}

impl DieselSystemSettingsRepository {
    pub fn new(pool: DieselPools) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SystemSettingsRepository for DieselSystemSettingsRepository {
    async fn get(&self, key: &str) -> Result<Option<JsonValue>, AppError> {
        use crate::schema::system_settings::dsl;
        let mut conn = crate::diesel_pool::TransactionConn(
            self.pool
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("DB pool error: {e}")))?,
        );

        let result: Option<crate::models::system_settings::SystemSetting> = dsl::system_settings
            .filter(dsl::key.eq(key))
            .first::<crate::models::system_settings::SystemSetting>(&mut *conn)
            .await
            .optional()
            .map_err(|e| {
                AppError::Internal(format!("Failed to read system setting '{key}': {e}"))
            })?;

        Ok(result.map(|r| r.value_json))
    }

    async fn set(&self, key: &str, value: &JsonValue) -> Result<(), AppError> {
        use crate::schema::system_settings::dsl;
        let mut conn = crate::diesel_pool::TransactionConn(
            self.pool
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("DB pool error: {e}")))?,
        );

        let new_val = NewSystemSetting {
            key: key.to_string(),
            value_json: value.clone(),
        };

        diesel::insert_into(dsl::system_settings)
            .values(&new_val)
            .on_conflict(dsl::key)
            .do_update()
            .set(dsl::value_json.eq(value.clone()))
            .execute(&mut *conn)
            .await
            .map_err(|e| {
                AppError::Internal(format!("Failed to upsert system setting '{key}': {e}"))
            })?;

        Ok(())
    }
}

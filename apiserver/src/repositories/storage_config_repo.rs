use crate::crypto::CryptoService;
use crate::diesel_pool::DieselPool;
use crate::errors::{AppError, AppResult};
use crate::models::storage_config::StorageConfig;
use crate::schema::storage_configs;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct StorageConfigRepository {
    pool: DieselPool,
    crypto: Arc<CryptoService>,
}

impl StorageConfigRepository {
    pub fn new(pool: DieselPool, crypto: Arc<CryptoService>) -> Self {
        Self { pool, crypto }
    }

    /// Gets active storage config for a specific workspace.
    /// Automatically decrypts the credentials.
    pub async fn get_active_config(
        &self,
        workspace_id: Uuid,
    ) -> AppResult<Option<crate::models::storage_config::StorageConfig>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let mut config: Option<StorageConfig> = storage_configs::table
            .filter(storage_configs::workspace_id.eq(workspace_id))
            .filter(storage_configs::is_active.eq(true))
            .first(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if let Some(ref mut c) = config {
            c.access_key_enc = self
                .crypto
                .decrypt(&c.access_key_enc)
                .map_err(|e| AppError::Internal(format!("Failed to decrypt access_key: {e}")))?;
            c.secret_key_enc = self
                .crypto
                .decrypt(&c.secret_key_enc)
                .map_err(|e| AppError::Internal(format!("Failed to decrypt secret_key: {e}")))?;
        }

        Ok(config)
    }

    /// Creates or updates the storage config.
    /// Automatically encrypts the credentials before saving.
    pub async fn upsert_config(
        &self,
        workspace_id: Uuid,
        endpoint_url: String,
        region: String,
        bucket_name: String,
        path_prefix: Option<String>,
        access_key: String,
        secret_key: String,
        use_path_style: bool,
    ) -> AppResult<StorageConfig> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let enc_access = self
            .crypto
            .encrypt(&access_key)
            .map_err(|e| AppError::Internal(format!("Failed to encrypt access_key: {e}")))?;
        let enc_secret = self
            .crypto
            .encrypt(&secret_key)
            .map_err(|e| AppError::Internal(format!("Failed to encrypt secret_key: {e}")))?;

        // Extract key version from the format 'version:nonce:ciphertext'
        let key_version = enc_access.split(':').next().unwrap_or("").to_string();

        let new_config = StorageConfig {
            id: crate::utils::id::new_id(),
            workspace_id,
            endpoint_url,
            region,
            bucket_name,
            path_prefix,
            access_key_enc: enc_access,
            secret_key_enc: enc_secret,
            encryption_key_version: key_version,
            use_path_style,
            presigned_ttl_secs: 900,         // Default 15 mins
            max_file_size: 10 * 1024 * 1024, // Default 10MB
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let result = diesel::insert_into(storage_configs::table)
            .values(&new_config)
            .on_conflict(storage_configs::workspace_id)
            .do_update()
            .set((
                storage_configs::endpoint_url.eq(&new_config.endpoint_url),
                storage_configs::region.eq(&new_config.region),
                storage_configs::bucket_name.eq(&new_config.bucket_name),
                storage_configs::path_prefix.eq(&new_config.path_prefix),
                storage_configs::access_key_enc.eq(&new_config.access_key_enc),
                storage_configs::secret_key_enc.eq(&new_config.secret_key_enc),
                storage_configs::encryption_key_version.eq(&new_config.encryption_key_version),
                storage_configs::use_path_style.eq(&new_config.use_path_style),
                storage_configs::is_active.eq(true),
                storage_configs::updated_at.eq(chrono::Utc::now()),
            ))
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to insert storage config: {e}")))?;

        Ok(result)
    }

    /// Deactivates the custom storage config for a workspace, falling back to system defaults.
    pub async fn deactivate_config(&self, workspace_id: Uuid) -> AppResult<bool> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let rows = diesel::update(
            storage_configs::table.filter(storage_configs::workspace_id.eq(workspace_id)),
        )
        .set((
            storage_configs::is_active.eq(false),
            storage_configs::updated_at.eq(chrono::Utc::now()),
        ))
        .execute(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to update storage config: {e}")))?;

        Ok(rows > 0)
    }
}

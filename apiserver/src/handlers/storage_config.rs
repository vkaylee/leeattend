use axum::{
    extract::{Path, State},
    Json,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    handlers::common::MemberContext,
    models::storage_config::StorageConfig,
    schema::storage_configs,
    services::storage::{
        s3_storage::{S3Config, S3Storage},
        ObjectStorage,
    },
    state::AppState,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateStorageConfigRequest {
    pub endpoint_url: String,
    pub region: String,
    pub bucket_name: String,
    pub path_prefix: Option<String>,
    pub access_key: String,
    pub secret_key: String,
    pub use_path_style: bool,
    pub presigned_ttl_secs: i32,
    pub max_file_size: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateStorageConfigRequest {
    pub endpoint_url: Option<String>,
    pub region: Option<String>,
    pub bucket_name: Option<String>,
    pub path_prefix: Option<Option<String>>,
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
    pub use_path_style: Option<bool>,
    pub presigned_ttl_secs: Option<i32>,
    pub max_file_size: Option<i64>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StorageConfigResponse {
    pub id: Uuid,
    pub endpoint_url: String,
    pub region: String,
    pub bucket_name: String,
    pub path_prefix: Option<String>,
    pub access_key_masked: String,
    pub use_path_style: bool,
    pub is_active: bool,
}

impl From<StorageConfig> for StorageConfigResponse {
    fn from(config: StorageConfig) -> Self {
        Self {
            id: config.id,
            endpoint_url: config.endpoint_url,
            region: config.region,
            bucket_name: config.bucket_name,
            path_prefix: config.path_prefix,
            access_key_masked: "***".to_string(), // Don't return the encrypted key directly for extra safety
            use_path_style: config.use_path_style,
            is_active: config.is_active,
        }
    }
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/settings/storage",
    tag = "storage_config",
    responses(
        (status = 200, description = "Returns storage config", body = StorageConfigResponse)
    ),
    security(("bearerAuth" = []))
)]
pub async fn get_storage_config(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(_ws_id): Path<Uuid>,
) -> AppResult<Json<Option<StorageConfigResponse>>> {
    let workspace_id = ctx.workspace_id;
    let mut conn = state
        .infra()
        .diesel()
        .api
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let config: Option<StorageConfig> = storage_configs::table
        .filter(storage_configs::workspace_id.eq(workspace_id))
        .first::<StorageConfig>(&mut *conn)
        .await
        .optional()
        .map_err(|e| AppError::Internal(format!("Failed to fetch storage config: {e}")))?;

    Ok(Json(config.map(StorageConfigResponse::from)))
}

#[utoipa::path(
    put,
    path = "/v1/workspaces/{workspace_id}/settings/storage",
    tag = "storage_config",
    request_body = CreateStorageConfigRequest,
    responses(
        (status = 200, description = "Config created or updated", body = StorageConfigResponse)
    ),
    security(("bearerAuth" = []))
)]
pub async fn upsert_storage_config(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(_ws_id): Path<Uuid>,
    Json(payload): Json<CreateStorageConfigRequest>,
) -> AppResult<Json<StorageConfigResponse>> {
    let workspace_id = ctx.workspace_id;
    let mut conn = state
        .infra()
        .diesel()
        .api
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let crypto = state.infra().crypto_service().ok_or_else(|| {
        tracing::error!("Crypto service not configured");
        AppError::Internal("Server configuration error".into())
    })?;

    let access_key_enc = crypto.encrypt(&payload.access_key).map_err(|e| {
        tracing::error!("Encryption failed: {}", e);
        AppError::Internal("Failed to encrypt credentials".into())
    })?;

    let secret_key_enc = crypto.encrypt(&payload.secret_key).map_err(|e| {
        tracing::error!("Encryption failed: {}", e);
        AppError::Internal("Failed to encrypt credentials".into())
    })?;

    // Extract active version from CryptoService
    // Since we don't have direct access to active version in CryptoService easily via public API,
    // we use a convention or we just extract it from the encrypted string `version:nonce:ciphertext`
    let encryption_key_version = access_key_enc.split(':').next().unwrap_or("v1").to_string();

    let new_config = StorageConfig {
        id: Uuid::new_v4(),
        workspace_id,
        endpoint_url: payload.endpoint_url,
        region: payload.region,
        bucket_name: payload.bucket_name,
        path_prefix: payload.path_prefix,
        access_key_enc,
        secret_key_enc,
        encryption_key_version,
        use_path_style: payload.use_path_style,
        presigned_ttl_secs: payload.presigned_ttl_secs,
        max_file_size: payload.max_file_size,
        is_active: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let saved_config = diesel::insert_into(storage_configs::table)
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
            storage_configs::presigned_ttl_secs.eq(&new_config.presigned_ttl_secs),
            storage_configs::max_file_size.eq(&new_config.max_file_size),
            storage_configs::updated_at.eq(chrono::Utc::now()),
        ))
        .get_result::<StorageConfig>(&mut *conn)
        .await
        .map_err(|e| {
            tracing::error!("Failed to upsert storage config: {}", e);
            AppError::Internal("Failed to upsert storage config".into())
        })?;

    drop(conn);
    // Invalidate cache if StorageResolver supports it
    state.storage().invalidate(workspace_id).await;

    Ok(Json(StorageConfigResponse::from(saved_config)))
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/settings/storage/test",
    tag = "storage_config",
    request_body = CreateStorageConfigRequest,
    responses(
        (status = 200, description = "Test result", body = bool)
    ),
    security(("bearerAuth" = []))
)]
pub async fn test_storage_config(
    State(_state): State<Arc<AppState>>,
    _ctx: MemberContext,
    Path(_ws_id): Path<Uuid>,
    Json(payload): Json<CreateStorageConfigRequest>,
) -> AppResult<Json<bool>> {
    let s3_config = S3Config {
        endpoint_url: payload.endpoint_url,
        region: payload.region,
        bucket_name: payload.bucket_name,
        access_key: payload.access_key,
        secret_key: payload.secret_key,
        use_path_style: payload.use_path_style,
    };

    let s3_storage = S3Storage::new(s3_config)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create S3 storage: {e}")))?;

    // Attempt a health check (e.g., list objects with max_keys=1)
    if let Err(e) = s3_storage.health_check().await {
        tracing::error!("Storage config test failed: {}", e);
        Err(AppError::Internal(
            "Failed to connect to S3 with provided credentials".into(),
        ))
    } else {
        Ok(Json(true))
    }
}

#[utoipa::path(
    delete,
    path = "/v1/workspaces/{workspace_id}/settings/storage",
    tag = "storage_config",
    responses(
        (status = 200, description = "Config deleted")
    ),
    security(("bearerAuth" = []))
)]
pub async fn delete_storage_config(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(_ws_id): Path<Uuid>,
) -> AppResult<Json<()>> {
    let workspace_id = ctx.workspace_id;
    let mut conn = state
        .infra()
        .diesel()
        .api
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    diesel::delete(storage_configs::table.filter(storage_configs::workspace_id.eq(workspace_id)))
        .execute(&mut *conn)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete storage config: {}", e);
            AppError::Internal("Failed to update storage config".into())
        })?;

    drop(conn);
    // Invalidate cache
    state.storage().invalidate(workspace_id).await;

    Ok(Json(()))
}

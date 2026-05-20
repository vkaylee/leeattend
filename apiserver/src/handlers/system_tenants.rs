//! System-level Tenant Management endpoints.
//!
//! All routes here require `SystemContext` with `PlatformAdmin` role.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;

use crate::errors::{AppError, AppResult};
use crate::handlers::common::ApiContext;
use crate::handlers::system_auth::SystemContext;
use crate::models::enums::SystemRoleType;
use crate::supavisor::build_tenant_payload;
use crate::AppState;
use uuid::Uuid;

use diesel_async::RunQueryDsl;

#[derive(Deserialize, ToSchema)]
pub struct CreateTenantRequest {
    pub workspace_id: Uuid,
    /// If provided, this URL is used as the tenant database directly (BYOD).
    /// If not provided, the system will provision a new database on the master cluster.
    pub custom_db_url: Option<String>,
    pub pool_size: Option<u32>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateTenantDbRequest {
    pub custom_db_url: String,
}

fn mask_db_url(url: &str) -> String {
    if let Ok(mut parsed) = reqwest::Url::parse(url) {
        let _ = parsed.set_password(Some("***"));
        parsed.to_string()
    } else {
        "*** REDACTED ***".to_string()
    }
}


impl std::fmt::Debug for CreateTenantRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CreateTenantRequest")
            .field("workspace_id", &self.workspace_id)
            .field(
                "custom_db_url",
                &if self.custom_db_url.is_some() {
                    "Some(***REDACTED***)"
                } else {
                    "None"
                },
            )
            .field("pool_size", &self.pool_size)
            .finish()
    }
}

/// GET /system/tenants
/// Retrieves the list of all tenants registered in Supavisor.
pub async fn list_tenants(
    State(_state): State<Arc<AppState>>,
    ctx: SystemContext,
    _api_ctx: ApiContext,
) -> AppResult<Json<serde_json::Value>> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    let mut conn = _state
        .infra
        .diesel
        .api
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Query Supavisor schema directly to list tenants
    // If the schema doesn't exist (e.g. pooler not enabled), return empty array
    #[derive(Debug, diesel::QueryableByName)]
    struct TenantRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        external_id: String,
        #[diesel(sql_type = diesel::sql_types::Integer)]
        default_pool_size: i32,
        #[diesel(sql_type = diesel::sql_types::Timestamp)]
        inserted_at: chrono::NaiveDateTime,
        #[diesel(sql_type = diesel::sql_types::Timestamp)]
        updated_at: chrono::NaiveDateTime,
    }

    let tenants_result = diesel::sql_query(
        r#"
        SELECT external_id, default_pool_size, inserted_at, updated_at 
        FROM _supavisor.tenants
        ORDER BY inserted_at DESC
        "#,
    )
    .load::<TenantRow>(&mut conn)
    .await;

    let tenants = match tenants_result {
        Ok(records) => records
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.external_id,
                    "pool_size": r.default_pool_size,
                    "created_at": r.inserted_at,
                    "updated_at": r.updated_at
                })
            })
            .collect::<Vec<_>>(),
        Err(e) => {
            tracing::warn!("Failed to fetch tenants from _supavisor.tenants: {}", e);
            vec![]
        }
    };

    Ok(Json(serde_json::json!(tenants)))
}

/// POST /system/tenants
/// Creates a new tenant (and optionally a physical DB) and registers it in Supavisor.
pub async fn create_tenant(
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    _api_ctx: ApiContext,
    Json(req): Json<CreateTenantRequest>,
) -> AppResult<StatusCode> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    // 1. Provision the database (or use BYOD) and run migrations
    let provisioner = std::sync::Arc::clone(&state.infra.tenant_provisioner);
    let workspace_id = req.workspace_id;
    let new_db_url = provisioner
        .provision_tenant_db(&workspace_id, req.custom_db_url.clone())
        .await?;

    // 2. Build dynamic Supavisor payload.
    // Tenant ID must match the format produced by diesel_pool::get_tenant_url():
    // tenant_<uuid-without-hyphens>. Using the raw UUID would cause a mismatch
    // when the API server connects through Supavisor.
    let tenant_id_alias =
        format!("tenant_{}", req.workspace_id.to_string().replace('-', ""));
    if let Some(size) = req.pool_size {
        std::env::set_var("SUPAVISOR_POOL_SIZE", size.to_string());
    }
    std::env::set_var("SUPAVISOR_TENANT_ID", &tenant_id_alias);

    let payload_opt = build_tenant_payload(&new_db_url);

    std::env::remove_var("SUPAVISOR_TENANT_ID");
    if req.pool_size.is_some() {
        std::env::remove_var("SUPAVISOR_POOL_SIZE");
    }

    let Some((tenant_id, payload)) = payload_opt else {
        return Err(AppError::Internal(
            "Failed to build Supavisor payload".into(),
        ));
    };

    // 3. Register in Supavisor
    state
        .infra
        .tenant_manager
        .register_tenant_dynamic(&tenant_id, &payload)
        .await?;

    // 4. Encrypt and save custom_db_url if provided
    if let Some(raw_url) = &req.custom_db_url {
        let encrypted_url = if let Some(crypto) = &state.infra.crypto_service {
            crypto.encrypt(raw_url).map_err(|e| {
                tracing::error!("Failed to encrypt DB URL: {}", e);
                AppError::Internal("Encryption failed".into())
            })?
        } else {
            tracing::warn!("No crypto service available, storing URL in plaintext!");
            raw_url.clone()
        };

        state
            .core
            .workspace_repo
            .update_workspace_byod_url(workspace_id, Some(encrypted_url))
            .await?;
    }

    // 4. Audit Log
    let details = serde_json::json!({
        "workspace_id": req.workspace_id,
        "is_byod": req.custom_db_url.is_some(),
        "pool_size": req.pool_size
    });

    let mut conn = state
        .infra
        .diesel
        .api
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    diesel::sql_query(
        r#"
        INSERT INTO system_audit_logs (id, actor_id, action, details, created_at)
        VALUES ($1, $2, 'tenant.create', $3, NOW())
        "#,
    )
    .bind::<diesel::sql_types::Uuid, _>(crate::utils::id::new_id())
    .bind::<diesel::sql_types::Uuid, _>(ctx.user_id)
    .bind::<diesel::sql_types::Jsonb, _>(details)
    .execute(&mut conn)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(StatusCode::CREATED)
}

/// DELETE /system/tenants/:tenant_id
/// Removes a tenant from Supavisor connection pool.
pub async fn delete_tenant_handler(
    Path(tenant_id): Path<String>,
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    _api_ctx: ApiContext,
) -> AppResult<StatusCode> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    let mut conn = state
        .infra
        .diesel
        .api
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // 1. Delete from Supavisor
    state.infra.tenant_manager.delete_tenant(&tenant_id).await?;

    // 2. Audit Log
    let details = serde_json::json!({
        "tenant_id": tenant_id
    });

    diesel::sql_query(
        r#"
        INSERT INTO system_audit_logs (id, actor_id, action, details, created_at)
        VALUES ($1, $2, 'tenant.delete', $3, NOW())
        "#,
    )
    .bind::<diesel::sql_types::Uuid, _>(crate::utils::id::new_id())
    .bind::<diesel::sql_types::Uuid, _>(ctx.user_id)
    .bind::<diesel::sql_types::Jsonb, _>(details)
    .execute(&mut conn)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

/// PUT /system/tenants/:tenant_id/database
/// Updates a tenant's database connection URL and registers it in Supavisor.
pub async fn update_tenant_database(
    Path(tenant_id): Path<String>,
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    _api_ctx: ApiContext,
    Json(req): Json<UpdateTenantDbRequest>,
) -> AppResult<StatusCode> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    let uuid_str = tenant_id.strip_prefix("tenant_").unwrap_or(&tenant_id);
    let workspace_id = uuid::Uuid::parse_str(uuid_str)
        .map_err(|_| AppError::Validation("Invalid tenant_id format".into()))?;

    // Build dynamic Supavisor payload.
    // Tenant ID must match the format produced by diesel_pool::get_tenant_url()
    // By this time we already have the correct `tenant_id` from the path.
    // We just need to build the payload with the new DB URL.
    
    // We temporarily set SUPAVISOR_TENANT_ID to trick build_tenant_payload to build correct payload, 
    // although build_tenant_payload uses it if we wanted to register it, but it only sets DB connections inside.
    let payload_opt = build_tenant_payload(&req.custom_db_url);

    let Some((_, payload)) = payload_opt else {
        return Err(AppError::Internal(
            "Failed to build Supavisor payload".into(),
        ));
    };

    // Update in Supavisor
    state
        .infra
        .tenant_manager
        .register_tenant_dynamic(&tenant_id, &payload)
        .await?;

    // Encrypt and save custom_db_url
    let encrypted_url = if let Some(crypto) = &state.infra.crypto_service {
        crypto.encrypt(&req.custom_db_url).map_err(|e| {
            tracing::error!("Failed to encrypt DB URL: {}", e);
            AppError::Internal("Encryption failed".into())
        })?
    } else {
        tracing::warn!("No crypto service available, storing URL in plaintext!");
        req.custom_db_url.clone()
    };

    state
        .core
        .workspace_repo
        .update_workspace_byod_url(workspace_id, Some(encrypted_url))
        .await?;

    // Audit Log
    let details = serde_json::json!({
        "workspace_id": workspace_id,
        "is_byod": true,
        "action": "update_database_url"
    });

    let mut conn = state
        .infra
        .diesel
        .api
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    diesel::sql_query(
        r#"
        INSERT INTO system_audit_logs (id, actor_id, action, details, created_at)
        VALUES ($1, $2, 'tenant.update_db', $3, NOW())
        "#,
    )
    .bind::<diesel::sql_types::Uuid, _>(crate::utils::id::new_id())
    .bind::<diesel::sql_types::Uuid, _>(ctx.user_id)
    .bind::<diesel::sql_types::Jsonb, _>(details)
    .execute(&mut conn)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(StatusCode::OK)
}


/// GET /system/tenants/:tenant_id/stats
/// Retrieves tenant configuration and stats from Supavisor API.
pub async fn get_tenant_stats(
    Path(tenant_id): Path<String>,
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    _api_ctx: ApiContext,
) -> AppResult<Json<serde_json::Value>> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    let tenant_info = state.infra.tenant_manager.get_tenant(&tenant_id).await?;

    let uuid_str = tenant_id.strip_prefix("tenant_").unwrap_or(&tenant_id);
    let masked_url = if let Ok(workspace_id) = uuid::Uuid::parse_str(uuid_str) {
        if let Ok(Some(ws)) = state.core.workspace_repo.get_workspace_by_id(workspace_id).await {
            if let Some(encrypted) = ws.custom_db_url_encrypted {
                let decrypted = if let Some(crypto) = &state.infra.crypto_service {
                    crypto.decrypt(&encrypted).unwrap_or(encrypted)
                } else {
                    encrypted
                };
                Some(mask_db_url(&decrypted))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let supavisor_host = std::env::var("SUPAVISOR_HOST").unwrap_or_else(|_| "localhost".to_string());
    let supavisor_port = std::env::var("SUPAVISOR_PORT").unwrap_or_else(|_| "5452".to_string());

    let response = serde_json::json!({
        "config": tenant_info,
        "connection": {
            "host": supavisor_host,
            "port": supavisor_port,
            "masked_db_url": masked_url,
        }
    });

    Ok(Json(response))
}

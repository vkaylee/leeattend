//! System-level Tenant (Workspace) Management endpoints.
//!
//! All routes here require `SystemContext` with `PlatformAdmin` role.
//! "Tenants" in the new architecture are Workspaces — each workspace IS a tenant.
//! BYOD tenants are workspaces with `custom_db_url_encrypted` set.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::handlers::common::ApiContext;
use crate::handlers::system_auth::SystemContext;
use crate::models::enums::SystemRoleType;
use crate::utils::i18n::I18nKey;
use crate::AppState;

// ── Response / Request DTOs ──────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct SystemTenantResponse {
    pub id: Uuid,
    pub name: String,
    pub is_byod: bool,
    pub timezone: String,
    pub created_at: DateTime<Utc>,
}

impl From<crate::models::Workspace> for SystemTenantResponse {
    fn from(ws: crate::models::Workspace) -> Self {
        Self {
            id: ws.id(),
            name: ws.name().to_string(),
            is_byod: ws.custom_db_url_encrypted().is_some(),
            timezone: ws.timezone().to_string(),
            created_at: ws.created_at(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTenantRequest {
    pub name: String,
    pub database_url: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTenantDbRequest {
    pub database_url: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TenantStatsResponse {
    pub id: Uuid,
    pub name: String,
    pub is_byod: bool,
    pub timezone: String,
    pub created_at: DateTime<Utc>,
    pub masked_db_url: Option<String>,
    pub connection: TenantConnectionInfo,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TenantConnectionInfo {
    pub host: String,
    pub port: String,
}

fn mask_db_url(url: &str) -> String {
    if let Ok(mut parsed) = reqwest::Url::parse(url) {
        let _ = parsed.set_password(Some("***"));
        parsed.to_string()
    } else {
        "*** REDACTED ***".to_string()
    }
}

#[allow(clippy::disallowed_methods)]
fn db_host_from_env() -> String {
    std::env::var("PGBOUNCER_HOST")
        .or_else(|_| std::env::var("DATABASE_HOST"))
        .unwrap_or_else(|_| "localhost".to_string())
}

#[allow(clippy::disallowed_methods)]
fn db_port_from_env() -> String {
    std::env::var("PGBOUNCER_PORT")
        .or_else(|_| std::env::var("PGPORT"))
        .unwrap_or_else(|_| "5432".to_string())
}

/// Resolve database host from the centralized env registry.
#[allow(dead_code)]
fn db_host_from_registry(registry: &crate::utils::env_registry::EnvRegistry) -> String {
    registry
        .get("PGBOUNCER_HOST")
        .map(ToString::to_string)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "localhost".to_string())
}

/// Resolve database port from the centralized env registry.
#[allow(dead_code)]
fn db_port_from_registry(registry: &crate::utils::env_registry::EnvRegistry) -> String {
    registry
        .get("PGBOUNCER_PORT")
        .map(ToString::to_string)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "5432".to_string())
}

// ── Handlers ─────────────────────────────────────────────────────────────

/// GET /system/tenants
/// List all tenants (workspaces).
#[utoipa::path(
    get,
    path = "/system/tenants",
    responses(
        (status = 200, description = "List of all tenants", body = [SystemTenantResponse])
    )
)]
pub async fn list_tenants(
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    _api_ctx: ApiContext,
) -> AppResult<Json<Vec<SystemTenantResponse>>> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    let workspaces = state.core().workspace_repo().get_all_workspaces().await?;
    let tenants: Vec<SystemTenantResponse> = workspaces.into_iter().map(Into::into).collect();

    Ok(Json(tenants))
}

/// POST /system/tenants
/// Create a new tenant (workspace). If database_url is provided, sets up BYOD.
/// Otherwise auto-provisions a tenant database.
#[utoipa::path(
    post,
    path = "/system/tenants",
    request_body = CreateTenantRequest,
    responses(
        (status = 201, description = "Tenant created successfully", body = SystemTenantResponse)
    )
)]
pub async fn create_tenant(
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    _api_ctx: ApiContext,
    Json(req): Json<CreateTenantRequest>,
) -> AppResult<(StatusCode, Json<SystemTenantResponse>)> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    let ws_id = crate::utils::id::new_id();

    // Encrypt database_url as custom_db_url_encrypted for BYOD support
    let encrypted_url = if let Some(url) = &req.database_url {
        if let Some(crypto) = state.infra().crypto_service() {
            Some(crypto.encrypt(url).map_err(|e| {
                tracing::error!("Failed to encrypt DB URL: {}", e);
                AppError::Internal("Encryption failed".into())
            })?)
        } else {
            tracing::warn!("No crypto service available, storing URL in plaintext!");
            Some(url.clone())
        }
    } else {
        None
    };

    let ws = state
        .core()
        .workspace_service()
        .create_workspace(
            ws_id,
            &req.name,
            ctx.user_id,
            req.database_url,
            encrypted_url,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(ws.into())))
}

/// DELETE /system/tenants/{tenant_id}
/// Delete a tenant (workspace). Removes the workspace row — does NOT drop
/// the physical tenant database.
#[utoipa::path(
    delete,
    path = "/system/tenants/{tenant_id}",
    params(
        ("tenant_id" = Uuid, Path, description = "Tenant (Workspace) ID")
    ),
    responses(
        (status = 204, description = "Tenant deleted successfully")
    )
)]
pub async fn delete_tenant(
    Path(tenant_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    _api_ctx: ApiContext,
) -> AppResult<StatusCode> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    state
        .core()
        .workspace_repo()
        .delete_workspace(tenant_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /system/tenants/{tenant_id}/stats
/// Get tenant connection details and stats.
#[utoipa::path(
    get,
    path = "/system/tenants/{tenant_id}/stats",
    params(
        ("tenant_id" = Uuid, Path, description = "Tenant (Workspace) ID")
    ),
    responses(
        (status = 200, description = "Tenant connection details", body = TenantStatsResponse)
    )
)]
pub async fn get_tenant_stats(
    Path(tenant_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    _api_ctx: ApiContext,
) -> AppResult<Json<TenantStatsResponse>> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    let ws = state
        .core()
        .workspace_repo()
        .get_workspace_by_id(tenant_id)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::WorkspaceNotFound))?;

    let masked_db_url = ws.custom_db_url_encrypted().and_then(|encrypted| {
        let crypto = state.infra().crypto_service()?;
        let decrypted = crypto.decrypt(encrypted).ok()?;
        Some(mask_db_url(&decrypted))
    });

    Ok(Json(TenantStatsResponse {
        id: ws.id(),
        name: ws.name().to_string(),
        is_byod: ws.custom_db_url_encrypted().is_some(),
        timezone: ws.timezone().to_string(),
        created_at: ws.created_at(),
        masked_db_url,
        connection: TenantConnectionInfo {
            host: db_host_from_env(),
            port: db_port_from_env(),
        },
    }))
}

/// PUT /system/tenants/{tenant_id}/database
/// Update the custom BYOD database URL for a tenant.
/// The system DOES NOT copy data — ensure the target database is migrated.
#[utoipa::path(
    put,
    path = "/system/tenants/{tenant_id}/database",
    params(
        ("tenant_id" = Uuid, Path, description = "Tenant (Workspace) ID")
    ),
    request_body = UpdateTenantDbRequest,
    responses(
        (status = 200, description = "Database URL updated successfully")
    )
)]
pub async fn update_tenant_database(
    Path(tenant_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    _api_ctx: ApiContext,
    Json(req): Json<UpdateTenantDbRequest>,
) -> AppResult<StatusCode> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    let encrypted_url = if let Some(crypto) = state.infra().crypto_service() {
        Some(crypto.encrypt(&req.database_url).map_err(|e| {
            tracing::error!("Failed to encrypt DB URL: {}", e);
            AppError::Internal("Encryption failed".into())
        })?)
    } else {
        tracing::warn!("No crypto service available, storing URL in plaintext!");
        Some(req.database_url)
    };

    state
        .core()
        .workspace_repo()
        .update_workspace_byod_url(tenant_id, encrypted_url)
        .await?;

    Ok(StatusCode::OK)
}

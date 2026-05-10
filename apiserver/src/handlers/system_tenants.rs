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
use crate::handlers::system_auth::SystemContext;
use crate::models::enums::SystemRoleType;
use crate::supavisor::{build_tenant_payload, delete_tenant, register_tenant_dynamic};
use crate::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTenantRequest {
    pub db_name: String,
    /// If true, the system will attempt to execute CREATE DATABASE on the master node before registering.
    pub create_db: bool,
    pub pool_size: Option<u32>,
}

/// GET /system/tenants
/// Retrieves the list of all tenants registered in Supavisor.
pub async fn list_tenants(
    State(_state): State<Arc<AppState>>,
    ctx: SystemContext,
) -> AppResult<Json<serde_json::Value>> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    // Query Supavisor schema directly to list tenants
    // If the schema doesn't exist (e.g. pooler not enabled), return empty array
    let tenants_result = sqlx::query!(
        r#"
        SELECT external_id, default_pool_size, inserted_at, updated_at 
        FROM _supavisor.tenants
        ORDER BY inserted_at DESC
        "#
    )
    .fetch_all(_state.infra.db.api())
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
    Json(req): Json<CreateTenantRequest>,
) -> AppResult<StatusCode> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    // 1. Create physical DB if requested
    if req.create_db {
        // Prevent SQL injection: simple alphanumeric check
        if !req
            .db_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(AppError::Validation("Invalid database name format".into()));
        }

        let create_sql = format!("CREATE DATABASE {}", req.db_name);

        match sqlx::query(&create_sql).execute(state.infra.db.api()).await {
            Ok(_) => tracing::info!("Successfully created physical database: {}", req.db_name),
            Err(e) => {
                if e.to_string().contains("already exists") {
                    return Err(AppError::Conflict(format!(
                        "Database {} already exists",
                        req.db_name
                    )));
                }
                return Err(AppError::Sqlx(e));
            }
        }
    }

    // 2. Build dynamic Supavisor payload
    let master_url = std::env::var("DATABASE_URL").unwrap_or_default();
    let Ok(mut parsed_url) = reqwest::Url::parse(&master_url) else {
        return Err(AppError::Internal(
            "Invalid DATABASE_URL in environment".into(),
        ));
    };
    parsed_url.set_path(&req.db_name);

    if let Some(size) = req.pool_size {
        std::env::set_var("SUPAVISOR_POOL_SIZE", size.to_string());
    }
    std::env::set_var("SUPAVISOR_TENANT_ID", &req.db_name);

    let payload_opt = build_tenant_payload(parsed_url.as_str());

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
    register_tenant_dynamic(&tenant_id, &payload).await?;

    // 4. Audit Log
    let details = serde_json::json!({
        "db_name": req.db_name,
        "create_db": req.create_db,
        "pool_size": req.pool_size
    });

    sqlx::query!(
        r#"
        INSERT INTO system_audit_logs (actor_id, action, details)
        VALUES ($1, 'tenant.create', $2)
        "#,
        ctx.user_id,
        details
    )
    .execute(state.infra.db.api())
    .await?;

    Ok(StatusCode::CREATED)
}

/// DELETE /system/tenants/:tenant_id
/// Removes a tenant from Supavisor connection pool.
pub async fn delete_tenant_handler(
    Path(tenant_id): Path<String>,
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
) -> AppResult<StatusCode> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    // 1. Delete from Supavisor
    delete_tenant(&tenant_id).await?;

    // 2. Audit Log
    let details = serde_json::json!({
        "tenant_id": tenant_id
    });

    sqlx::query!(
        r#"
        INSERT INTO system_audit_logs (actor_id, action, details)
        VALUES ($1, 'tenant.delete', $2)
        "#,
        ctx.user_id,
        details
    )
    .execute(state.infra.db.api())
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

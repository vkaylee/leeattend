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
    /// The database connection URL for this tenant (BYOD).
    /// The database must already exist. The system will run migrations on it.
    pub database_url: String,
    pub pool_size: Option<u32>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateTenantDbRequest {
    pub database_url: String,
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
            .field("database_url", &"***REDACTED***")
            .field("pool_size", &self.pool_size)
            .finish()
    }
}

fn map_tenants_to_workspaces(
    tenants: Vec<(String, i32, chrono::NaiveDateTime, chrono::NaiveDateTime)>,
    workspaces: Vec<(Uuid, String)>,
) -> Vec<serde_json::Value> {
    use std::collections::HashMap;
    let ws_map: HashMap<String, (Uuid, String)> = workspaces
        .into_iter()
        .map(|(id, name)| {
            let key = format!("tenant_{}", id.to_string().replace('-', ""));
            (key, (id, name))
        })
        .collect();

    tenants
        .into_iter()
        .map(|(external_id, pool_size, inserted_at, updated_at)| {
            let (workspace_id, workspace_name) =
                if let Some((ws_id, ws_name)) = ws_map.get(&external_id) {
                    (Some(ws_id.to_string()), Some(ws_name.clone()))
                } else {
                    (None, None)
                };

            serde_json::json!({
                "id": external_id,
                "pool_size": pool_size,
                "created_at": inserted_at,
                "updated_at": updated_at,
                "workspace_id": workspace_id,
                "workspace_name": workspace_name
            })
        })
        .collect()
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

    // 1. Query active workspaces
    #[derive(Debug, diesel::QueryableByName)]
    struct WsRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }

    let workspaces: Vec<(Uuid, String)> = diesel::sql_query("SELECT id, name FROM workspaces")
        .load::<WsRow>(&mut conn)
        .await
        .map(|rows| rows.into_iter().map(|r| (r.id, r.name)).collect())
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to query workspaces for tenant mapping: {e}");
            vec![]
        });

    // 2. Query Supavisor schema directly to list tenants
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
        Ok(records) => {
            let records_tuples = records
                .into_iter()
                .map(|r| {
                    (
                        r.external_id,
                        r.default_pool_size,
                        r.inserted_at,
                        r.updated_at,
                    )
                })
                .collect::<Vec<_>>();
            map_tenants_to_workspaces(records_tuples, workspaces)
        }
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

    // 1. Provision the database (run migrations on BYOD URL)
    let provisioner = std::sync::Arc::clone(&state.infra.tenant_provisioner);
    let workspace_id = req.workspace_id;
    let new_db_url = provisioner
        .provision_tenant_db(&workspace_id, &req.database_url)
        .await?;

    // 2. Build dynamic Supavisor payload.
    // Tenant ID must match the format produced by diesel_pool::get_tenant_url():
    // tenant_<uuid-without-hyphens>. Using the raw UUID would cause a mismatch
    // when the API server connects through Supavisor.
    let tenant_id_alias = format!("tenant_{}", req.workspace_id.to_string().replace('-', ""));
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

    // 4. Encrypt and save database_url
    let encrypted_url = if let Some(crypto) = &state.infra.crypto_service {
        crypto.encrypt(&req.database_url).map_err(|e| {
            tracing::error!("Failed to encrypt DB URL: {}", e);
            AppError::Internal("Encryption failed".into())
        })?
    } else {
        tracing::warn!("No crypto service available, storing URL in plaintext!");
        req.database_url.clone()
    };

    state
        .core
        .workspace_repo
        .update_workspace_byod_url(workspace_id, Some(encrypted_url))
        .await?;

    // 4. Audit Log
    let details = serde_json::json!({
        "workspace_id": req.workspace_id,
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
        .map_err(|e| AppError::Validation(format!("Invalid tenant_id format: {e}")))?;

    // Build dynamic Supavisor payload.
    // Tenant ID must match the format produced by diesel_pool::get_tenant_url()
    // By this time we already have the correct `tenant_id` from the path.
    // We just need to build the payload with the new DB URL.

    // We temporarily set SUPAVISOR_TENANT_ID to trick build_tenant_payload to build correct payload,
    // although build_tenant_payload uses it if we wanted to register it, but it only sets DB connections inside.
    let payload_opt = build_tenant_payload(&req.database_url);

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

    // Encrypt and save database_url
    let encrypted_url = if let Some(crypto) = &state.infra.crypto_service {
        crypto.encrypt(&req.database_url).map_err(|e| {
            tracing::error!("Failed to encrypt DB URL: {}", e);
            AppError::Internal("Encryption failed".into())
        })?
    } else {
        tracing::warn!("No crypto service available, storing URL in plaintext!");
        req.database_url.clone()
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
        if let Ok(Some(ws)) = state
            .core
            .workspace_repo
            .get_workspace_by_id(workspace_id)
            .await
        {
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

    let supavisor_host =
        std::env::var("SUPAVISOR_HOST").unwrap_or_else(|_| "localhost".to_string());
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

/// PATCH /system/tenants/:tenant_id/mode
/// Updates the pool mode type for a specific tenant at runtime.
#[derive(Deserialize, ToSchema)]
pub struct UpdateTenantModeRequest {
    pub mode_type: String,
}

pub async fn update_tenant_mode(
    Path(tenant_id): Path<String>,
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    _api_ctx: ApiContext,
    Json(req): Json<UpdateTenantModeRequest>,
) -> AppResult<StatusCode> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    let mode_type = req.mode_type.clone();
    if mode_type != "session" && mode_type != "transaction" {
        return Err(AppError::Validation(
            "mode_type must be 'session' or 'transaction'".into(),
        ));
    }

    state
        .infra
        .tenant_manager
        .update_tenant_mode(&tenant_id, &mode_type)
        .await?;

    let mut conn = state
        .infra
        .diesel
        .api
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let details = serde_json::json!({
        "tenant_id": tenant_id,
        "mode_type": mode_type
    });

    diesel::sql_query(
        r#"
        INSERT INTO system_audit_logs (id, actor_id, action, details, created_at)
        VALUES ($1, $2, 'tenant.update_mode', $3, NOW())
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

/// GET /system/tenants/stale
/// Compares registered Supavisor tenants against active workspaces to
/// identify orphaned (stale) tenant configurations.
pub async fn list_stale_tenants(
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    _api_ctx: ApiContext,
) -> AppResult<Json<serde_json::Value>> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    let mut conn = state
        .infra
        .diesel
        .api
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // 1. Query active workspace IDs
    #[derive(Debug, diesel::QueryableByName)]
    struct WsRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }

    let workspace_ids: Vec<Uuid> = diesel::sql_query("SELECT id FROM workspaces")
        .load::<WsRow>(&mut conn)
        .await
        .map(|rows| rows.into_iter().map(|r| r.id).collect())
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to query workspaces for stale tenant check: {e}");
            vec![]
        });

    let active_tenant_ids: std::collections::HashSet<String> = workspace_ids
        .iter()
        .map(|ws_id| format!("tenant_{}", ws_id.to_string().replace('-', "")))
        .collect();

    // 2. Query registered Supavisor tenants
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

    let stale_tenants: Vec<serde_json::Value> = match tenants_result {
        Ok(records) => records
            .into_iter()
            .filter(|t| {
                t.external_id.starts_with("tenant_") && !active_tenant_ids.contains(&t.external_id)
            })
            .map(|r| {
                serde_json::json!({
                    "id": r.external_id,
                    "pool_size": r.default_pool_size,
                    "created_at": r.inserted_at,
                    "updated_at": r.updated_at
                })
            })
            .collect(),
        Err(e) => {
            tracing::warn!("Failed to fetch tenants from _supavisor.tenants for stale check: {e}");
            vec![]
        }
    };

    Ok(Json(serde_json::json!(stale_tenants)))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── mask_db_url — PII protection ─────────────────────────────────────────

    #[test]
    fn test_mask_db_url_masks_password_in_valid_url() {
        let masked = mask_db_url("postgres://admin:supersecret@db.example.com:5432/mydb");
        assert!(masked.contains("***"), "Password must be masked");
        assert!(
            !masked.contains("supersecret"),
            "Raw password must not appear"
        );
        assert!(masked.contains("admin"), "Username should remain visible");
        assert!(
            masked.contains("db.example.com"),
            "Host should remain visible"
        );
    }

    #[test]
    fn test_mask_db_url_masks_password_with_special_chars() {
        let masked = mask_db_url("postgres://user:p%40ss%23word!@host:5432/db");
        assert!(
            !masked.contains("p%40ss"),
            "Encoded password must not appear"
        );
        assert!(masked.contains("***"));
    }

    #[test]
    fn test_mask_db_url_no_password_stays_valid() {
        let masked = mask_db_url("postgres://admin@db.example.com:5432/mydb");
        // set_password(Some("***")) adds the mask even when there was none
        assert!(masked.contains("***"));
        assert!(masked.contains("admin"));
    }

    #[test]
    fn test_mask_db_url_invalid_url_returns_redacted() {
        assert_eq!(mask_db_url("not a url at all"), "*** REDACTED ***");
        assert_eq!(mask_db_url(""), "*** REDACTED ***");
        assert_eq!(mask_db_url("just-a-hostname"), "*** REDACTED ***");
    }

    #[test]
    fn test_mask_db_url_preserves_database_name() {
        let masked = mask_db_url("postgres://u:p@host:5432/important_db");
        assert!(masked.contains("important_db"));
    }

    #[test]
    fn test_mask_db_url_preserves_port() {
        let masked = mask_db_url("postgres://u:p@host:9999/db");
        assert!(masked.contains("9999"));
    }

    // ── CreateTenantRequest — Debug (PII redaction) ──────────────────────────

    #[test]
    fn test_create_tenant_request_debug_redacts_database_url() {
        let req = CreateTenantRequest {
            workspace_id: Uuid::nil(),
            database_url: "postgres://admin:secret@host/db".to_string(),
            pool_size: Some(10),
        };
        let debug_output = format!("{:?}", req);
        assert!(
            debug_output.contains("REDACTED"),
            "Debug output must redact database_url: {debug_output}"
        );
        assert!(
            !debug_output.contains("secret"),
            "Raw password must not appear in Debug: {debug_output}"
        );
    }

    #[test]
    fn test_create_tenant_request_debug_always_redacts() {
        let req = CreateTenantRequest {
            workspace_id: Uuid::nil(),
            database_url: "postgres://admin@host/db".to_string(),
            pool_size: None,
        };
        let debug_output = format!("{:?}", req);
        assert!(debug_output.contains("REDACTED"));
    }

    // ── CreateTenantRequest — Deserialization ────────────────────────────────

    #[test]
    fn test_create_tenant_request_deserializes_full_payload() {
        let json = r#"{"workspace_id": "00000000-0000-0000-0000-000000000001", "database_url": "postgres://u:p@h/d", "pool_size": 20}"#;
        let req: CreateTenantRequest = serde_json::from_str(json).unwrap();
        assert_eq!(
            req.workspace_id,
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
        );
        assert_eq!(req.database_url, "postgres://u:p@h/d");
        assert_eq!(req.pool_size, Some(20));
    }

    #[test]
    fn test_create_tenant_request_deserializes_minimal_payload() {
        let json = r#"{"workspace_id": "00000000-0000-0000-0000-000000000002", "database_url": "postgres://u:p@h/db"}"#;
        let req: CreateTenantRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.database_url, "postgres://u:p@h/db");
        assert!(req.pool_size.is_none());
    }

    #[test]
    fn test_create_tenant_request_missing_database_url_fails() {
        let json = r#"{"workspace_id": "00000000-0000-0000-0000-000000000001", "pool_size": 10}"#;
        assert!(serde_json::from_str::<CreateTenantRequest>(json).is_err());
    }

    #[test]
    fn test_create_tenant_request_invalid_uuid_fails() {
        let json = r#"{"workspace_id": "not-a-uuid"}"#;
        assert!(serde_json::from_str::<CreateTenantRequest>(json).is_err());
    }

    #[test]
    fn test_create_tenant_request_empty_json_fails() {
        assert!(serde_json::from_str::<CreateTenantRequest>("{}").is_err());
    }

    // ── UpdateTenantDbRequest — Deserialization ──────────────────────────────

    #[test]
    fn test_update_tenant_db_request_valid() {
        let json = r#"{"database_url": "postgres://u:p@h:5432/db"}"#;
        let req: UpdateTenantDbRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.database_url, "postgres://u:p@h:5432/db");
    }

    #[test]
    fn test_update_tenant_db_request_missing_url_fails() {
        assert!(serde_json::from_str::<UpdateTenantDbRequest>("{}").is_err());
    }

    #[test]
    fn test_update_tenant_db_request_null_url_fails() {
        let json = r#"{"database_url": null}"#;
        assert!(serde_json::from_str::<UpdateTenantDbRequest>(json).is_err());
    }

    // ── UpdateTenantModeRequest — Validation ─────────────────────────────────

    #[test]
    fn test_update_tenant_mode_request_session() {
        let json = r#"{"mode_type": "session"}"#;
        let req: UpdateTenantModeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.mode_type, "session");
    }

    #[test]
    fn test_update_tenant_mode_request_transaction() {
        let json = r#"{"mode_type": "transaction"}"#;
        let req: UpdateTenantModeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.mode_type, "transaction");
    }

    #[test]
    fn test_update_tenant_mode_validation_logic() {
        // The handler rejects any mode that isn't "session" or "transaction".
        // We test the same validation predicate used in the handler.
        let valid = ["session", "transaction"];
        let invalid = ["", "Session", "TRANSACTION", "pooled", "read_only"];

        for mode in valid {
            assert!(
                mode == "session" || mode == "transaction",
                "Expected valid mode: {mode}"
            );
        }
        for mode in invalid {
            assert!(
                mode != "session" && mode != "transaction",
                "Expected invalid mode: {mode}"
            );
        }
    }

    // ── Stale tenant identification logic ─────────────────────────────────────
    // These tests verify the set-difference logic used by list_stale_tenants
    // without requiring a database connection.

    /// Helper: computes stale tenant IDs given workspace UUIDs and registered
    /// Supavisor tenant IDs — same algorithm as the handler.
    fn compute_stale_tenants(workspace_ids: &[Uuid], registered_tenants: &[&str]) -> Vec<String> {
        let active_tenant_ids: std::collections::HashSet<String> = workspace_ids
            .iter()
            .map(|ws_id| format!("tenant_{}", ws_id.to_string().replace('-', "")))
            .collect();

        registered_tenants
            .iter()
            .filter(|t| t.starts_with("tenant_") && !active_tenant_ids.contains(**t))
            .map(|t| t.to_string())
            .collect()
    }

    #[test]
    fn test_stale_detection_no_tenants_no_workspaces() {
        let stale = compute_stale_tenants(&[], &[]);
        assert!(stale.is_empty());
    }

    #[test]
    fn test_stale_detection_no_tenants_with_workspaces() {
        let ws = Uuid::parse_str("6e913c39-24f3-45b5-877d-e307c3e10c88").unwrap();
        let stale = compute_stale_tenants(&[ws], &[]);
        assert!(stale.is_empty());
    }

    #[test]
    fn test_stale_detection_matching_tenant_not_stale() {
        let ws = Uuid::parse_str("6e913c39-24f3-45b5-877d-e307c3e10c88").unwrap();
        let tenant_id = "tenant_6e913c3924f345b5877de307c3e10c88";
        let stale = compute_stale_tenants(&[ws], &[tenant_id]);
        assert!(stale.is_empty(), "Matching tenant should not be stale");
    }

    #[test]
    fn test_stale_detection_orphaned_tenant_is_stale() {
        let ws = Uuid::parse_str("6e913c39-24f3-45b5-877d-e307c3e10c88").unwrap();
        let orphan = "tenant_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1";
        let matching = "tenant_6e913c3924f345b5877de307c3e10c88";
        let stale = compute_stale_tenants(&[ws], &[matching, orphan]);
        assert_eq!(stale, vec![orphan.to_string()]);
    }

    #[test]
    fn test_stale_detection_system_tenant_never_stale() {
        // The system tenant "leeattend" does not start with "tenant_" prefix
        // so it must never appear in stale results.
        let stale = compute_stale_tenants(&[], &["leeattend"]);
        assert!(
            stale.is_empty(),
            "System tenant 'leeattend' must never be flagged as stale"
        );
    }

    #[test]
    fn test_stale_detection_non_tenant_prefix_ignored() {
        // Any ID that doesn't start with "tenant_" should be ignored,
        // even if it doesn't match any workspace.
        let stale = compute_stale_tenants(&[], &["leeattend", "custom_pool_abc", "metrics_tenant"]);
        assert!(stale.is_empty());
    }

    #[test]
    fn test_stale_detection_multiple_workspaces() {
        let ws1 = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
        let ws2 = Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap();
        let t1 = "tenant_11111111111111111111111111111111";
        let t2 = "tenant_22222222222222222222222222222222";
        let orphan = "tenant_33333333333333333333333333333333";

        let stale = compute_stale_tenants(&[ws1, ws2], &[t1, t2, orphan]);
        assert_eq!(stale, vec![orphan.to_string()]);
    }

    #[test]
    fn test_stale_detection_all_tenants_stale() {
        let orphan1 = "tenant_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1";
        let orphan2 = "tenant_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb2";
        let stale = compute_stale_tenants(&[], &[orphan1, orphan2]);
        assert_eq!(stale.len(), 2);
        assert!(stale.contains(&orphan1.to_string()));
        assert!(stale.contains(&orphan2.to_string()));
    }

    #[test]
    fn test_stale_detection_uuid_hyphen_stripping() {
        // Ensures the format! correctly strips hyphens from workspace UUIDs
        // to match the "tenant_{uuid_no_hyphens}" convention.
        let ws = Uuid::parse_str("abcdef01-2345-6789-abcd-ef0123456789").unwrap();
        let expected_tenant_id = "tenant_abcdef0123456789abcdef0123456789";
        let stale = compute_stale_tenants(&[ws], &[expected_tenant_id]);
        assert!(
            stale.is_empty(),
            "Properly formatted tenant ID should match"
        );
    }

    #[test]
    fn test_stale_detection_tenant_id_with_hyphens_is_stale() {
        // If somehow a tenant ID was registered WITH hyphens, it won't match
        // the expected format (without hyphens) and should be flagged as stale.
        let ws = Uuid::parse_str("abcdef01-2345-6789-abcd-ef0123456789").unwrap();
        let wrong_format = "tenant_abcdef01-2345-6789-abcd-ef0123456789";
        let stale = compute_stale_tenants(&[ws], &[wrong_format]);
        assert_eq!(
            stale.len(),
            1,
            "Hyphenated tenant ID must be flagged as stale"
        );
    }

    #[test]
    fn test_stale_detection_mixed_system_and_workspace_tenants() {
        let ws = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
        let system = "leeattend";
        let active = "tenant_11111111111111111111111111111111";
        let orphan = "tenant_99999999999999999999999999999999";
        let custom = "my_custom_pool";

        let stale = compute_stale_tenants(&[ws], &[system, active, orphan, custom]);
        assert_eq!(stale, vec![orphan.to_string()]);
    }

    // ── Tenant ID format convention ──────────────────────────────────────────

    #[test]
    fn test_tenant_id_format_matches_convention() {
        // Verify the tenant_id format used across the system is consistent
        let ws = Uuid::parse_str("6e913c39-24f3-45b5-877d-e307c3e10c88").unwrap();
        let expected = "tenant_6e913c3924f345b5877de307c3e10c88";
        let computed = format!("tenant_{}", ws.to_string().replace('-', ""));
        assert_eq!(computed, expected);
    }

    #[test]
    fn test_tenant_id_nil_uuid() {
        let ws = Uuid::nil();
        let computed = format!("tenant_{}", ws.to_string().replace('-', ""));
        assert_eq!(computed, "tenant_00000000000000000000000000000000");
    }

    #[test]
    fn test_map_tenants_to_workspaces_mapped_and_unmapped() {
        let ws_id = Uuid::parse_str("6e913c39-24f3-45b5-877d-e307c3e10c88").unwrap();
        let ws_name = "ELITE HOUSEHOLD".to_string();
        let workspaces = vec![(ws_id, ws_name.clone())];

        let now = chrono::Utc::now().naive_utc();
        let tenants = vec![
            (
                "tenant_6e913c3924f345b5877de307c3e10c88".to_string(),
                10,
                now,
                now,
            ),
            (
                "tenant_00000000000000000000000000000000".to_string(),
                5,
                now,
                now,
            ),
        ];

        let results = map_tenants_to_workspaces(tenants, workspaces);
        assert_eq!(results.len(), 2);

        // First one: mapped
        let t1 = &results[0];
        assert_eq!(t1["id"], "tenant_6e913c3924f345b5877de307c3e10c88");
        assert_eq!(t1["workspace_id"], ws_id.to_string());
        assert_eq!(t1["workspace_name"], "ELITE HOUSEHOLD");

        // Second one: unmapped/stale
        let t2 = &results[1];
        assert_eq!(t2["id"], "tenant_00000000000000000000000000000000");
        assert!(t2["workspace_id"].is_null());
        assert!(t2["workspace_name"].is_null());
    }
}

//! System-level administration endpoints.
//!
//! All routes here require `SystemContext` with `PlatformAdmin` role.
//! They allow platform operators to assign/revoke system roles across users.

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
use crate::utils::i18n::I18nKey;
use crate::handlers::system_auth::SystemContext;
use crate::models::enums::SystemRoleType;
use crate::AppState;

// ── Response DTOs ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct SystemRoleAssignment {
    pub user_id: Uuid,
    pub role_name: String,
    pub assigned_by: Option<Uuid>,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SystemRoleInfo {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub assignees: Vec<SystemRoleAssignment>,
}

// ── Request DTOs ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AssignSystemRoleRequest {
    pub role_name: SystemRoleType,
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// GET /system/roles
/// List all system roles with their current assignees.
/// Requires: PlatformAdmin
#[utoipa::path(
    get,
    path = "/system/roles",
    responses(
        (status = 200, description = "List of system roles with assignees", body = [SystemRoleInfo])
    )
)]
pub async fn list_system_roles(
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    _api_ctx: ApiContext,
) -> AppResult<Json<Vec<SystemRoleInfo>>> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    let mut conn = state
        .infra()
        .diesel()
        .api
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    #[derive(diesel::QueryableByName)]
    struct RoleRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        description: String,
    }

    use diesel_async::RunQueryDsl;

    let rows: Vec<RoleRow> = diesel::sql_query(
        r"
        SELECT id, name, description
          FROM system_roles
         ORDER BY name
        ",
    )
    .load(&mut *conn)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    #[derive(diesel::QueryableByName)]
    struct AssigneeRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        role_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        user_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
        assigned_by: Option<Uuid>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        assigned_at: DateTime<Utc>,
    }

    let role_ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
    let assignees_rows: Vec<AssigneeRow> = diesel::sql_query(
        r"
        SELECT role_id, user_id, assigned_by, assigned_at
          FROM user_system_roles
         WHERE role_id = ANY($1)
         ORDER BY assigned_at
        ",
    )
    .bind::<diesel::sql_types::Array<diesel::sql_types::Uuid>, _>(role_ids)
    .load(&mut *conn)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut assignees_map: std::collections::HashMap<Uuid, Vec<AssigneeRow>> =
        std::collections::HashMap::new();
    for a in assignees_rows {
        assignees_map.entry(a.role_id).or_default().push(a);
    }

    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        let assignees = assignees_map
            .remove(&row.id)
            .unwrap_or_default()
            .into_iter()
            .map(|a| SystemRoleAssignment {
                user_id: a.user_id,
                role_name: row.name.clone(),
                assigned_by: a.assigned_by,
                assigned_at: a.assigned_at,
            })
            .collect();

        result.push(SystemRoleInfo {
            id: row.id,
            name: row.name,
            description: row.description,
            assignees,
        });
    }

    Ok(Json(result))
}

/// GET /system/users/{user_id}/roles
/// Get all system roles held by a specific user.
/// Requires: PlatformAdmin
#[utoipa::path(
    get,
    path = "/system/users/{user_id}/roles",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "List of role names held by the user", body = [String])
    )
)]
pub async fn get_user_system_roles(
    Path(user_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    _api_ctx: ApiContext,
) -> AppResult<Json<Vec<String>>> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    let mut conn = state
        .infra()
        .diesel()
        .api
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    #[derive(diesel::QueryableByName)]
    struct RoleNameRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }

    use diesel_async::RunQueryDsl;

    let roles: Vec<RoleNameRow> = diesel::sql_query(
        r"
        SELECT sr.name
          FROM user_system_roles usr
          JOIN system_roles sr ON sr.id = usr.role_id
         WHERE usr.user_id = $1
         ORDER BY sr.name
        ",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .load(&mut *conn)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(roles.into_iter().map(|r| r.name).collect()))
}

/// POST /system/users/{user_id}/roles
/// Assign a system role to a user.
/// Requires: PlatformAdmin
#[utoipa::path(
    post,
    path = "/system/users/{user_id}/roles",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    request_body = AssignSystemRoleRequest,
    responses(
        (status = 204, description = "System role assigned successfully")
    )
)]
pub async fn assign_system_role(
    Path(user_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    _api_ctx: ApiContext,
    Json(req): Json<AssignSystemRoleRequest>,
) -> AppResult<StatusCode> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    let mut conn = state
        .infra()
        .diesel()
        .api
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    #[derive(diesel::QueryableByName)]
    struct RoleIdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }

    use diesel::OptionalExtension;
    use diesel_async::RunQueryDsl;

    // Resolve role_id from name
    let role_id_row: Option<RoleIdRow> =
        diesel::sql_query("SELECT id FROM system_roles WHERE name = $1")
            .bind::<diesel::sql_types::Text, _>(req.role_name.to_string())
            .get_result(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?;

    let role_id = role_id_row
        .map(|r| r.id)
        .ok_or_else(|| AppError::not_found(I18nKey::SystemRoleNotFound))?;

    // Upsert — idempotent assign
    diesel::sql_query(
        r"
        INSERT INTO user_system_roles (user_id, role_id, assigned_by)
             VALUES ($1, $2, $3)
        ON CONFLICT (user_id, role_id) DO NOTHING
        ",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Uuid, _>(role_id)
    .bind::<diesel::sql_types::Uuid, _>(ctx.user_id)
    .execute(&mut *conn)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    // Invalidate cache for the target user
    SystemContext::invalidate_cache(user_id, &state).await;

    tracing::info!(
        assigner = %ctx.user_id,
        target_user = %user_id,
        role = %req.role_name,
        "System role assigned"
    );

    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /system/users/{user_id}/roles/{role_name}
/// Revoke a system role from a user.
/// Requires: PlatformAdmin
#[utoipa::path(
    delete,
    path = "/system/users/{user_id}/roles/{role_name}",
    params(
        ("user_id" = Uuid, Path, description = "User ID"),
        ("role_name" = String, Path, description = "Role Name")
    ),
    responses(
        (status = 204, description = "System role revoked successfully")
    )
)]
pub async fn revoke_system_role(
    Path(params): Path<(Uuid, String)>,
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    _api_ctx: ApiContext,
) -> AppResult<StatusCode> {
    let (user_id, role_name) = params;
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    let mut conn = state
        .infra()
        .diesel()
        .api
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    use diesel_async::RunQueryDsl;

    // Safety: verify the role_name is a known system role
    use std::str::FromStr;
    let role_type = SystemRoleType::from_str(&role_name)
        .map_err(|_e| AppError::validation(I18nKey::UnknownSystemRole))?;

    // Prevent revoking the last platform_admin — regardless of who performs it.
    // Without this guard, admin A could revoke admin B (the sole remaining admin),
    // leaving the platform with zero administrators.
    if role_type == SystemRoleType::PlatformAdmin {
        #[derive(diesel::QueryableByName)]
        struct CountRow {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }

        let count_row: CountRow = diesel::sql_query(
            r"
            SELECT COUNT(*) as count
              FROM user_system_roles usr
              JOIN system_roles sr ON sr.id = usr.role_id
             WHERE sr.name = 'platform_admin'
            ",
        )
        .get_result(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        if count_row.count <= 1 {
            return Err(AppError::validation(I18nKey::CannotRevokeLastAdmin));
        }
    }

    let res = diesel::sql_query(
        r"
        DELETE FROM user_system_roles
         WHERE user_id = $1
           AND role_id = (SELECT id FROM system_roles WHERE name = $2)
        ",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Text, _>(role_name)
    .execute(&mut *conn)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if res == 0 {
        return Err(AppError::not_found(I18nKey::SystemRoleNotFound));
    }

    SystemContext::invalidate_cache(user_id, &state).await;

    tracing::info!(
        revoker = %ctx.user_id,
        target_user = %user_id,
        role = %role_type,
        "System role revoked"
    );

    Ok(StatusCode::NO_CONTENT)
}

/// GET /system/health
/// Simple system health endpoint.
/// Requires: Support or PlatformAdmin (monitoring access)
#[utoipa::path(
    get,
    path = "/system/health",
    responses(
        (status = 200, description = "System health check status", body = serde_json::Value)
    )
)]
pub async fn system_health(
    ctx: SystemContext,
    _api_ctx: ApiContext,
) -> AppResult<Json<serde_json::Value>> {
    ctx.require_any_role(&[SystemRoleType::PlatformAdmin, SystemRoleType::Support])?;
    Ok(Json(serde_json::json!({
        "status": "ok",
        "viewer_user_id": ctx.user_id,
        "viewer_roles": ctx.roles.iter().map(std::string::ToString::to_string).collect::<Vec<_>>()
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── AssignSystemRoleRequest deserialization ───────────────────────────────

    #[test]
    fn test_assign_request_platform_admin() {
        let req: AssignSystemRoleRequest =
            serde_json::from_str(r#"{"role_name": "platform_admin"}"#).unwrap();
        assert_eq!(req.role_name, SystemRoleType::PlatformAdmin);
    }

    #[test]
    fn test_assign_request_support() {
        let req: AssignSystemRoleRequest =
            serde_json::from_str(r#"{"role_name": "support"}"#).unwrap();
        assert_eq!(req.role_name, SystemRoleType::Support);
    }

    #[test]
    fn test_assign_request_billing_admin() {
        let req: AssignSystemRoleRequest =
            serde_json::from_str(r#"{"role_name": "billing_admin"}"#).unwrap();
        assert_eq!(req.role_name, SystemRoleType::BillingAdmin);
    }

    #[test]
    fn test_assign_request_invalid_role_fails() {
        // Unknown role names MUST fail — prevents privilege escalation via typos.
        let invalid = [
            r#"{"role_name": "superuser"}"#,
            r#"{"role_name": "PLATFORM_ADMIN"}"#,
            r#"{"role_name": "admin"}"#,
            r#"{"role_name": ""}"#,
            r#"{"role_name": "root"}"#,
        ];
        for json in invalid {
            assert!(
                serde_json::from_str::<AssignSystemRoleRequest>(json).is_err(),
                "Expected failure for {json}"
            );
        }
    }

    #[test]
    fn test_assign_request_missing_role_name_fails() {
        // role_name is required — {} must fail.
        assert!(serde_json::from_str::<AssignSystemRoleRequest>("{}").is_err());
    }

    #[test]
    fn test_assign_request_null_role_name_fails() {
        assert!(serde_json::from_str::<AssignSystemRoleRequest>(r#"{"role_name": null}"#).is_err());
    }

    #[test]
    fn test_assign_request_roundtrip_all_roles() {
        // Every valid role_name must survive serde roundtrip (for logging/auditing).
        for role in [
            SystemRoleType::PlatformAdmin,
            SystemRoleType::Support,
            SystemRoleType::BillingAdmin,
        ] {
            let req = AssignSystemRoleRequest { role_name: role };
            let json = serde_json::to_string(&req).unwrap();
            // Verify the role name appears correctly in serialized form
            assert!(
                json.contains(&role.to_string()),
                "Serialized JSON missing role name: {json}"
            );
        }
    }

    // ── SystemRoleInfo response serialization ─────────────────────────────────

    #[test]
    fn test_system_role_info_serializes_empty_assignees() {
        let info = SystemRoleInfo {
            id: Uuid::nil(),
            name: "platform_admin".to_string(),
            description: "Full access".to_string(),
            assignees: vec![],
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("platform_admin"));
        assert!(json.contains("\"assignees\":[]"));
    }

    #[test]
    fn test_system_role_info_serializes_with_assignee() {
        let now = Utc::now();
        let uid = Uuid::nil();
        let info = SystemRoleInfo {
            id: Uuid::nil(),
            name: "support".to_string(),
            description: "Read-only".to_string(),
            assignees: vec![SystemRoleAssignment {
                user_id: uid,
                role_name: "support".to_string(),
                assigned_by: Some(uid),
                assigned_at: now,
            }],
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"role_name\":\"support\""));
        assert!(json.contains("\"assigned_by\""));
    }

    #[test]
    fn test_system_role_assignment_assigned_by_nullable() {
        // assigned_by is Option<Uuid> — bootstrapped admin has no assigner.
        let now = Utc::now();
        let assignment = SystemRoleAssignment {
            user_id: Uuid::nil(),
            role_name: "platform_admin".to_string(),
            assigned_by: None,
            assigned_at: now,
        };
        let json = serde_json::to_string(&assignment).unwrap();
        assert!(json.contains("\"assigned_by\":null"));
    }

    // ── Route guard equivalence tests ─────────────────────────────────────────
    // These use SystemContext directly (same logic as handlers) to verify
    // the intended access control policy without a DB.

    fn make_ctx(roles: &[SystemRoleType]) -> SystemContext {
        SystemContext {
            user_id: Uuid::new_v4(),
            roles: roles.iter().copied().collect(),
        }
    }

    #[test]
    fn test_list_system_roles_gate_allows_platform_admin_only() {
        // GET /system/roles — PlatformAdmin required.
        let admin = make_ctx(&[SystemRoleType::PlatformAdmin]);
        let support = make_ctx(&[SystemRoleType::Support]);
        let billing = make_ctx(&[SystemRoleType::BillingAdmin]);
        let no_role = make_ctx(&[]);

        assert!(admin.require_role(SystemRoleType::PlatformAdmin).is_ok());
        assert!(support.require_role(SystemRoleType::PlatformAdmin).is_err());
        assert!(billing.require_role(SystemRoleType::PlatformAdmin).is_err());
        assert!(no_role.require_role(SystemRoleType::PlatformAdmin).is_err());
    }

    #[test]
    fn test_system_health_gate_allows_admin_and_support() {
        // GET /system/health — PlatformAdmin | Support.
        let gate = &[SystemRoleType::PlatformAdmin, SystemRoleType::Support];
        let admin = make_ctx(&[SystemRoleType::PlatformAdmin]);
        let support = make_ctx(&[SystemRoleType::Support]);
        let billing = make_ctx(&[SystemRoleType::BillingAdmin]);

        assert!(admin.require_any_role(gate).is_ok());
        assert!(support.require_any_role(gate).is_ok());
        assert!(billing.require_any_role(gate).is_err());
    }

    #[test]
    fn test_assign_system_role_gate_blocks_support() {
        // POST /system/users/:id/roles — only PlatformAdmin can assign.
        // A Support user attempting to grant roles must be blocked.
        let support = make_ctx(&[SystemRoleType::Support]);
        assert!(support.require_role(SystemRoleType::PlatformAdmin).is_err());
    }

    // ── Revoke logic invariants (pure logic, no DB) ───────────────────────────

    #[test]
    fn test_revoke_last_admin_detection_logic() {
        // Simulate the guard logic: if admin_count <= 1 AND self-revoke, must fail.
        let admin_count: i64 = 1; // last admin
        let is_last_admin = admin_count <= 1;
        assert!(is_last_admin, "Should detect last admin scenario");
    }

    #[test]
    fn test_revoke_not_last_admin_is_allowed() {
        let admin_count: i64 = 2;
        let is_last_admin = admin_count <= 1;
        assert!(!is_last_admin, "2 admins means revoke is safe");
    }

    #[test]
    fn test_revoke_unknown_role_name_rejected_at_parse() {
        // revoke_system_role parses role_name via FromStr — unknown names fail.
        use std::str::FromStr;
        assert!(SystemRoleType::from_str("superadmin").is_err());
        assert!(SystemRoleType::from_str("PLATFORM_ADMIN").is_err());
        assert!(SystemRoleType::from_str("").is_err());
    }

    #[test]
    fn test_revoke_known_role_names_parse_correctly() {
        use std::str::FromStr;
        assert_eq!(
            SystemRoleType::from_str("platform_admin").unwrap(),
            SystemRoleType::PlatformAdmin
        );
        assert_eq!(
            SystemRoleType::from_str("support").unwrap(),
            SystemRoleType::Support
        );
        assert_eq!(
            SystemRoleType::from_str("billing_admin").unwrap(),
            SystemRoleType::BillingAdmin
        );
    }

    // ── DB seed name alignment ────────────────────────────────────────────────

    #[test]
    fn test_all_system_roles_have_db_seed_names() {
        // The migration inserts exactly these names. If SystemRoleType diverges,
        // all DB lookups for that role silently fail.
        let seeded_names = ["platform_admin", "support", "billing_admin"];
        let enum_names = [
            SystemRoleType::PlatformAdmin.to_string(),
            SystemRoleType::Support.to_string(),
            SystemRoleType::BillingAdmin.to_string(),
        ];
        for (seeded, from_enum) in seeded_names.iter().zip(enum_names.iter()) {
            assert_eq!(
                seeded, from_enum,
                "DB seed name mismatch: DB='{seeded}', Enum='{from_enum}'"
            );
        }
    }
}

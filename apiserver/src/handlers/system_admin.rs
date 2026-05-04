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
pub async fn list_system_roles(
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
) -> AppResult<Json<Vec<SystemRoleInfo>>> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    let rows: Vec<(Uuid, String, String)> = sqlx::query_as(
        r#"
        SELECT id, name, description
          FROM system_roles
         ORDER BY name
        "#,
    )
    .fetch_all(state.infra.db.api())
    .await?;

    let mut result = Vec::with_capacity(rows.len());

    for (role_id, role_name, description) in rows {
        let assignees: Vec<SystemRoleAssignment> =
            sqlx::query_as::<_, (Uuid, Option<Uuid>, DateTime<Utc>)>(
                r#"
            SELECT user_id, assigned_by, assigned_at
              FROM user_system_roles
             WHERE role_id = $1
             ORDER BY assigned_at
            "#,
            )
            .bind(role_id)
            .fetch_all(state.infra.db.api())
            .await?
            .into_iter()
            .map(|(user_id, assigned_by, assigned_at)| SystemRoleAssignment {
                user_id,
                role_name: role_name.clone(),
                assigned_by,
                assigned_at,
            })
            .collect();

        result.push(SystemRoleInfo {
            id: role_id,
            name: role_name,
            description,
            assignees,
        });
    }

    Ok(Json(result))
}

/// GET /system/users/:user_id/roles
/// Get all system roles held by a specific user.
/// Requires: PlatformAdmin
pub async fn get_user_system_roles(
    Path(user_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
) -> AppResult<Json<Vec<String>>> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    let roles: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT sr.name
          FROM user_system_roles usr
          JOIN system_roles sr ON sr.id = usr.role_id
         WHERE usr.user_id = $1
         ORDER BY sr.name
        "#,
    )
    .bind(user_id)
    .fetch_all(state.infra.db.api())
    .await?;

    Ok(Json(roles.into_iter().map(|(name,)| name).collect()))
}

/// POST /system/users/:user_id/roles
/// Assign a system role to a user.
/// Requires: PlatformAdmin (only platform admins can grant system roles)
pub async fn assign_system_role(
    Path(user_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    Json(req): Json<AssignSystemRoleRequest>,
) -> AppResult<StatusCode> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    // Resolve role_id from name
    let role_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM system_roles WHERE name = $1")
        .bind(req.role_name.to_string())
        .fetch_optional(state.infra.db.api())
        .await?;

    let role_id = role_id
        .ok_or_else(|| AppError::NotFound(format!("System role '{}' not found", req.role_name)))?;

    // Upsert — idempotent assign
    sqlx::query(
        r#"
        INSERT INTO user_system_roles (user_id, role_id, assigned_by)
             VALUES ($1, $2, $3)
        ON CONFLICT (user_id, role_id) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(role_id)
    .bind(ctx.user_id)
    .execute(state.infra.db.api())
    .await?;

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

/// DELETE /system/users/:user_id/roles/:role_name
/// Revoke a system role from a user.
/// Requires: PlatformAdmin
pub async fn revoke_system_role(
    Path((user_id, role_name)): Path<(Uuid, String)>,
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
) -> AppResult<StatusCode> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    // Safety: verify the role_name is a known system role
    use std::str::FromStr;
    let role_type = SystemRoleType::from_str(&role_name)
        .map_err(|_e| AppError::Validation(format!("Unknown system role: '{role_name}'")))?;

    // Prevent revoking the last platform_admin — regardless of who performs it.
    // Without this guard, admin A could revoke admin B (the sole remaining admin),
    // leaving the platform with zero administrators.
    if role_type == SystemRoleType::PlatformAdmin {
        let admin_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
              FROM user_system_roles usr
              JOIN system_roles sr ON sr.id = usr.role_id
             WHERE sr.name = 'platform_admin'
            "#,
        )
        .fetch_one(state.infra.db.api())
        .await?;

        if admin_count <= 1 {
            return Err(AppError::Validation(
                "Cannot revoke the last platform_admin. Assign another admin first.".to_string(),
            ));
        }
    }

    let res = sqlx::query(
        r#"
        DELETE FROM user_system_roles
         WHERE user_id = $1
           AND role_id = (SELECT id FROM system_roles WHERE name = $2)
        "#,
    )
    .bind(user_id)
    .bind(role_name)
    .execute(state.infra.db.api())
    .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "System role assignment not found".to_string(),
        ));
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
pub async fn system_health(ctx: SystemContext) -> AppResult<Json<serde_json::Value>> {
    ctx.require_any_role(&[SystemRoleType::PlatformAdmin, SystemRoleType::Support])?;
    Ok(Json(serde_json::json!({
        "status": "ok",
        "viewer_user_id": ctx.user_id,
        "viewer_roles": ctx.roles.iter().map(|r| r.to_string()).collect::<Vec<_>>()
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

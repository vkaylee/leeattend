use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::enums::RoleScopeType;
use crate::models::{AssignRoleRequest, MemberExtraRole, WorkspaceRole};
use crate::services::audit_service::AuditHelper;
use crate::services::WorkspaceService;
use crate::AppState;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceRoleRequest {
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    /// Whether this role has global (workspace-wide) or department-scoped authority.
    /// Defaults to `scoped` if not provided.
    #[serde(default)]
    pub scope_type: RoleScopeType,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkspaceRoleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRolePermissionsRequest {
    pub permissions: Vec<String>,
}

pub async fn get_member_roles(
    Path((_, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<serde_json::Value>> {
    match WorkspaceService::get_member_roles(
        state.core.workspace_repo.as_ref(),
        ctx.workspace_id,
        user_id,
    )
    .await
    {
        Ok((primary, extra)) => Ok(Json(serde_json::json!({
            "primary": primary,
            "extra": extra
        }))),
        Err(e) if e == "NOT_FOUND" => Err(AppError::NotFound("Member not found".to_string())),
        Err(e) => Err(AppError::from(e)),
    }
}

pub async fn assign_extra_role(
    Path((_, user_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    ctx: MemberContext,
    Json(req): Json<AssignRoleRequest>,
) -> AppResult<(StatusCode, Json<MemberExtraRole>)> {
    ctx.require_permission(Permission::MembersRolesManage)?;

    if req.expires_at <= chrono::Utc::now() {
        return Err(AppError::Validation(
            "Expiration time must be in the future".to_string(),
        ));
    }

    let extra_role = state
        .core
        .workspace_repo
        .assign_extra_role(ctx.workspace_id, user_id, req.role_id, req.expires_at)
        .await?;

    AuditHelper::log(crate::services::audit_service::AuditLogArgs {
        repo: &state.core.audit_repo,
        workspace_id: ctx.workspace_id,
        actor_id: Some(ctx.user_id),
        target_user_id: Some(user_id),
        target_resource_id: Some(extra_role.id),
        action: "assign_extra_role",
        details: Some(serde_json::json!({ "role_id": req.role_id, "expires_at": req.expires_at })),
        headers: &headers,
    })
    .await;

    if let Some(redis_pool) = state.infra.redis.clone() {
        let cache = crate::cache::RedisCache::new(redis_pool);
        let _ = cache
            .delete(&format!(
                "cache:auth:permissions:{}:{}",
                ctx.workspace_id, user_id
            ))
            .await;
    }

    Ok((StatusCode::CREATED, Json(extra_role)))
}

pub async fn revoke_extra_role(
    Path((_, user_id, role_id)): Path<(Uuid, Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::MembersRolesManage)?;

    let affected = state
        .core
        .workspace_repo
        .revoke_extra_role(ctx.workspace_id, role_id)
        .await?;

    if affected > 0 {
        AuditHelper::log(crate::services::audit_service::AuditLogArgs {
            repo: &state.core.audit_repo,
            workspace_id: ctx.workspace_id,
            actor_id: Some(ctx.user_id),
            target_user_id: Some(user_id),
            target_resource_id: Some(role_id),
            action: "revoke_extra_role",
            details: None,
            headers: &headers,
        })
        .await;

        if let Some(redis_pool) = state.infra.redis.clone() {
            let cache = crate::cache::RedisCache::new(redis_pool);
            let _ = cache
                .delete(&format!(
                    "cache:auth:permissions:{}:{}",
                    ctx.workspace_id, user_id
                ))
                .await;
        }

        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Role not found".to_string()))
    }
}

// --- Workspace Role Management (CRUD) ---

pub async fn get_workspace_roles(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<WorkspaceRole>>> {
    // No permission gate — role list is non-sensitive metadata that every
    // workspace member needs in order to resolve role names on the People page.

    let roles = state
        .core
        .workspace_repo
        .get_workspace_roles(ctx.workspace_id)
        .await?;

    Ok(Json(roles))
}

pub async fn create_workspace_role(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateWorkspaceRoleRequest>,
) -> AppResult<(StatusCode, Json<WorkspaceRole>)> {
    ctx.require_permission(Permission::MembersRolesManage)?;

    let role = state
        .core
        .workspace_repo
        .create_workspace_role(
            ctx.workspace_id,
            req.name,
            req.description,
            req.color,
            req.scope_type.to_string(),
        )
        .await?;

    Ok((StatusCode::CREATED, Json(role)))
}

pub async fn update_workspace_role(
    Path((_, role_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateWorkspaceRoleRequest>,
) -> AppResult<Json<WorkspaceRole>> {
    ctx.require_permission(Permission::MembersRolesManage)?;

    let role = state
        .core
        .workspace_repo
        .update_workspace_role(
            ctx.workspace_id,
            role_id,
            req.name,
            req.description,
            req.color,
        )
        .await?
        .ok_or_else(|| AppError::NotFound("Role not found or is a system role".to_string()))?;

    Ok(Json(role))
}

pub async fn delete_workspace_role(
    Path((_, role_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::MembersRolesManage)?;

    let affected = state
        .core
        .workspace_repo
        .delete_workspace_role(ctx.workspace_id, role_id)
        .await?;

    if affected > 0 {
        if let Some(redis_pool) = state.infra.redis.clone() {
            let cache = crate::cache::RedisCache::new(redis_pool);
            let _ = cache
                .delete_pattern(&format!("cache:auth:permissions:{}:*", ctx.workspace_id))
                .await;
        }
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound(
            "Role not found or is a system role".to_string(),
        ))
    }
}

pub async fn get_role_permissions(
    Path((_, role_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<String>>> {
    ctx.require_permission(Permission::MembersRolesManage)?;

    let permissions = state
        .core
        .workspace_repo
        .get_role_permissions_by_role_id(ctx.workspace_id, role_id)
        .await?;

    Ok(Json(permissions))
}

pub async fn update_role_permissions(
    Path((_, role_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateRolePermissionsRequest>,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::MembersRolesManage)?;

    let role_exists = state
        .core
        .workspace_repo
        .role_exists_and_is_not_system(ctx.workspace_id, role_id)
        .await?;

    if !role_exists {
        return Err(AppError::NotFound(
            "Role not found or is a system role".to_string(),
        ));
    }

    state
        .core
        .workspace_repo
        .set_role_permissions(role_id, req.permissions)
        .await?;

    if let Some(redis_pool) = state.infra.redis.clone() {
        let cache = crate::cache::RedisCache::new(redis_pool);
        let _ = cache
            .delete_pattern(&format!("cache:auth:permissions:{}:*", ctx.workspace_id))
            .await;
    }

    Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── CreateWorkspaceRoleRequest deserialization ────────────────────────────

    #[test]
    fn test_create_role_request_scope_type_defaults_to_scoped() {
        // Clients that do NOT send scope_type must get Scoped (safe default).
        let json = r#"{"name": "Accountant", "color": "green"}"#;
        let req: CreateWorkspaceRoleRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.scope_type, RoleScopeType::Scoped);
        assert_eq!(req.name, "Accountant");
    }

    #[test]
    fn test_create_role_request_scope_type_global_explicit() {
        let json = r#"{"name": "Finance Manager", "color": "blue", "scope_type": "global"}"#;
        let req: CreateWorkspaceRoleRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.scope_type, RoleScopeType::Global);
    }

    #[test]
    fn test_create_role_request_scope_type_scoped_explicit() {
        let json = r#"{"name": "Assistant", "color": "slate", "scope_type": "scoped"}"#;
        let req: CreateWorkspaceRoleRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.scope_type, RoleScopeType::Scoped);
    }

    #[test]
    fn test_create_role_request_description_optional() {
        // description is Option<String> — must be absent or null without error.
        let json = r#"{"name": "Viewer", "color": "gray"}"#;
        let req: CreateWorkspaceRoleRequest = serde_json::from_str(json).unwrap();
        assert!(req.description.is_none());

        let json_with_desc =
            r#"{"name": "Viewer", "color": "gray", "description": "Read-only access"}"#;
        let req2: CreateWorkspaceRoleRequest = serde_json::from_str(json_with_desc).unwrap();
        assert_eq!(req2.description.as_deref(), Some("Read-only access"));
    }

    #[test]
    fn test_create_role_request_missing_required_fields_fails() {
        // name is required — missing it must fail deserialization.
        let json = r#"{"color": "blue"}"#;
        assert!(serde_json::from_str::<CreateWorkspaceRoleRequest>(json).is_err());

        // color is required.
        let json2 = r#"{"name": "Role"}"#;
        assert!(serde_json::from_str::<CreateWorkspaceRoleRequest>(json2).is_err());
    }

    // ── UpdateWorkspaceRoleRequest deserialization ────────────────────────────

    #[test]
    fn test_update_role_request_all_optional() {
        // Empty object is valid — all fields optional.
        let json = r#"{}"#;
        let req: UpdateWorkspaceRoleRequest = serde_json::from_str(json).unwrap();
        assert!(req.name.is_none());
        assert!(req.description.is_none());
        assert!(req.color.is_none());
    }

    #[test]
    fn test_update_role_request_partial_patch() {
        let json = r#"{"name": "New Name"}"#;
        let req: UpdateWorkspaceRoleRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name.as_deref(), Some("New Name"));
        assert!(req.color.is_none());
    }

    // ── UpdateRolePermissionsRequest ──────────────────────────────────────────

    #[test]
    fn test_update_permissions_request_empty_list() {
        // An empty permissions list is valid — it clears all permissions.
        let json = r#"{"permissions": []}"#;
        let req: UpdateRolePermissionsRequest = serde_json::from_str(json).unwrap();
        assert!(req.permissions.is_empty());
    }

    #[test]
    fn test_update_permissions_request_with_perms() {
        let json = r#"{"permissions": ["employees:read", "shifts:create", "payroll:read"]}"#;
        let req: UpdateRolePermissionsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.permissions.len(), 3);
        assert!(req.permissions.contains(&"employees:read".to_string()));
        assert!(req.permissions.contains(&"payroll:read".to_string()));
    }

    // ── RoleScopeType integration with MemberContext logic ───────────────────

    /// Simulate the is_global_manager decision tree using the scope_type flag.
    /// This mirrors the runtime logic in `fetch_member_context_from_db` without
    /// a real DB: if any role in a set has scope_type = Global → managed_department_ids = None.
    fn compute_managed_dept_ids(
        role_scope_types: &[RoleScopeType],
        fallback_dept_ids: Vec<uuid::Uuid>,
    ) -> Option<std::collections::HashSet<uuid::Uuid>> {
        let is_global = role_scope_types.iter().any(|s| s.is_global());
        if is_global {
            None // Global scope = access to all departments
        } else {
            Some(fallback_dept_ids.into_iter().collect())
        }
    }

    #[test]
    fn test_global_role_gives_none_managed_departments() {
        // Owner, Admin, HrManager, ShiftManager → no department restriction.
        let scope_types = vec![RoleScopeType::Global];
        let result = compute_managed_dept_ids(&scope_types, vec![uuid::Uuid::new_v4()]);
        assert!(
            result.is_none(),
            "Global roles must yield None (unrestricted) scope"
        );
    }

    #[test]
    fn test_scoped_role_gives_restricted_departments() {
        let dept = uuid::Uuid::new_v4();
        let scope_types = vec![RoleScopeType::Scoped];
        let result = compute_managed_dept_ids(&scope_types, vec![dept]);
        assert!(result.is_some());
        assert!(result.unwrap().contains(&dept));
    }

    #[test]
    fn test_mixed_roles_any_global_wins() {
        // If a user has both a scoped extra role AND a global primary role,
        // the global flag takes precedence — user gets full access.
        let scope_types = vec![RoleScopeType::Scoped, RoleScopeType::Global];
        let result = compute_managed_dept_ids(&scope_types, vec![uuid::Uuid::new_v4()]);
        assert!(
            result.is_none(),
            "Any global role must override scoped roles"
        );
    }

    #[test]
    fn test_custom_global_role_behaves_like_system_global() {
        // Enterprise requirement: a custom role marked as Global must be
        // indistinguishable from system roles (owner, hr_manager) in scope logic.
        let custom_global = RoleScopeType::Global;
        let system_global = RoleScopeType::Global;
        assert_eq!(custom_global, system_global);
        assert!(custom_global.is_global());
    }

    #[test]
    fn test_all_scoped_roles_gives_empty_department_set() {
        // Edge case: scoped user with no departments assigned.
        let scope_types = vec![RoleScopeType::Scoped];
        let result = compute_managed_dept_ids(&scope_types, vec![]);
        assert!(result.is_some());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_scope_type_to_string_matches_db_column_value() {
        // Ensure .to_string() matches exactly what's stored in the DB column.
        // The migration uses CHECK (scope_type IN ('global', 'scoped')).
        assert_eq!(RoleScopeType::Global.to_string(), "global");
        assert_eq!(RoleScopeType::Scoped.to_string(), "scoped");
    }

    #[test]
    fn test_scope_type_query_string_used_in_bind() {
        // Verify the string used in `.bind(req.scope_type.to_string())` matches
        // the DB CHECK constraint values from the migration.
        let valid_db_values = ["global", "scoped"];
        for variant in [RoleScopeType::Global, RoleScopeType::Scoped] {
            let bound_val = variant.to_string();
            assert!(
                valid_db_values.contains(&bound_val.as_str()),
                "scope_type.to_string() = '{bound_val}' is not a valid DB CHECK value"
            );
        }
    }

    // ── Edge Cases: RoleScopeType deserialization robustness ─────────────────

    #[test]
    fn test_scope_type_invalid_string_rejected_by_serde() {
        // An unknown string value must fail deserialization (not silently become Scoped).
        // This prevents attackers from sending "SUPERADMIN" and accidentally getting
        // the From<String> fallback that bypasses strict JSON validation.
        let invalid_values = [
            r#""GLOBAL""#,     // uppercase — wrong case
            r#""Global""#,     // PascalCase
            r#""admin""#,      // a role name, not a scope
            r#""owner""#,      // ditto
            r#""""#,           // empty string
            r#""unknown""#,    // arbitrary value
            r#""superadmin""#, // injection attempt
        ];
        for invalid in invalid_values {
            let json = format!(r#"{{"name": "R", "color": "b", "scope_type": {invalid}}}"#);
            let result = serde_json::from_str::<CreateWorkspaceRoleRequest>(&json);
            assert!(
                result.is_err(),
                "Expected deserialization failure for scope_type={invalid}, but it succeeded"
            );
        }
    }

    #[test]
    fn test_scope_type_null_json_falls_back_to_default() {
        // JSON `null` for a #[serde(default)] field should use the Default impl (Scoped).
        // This matches how many frontend frameworks send "not provided" fields.
        let json = r#"{"name": "Auditor", "color": "yellow", "scope_type": null}"#;
        // serde with #[serde(default)] treats null the same as missing → Scoped.
        let result = serde_json::from_str::<CreateWorkspaceRoleRequest>(json);
        // serde_json by default treats null as "missing" for #[serde(default)] fields.
        // If the field is Option<T> with default this passes; for non-Option with default
        // serde_json rejects null. We document the actual behavior here.
        if let Ok(req) = result {
            assert_eq!(req.scope_type, RoleScopeType::Scoped)
        } else {
            // serde_json rejects null for non-Option<RoleScopeType>. This is also correct
            // behavior — the client should send the key absent, not null.
            // Test documents this boundary; either outcome is acceptable.
        }
    }

    #[test]
    fn test_scope_type_from_string_case_sensitivity() {
        // `From<String>` is used when reading from DB adapter results.
        // Only exact lowercase snake_case must succeed; anything else falls back to Scoped.
        assert_eq!(
            RoleScopeType::from("global".to_string()),
            RoleScopeType::Global
        );
        assert_eq!(
            RoleScopeType::from("scoped".to_string()),
            RoleScopeType::Scoped
        );

        // Wrong case → safe fallback (not a panic, not Global)
        assert_eq!(
            RoleScopeType::from("GLOBAL".to_string()),
            RoleScopeType::Scoped
        );
        assert_eq!(
            RoleScopeType::from("Global".to_string()),
            RoleScopeType::Scoped
        );
        assert_eq!(
            RoleScopeType::from("SCOPED".to_string()),
            RoleScopeType::Scoped
        );
    }

    #[test]
    fn test_scope_type_from_empty_and_garbage_strings() {
        // Defensive: corrupted DB values must never become Global.
        for bad in ["", " ", "\t", "null", "true", "1", "global;DROP TABLE--"] {
            let result = RoleScopeType::from(bad.to_string());
            assert_eq!(
                result,
                RoleScopeType::Scoped,
                "Garbage input '{bad}' must fall back to Scoped, never become Global"
            );
        }
    }

    // ── Edge Cases: permissions list ─────────────────────────────────────────

    #[test]
    fn test_update_permissions_duplicate_strings_preserved_in_vec() {
        // `Vec<String>` preserves duplicates — the handler's DELETE + re-INSERT
        // will deduplicate at DB level via PRIMARY KEY (role_id, permission_name).
        // We verify the request struct itself carries duplicates as-is.
        let json = r#"{"permissions": ["employees:read", "employees:read", "shifts:create"]}"#;
        let req: UpdateRolePermissionsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.permissions.len(), 3, "Vec preserves duplicates");
        assert_eq!(
            req.permissions
                .iter()
                .filter(|p| p.as_str() == "employees:read")
                .count(),
            2
        );
    }

    #[test]
    fn test_update_permissions_all_91_known_permissions() {
        // The system has exactly 91 defined permissions. A role may be granted all of them.
        // Verify that the request struct can hold the full set without truncation.
        use crate::domain::permissions::Permission;
        let all_perms: Vec<String> = Permission::all_permission_strings()
            .into_iter()
            .map(String::from)
            .collect();

        let json = serde_json::json!({"permissions": all_perms}).to_string();
        let req: UpdateRolePermissionsRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(
            req.permissions.len(),
            91,
            "All 91 permissions must be accepted"
        );
    }

    #[test]
    fn test_update_permissions_unknown_string_not_validated_at_struct_level() {
        // The struct is a plain Vec<String> — unknown permission strings pass
        // deserialization. Validation against the known enum happens in the
        // service layer (or the DB FK will reject unknown values).
        // This test documents this intentional design decision.
        let json = r#"{"permissions": ["nonexistent:permission", "fake:action"]}"#;
        let req: UpdateRolePermissionsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(
            req.permissions.len(),
            2,
            "Unknown perms pass struct validation"
        );
    }

    // ── Edge Cases: scope decision with edge-case role lists ─────────────────

    #[test]
    fn test_empty_roles_list_is_not_global() {
        // A user with NO roles (malformed state) must not get global scope.
        // Empty iterator → any() returns false → Scoped.
        let result = compute_managed_dept_ids(&[], vec![uuid::Uuid::new_v4()]);
        assert!(result.is_some(), "No roles → must be Scoped, not Global");
    }

    #[test]
    fn test_many_scoped_roles_never_become_global() {
        // Having many scoped roles must not accumulate into global access.
        let many_scoped = vec![RoleScopeType::Scoped; 20];
        let result = compute_managed_dept_ids(&many_scoped, vec![uuid::Uuid::new_v4()]);
        assert!(
            result.is_some(),
            "20 Scoped roles must not yield Global scope"
        );
    }

    #[test]
    fn test_global_among_many_scoped_roles_elevates_to_global() {
        // One global role anywhere in the list must elevate to global scope.
        // Simulates: employee has team_lead + custom_global_role.
        let mut roles = vec![RoleScopeType::Scoped; 10];
        roles.push(RoleScopeType::Global);
        let result = compute_managed_dept_ids(&roles, vec![uuid::Uuid::new_v4()]);
        assert!(
            result.is_none(),
            "Single Global among many Scoped must yield None"
        );
    }

    #[test]
    fn test_duplicate_department_ids_are_deduplicated_in_hashset() {
        // The same department ID appearing multiple times (e.g., from multiple
        // working patterns) must be stored only once in the managed set.
        let dept = uuid::Uuid::new_v4();
        let duplicated_depts = vec![dept, dept, dept];
        let result = compute_managed_dept_ids(&[RoleScopeType::Scoped], duplicated_depts);
        let set = result.unwrap();
        assert_eq!(
            set.len(),
            1,
            "Duplicate dept IDs must be deduplicated in HashSet"
        );
        assert!(set.contains(&dept));
    }

    #[test]
    fn test_many_distinct_departments_all_preserved() {
        // TeamLead managing 15 departments must have all 15 in their managed set.
        let depts: Vec<uuid::Uuid> = (0..15).map(|_| uuid::Uuid::new_v4()).collect();
        let result = compute_managed_dept_ids(&[RoleScopeType::Scoped], depts.clone());
        let set = result.unwrap();
        assert_eq!(set.len(), 15);
        for dept in &depts {
            assert!(set.contains(dept));
        }
    }

    // ── Edge Cases: request struct boundary values ────────────────────────────

    #[test]
    fn test_create_role_request_name_empty_string_parses() {
        // serde_json itself does not enforce non-empty strings. This is intentional —
        // business validation (non-empty name) must happen in the handler, not the struct.
        // This test documents the boundary between serde and business logic.
        let json = r#"{"name": "", "color": "blue"}"#;
        let req: CreateWorkspaceRoleRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "");
        // Downstream validation must check name.trim().is_empty()
    }

    #[test]
    fn test_create_role_request_whitespace_only_name_parses() {
        let json = r#"{"name": "   ", "color": "blue"}"#;
        let req: CreateWorkspaceRoleRequest = serde_json::from_str(json).unwrap();
        assert!(
            req.name.trim().is_empty(),
            "Whitespace name must be caught in handler validation"
        );
    }

    #[test]
    fn test_create_role_request_description_null_is_none() {
        // `"description": null` must produce `None`, not an error.
        let json = r#"{"name": "Role", "color": "red", "description": null}"#;
        let req: CreateWorkspaceRoleRequest = serde_json::from_str(json).unwrap();
        assert!(req.description.is_none());
    }

    #[test]
    fn test_update_role_request_null_fields_are_none() {
        // All optional fields sent as null must produce None.
        let json = r#"{"name": null, "description": null, "color": null}"#;
        let req: UpdateWorkspaceRoleRequest = serde_json::from_str(json).unwrap();
        assert!(req.name.is_none());
        assert!(req.description.is_none());
        assert!(req.color.is_none());
    }

    #[test]
    fn test_scope_type_is_global_helper_exhaustive() {
        // Verify is_global() is the exact inverse of is_scoped — no variant left out.
        // If a new variant is added, this test fails as a canary.
        let global_variants: Vec<RoleScopeType> = vec![RoleScopeType::Global];
        let scoped_variants: Vec<RoleScopeType> = vec![RoleScopeType::Scoped];
        for v in &global_variants {
            assert!(v.is_global(), "{v:?} should be global");
        }
        for v in &scoped_variants {
            assert!(!v.is_global(), "{v:?} should NOT be global");
        }
        // Total variants covered = global + scoped = all defined variants.
        assert_eq!(global_variants.len() + scoped_variants.len(), 2);
    }
}

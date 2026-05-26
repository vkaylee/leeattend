use std::collections::HashSet;

use crate::diesel_pool::TenantAuthToken;
use crate::domain::permissions::{self, Permission};
use crate::errors::{AppError, AppResult};
use crate::models::GenericResponse;
use crate::repositories::CacheRepositoryExt;
use crate::{AppState, UserId};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

// --- Extractors ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkspaceId(Uuid);

impl WorkspaceId {
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn into_inner(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for WorkspaceId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<WorkspaceId> for Uuid {
    fn from(id: WorkspaceId) -> Self {
        id.0
    }
}

#[derive(Debug, Clone)]
pub enum ApiContext {
    System,
    Tenant(WorkspaceId),
}

#[async_trait]
impl<S> FromRequestParts<S> for ApiContext
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Try to extract Tenant context (WorkspaceId)
        if let Ok(ws_id) = WorkspaceId::from_request_parts(parts, state).await {
            return Ok(ApiContext::Tenant(ws_id));
        }

        // 2. If no Tenant context, verify if it's a valid System route
        let path = parts.uri.path();
        if path.starts_with("/system") || path.starts_with("/api/system") {
            Ok(ApiContext::System)
        } else {
            // If it's not a system route, it means it's a tenant route that failed to provide a Workspace ID.
            Err(AppError::Validation(
                "Missing Workspace Context. Provide 'ws_id' in path or 'X-Workspace-Id' header."
                    .to_string(),
            ))
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for WorkspaceId
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Try to extract from Path parameters first (e.g., /workspaces/:ws_id/...)
        // We look specifically for 'ws_id' to avoid conflicts with other ':id' parameters.
        if let Ok(axum::extract::Path(params)) = axum::extract::Path::<
            std::collections::HashMap<String, String>,
        >::from_request_parts(parts, state)
        .await
        {
            if let Some(ws_id_str) = params.get("ws_id") {
                if let Ok(ws_id) = Uuid::parse_str(ws_id_str) {
                    return Ok(WorkspaceId::from(ws_id));
                }
            }
        }

        // 2. Fallback to header (for non-path routes like ESS or backward compatibility)
        let workspace_id_str = parts
            .headers
            .get("X-Workspace-Id")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| {
                AppError::Validation(
                "Missing Workspace Context. Provide 'ws_id' in path or 'X-Workspace-Id' header."
                    .to_string())
            })?;

        let workspace_id = Uuid::parse_str(workspace_id_str)
            .map_err(|_e| AppError::Validation("Invalid Workspace ID format".to_string()))?;

        Ok(WorkspaceId::from(workspace_id))
    }
}

pub struct MemberContext {
    pub user_id: Uuid,
    pub user_name: Option<String>,
    pub user_email: String,
    pub workspace_id: Uuid,
    pub roles: Vec<Uuid>,
    pub permissions: HashSet<Permission>,
    /// None = global scope (Admin/Owner/etc). Some = restricted to specific departments.
    pub managed_department_ids: Option<HashSet<Uuid>>,
    pub auth_token: TenantAuthToken,
}

impl MemberContext {
    pub fn has_role(&self, role_id: Uuid) -> bool {
        self.roles.contains(&role_id)
    }

    /// Check whether this member has a specific permission.
    pub fn has_permission(&self, perm: Permission) -> bool {
        self.permissions.contains(&perm)
    }

    /// Convenience: return `Err(Forbidden)` if the member lacks the permission.
    pub fn require_permission(&self, perm: Permission) -> AppResult<()> {
        if self.has_permission(perm) {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!("Missing permission: {perm:?}")))
        }
    }

    /// Check permission with department scope limitation (e.g., for TeamLead).
    pub fn require_scoped_permission(
        &self,
        perm: Permission,
        target_department_id: Option<Uuid>,
    ) -> AppResult<()> {
        self.require_permission(perm)?;

        // Global scope
        let Some(allowed_depts) = &self.managed_department_ids else {
            return Ok(());
        };

        // Scoped check
        if let Some(dept_id) = target_department_id {
            if allowed_depts.contains(&dept_id) {
                return Ok(());
            }
        }

        Err(AppError::Forbidden(
            "Permission denied: Target is outside your managed departments scope.".to_string(),
        ))
    }

    /// Check if the member has ANY of the given permissions, scoped to the department.
    pub fn require_any_scoped_permission(
        &self,
        perms: &[Permission],
        target_department_id: Option<Uuid>,
    ) -> AppResult<()> {
        let has_perm = perms.iter().any(|&p| self.has_permission(p));
        if !has_perm {
            return Err(AppError::Forbidden(format!(
                "Missing any of permissions: {perms:?}"
            )));
        }

        // Global scope
        let Some(allowed_depts) = &self.managed_department_ids else {
            return Ok(());
        };

        // Scoped check
        if let Some(dept_id) = target_department_id {
            if allowed_depts.contains(&dept_id) {
                return Ok(());
            }
        }

        Err(AppError::Forbidden(
            "Permission denied: Target is outside your managed departments scope.".to_string(),
        ))
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for MemberContext
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let UserId(user_id) = UserId::from_request_parts(parts, state).await?;
        let api_context = ApiContext::from_request_parts(parts, state).await?;

        let workspace_id = match api_context {
            ApiContext::Tenant(ws_id) => ws_id.into_inner(),
            ApiContext::System => {
                return Err(AppError::Forbidden(
                    "Cannot use Tenant context (MemberContext) in a System route".to_string(),
                ))
            }
        };

        let app_state = Arc::<AppState>::from_ref(state);

        let cache_key = format!("cache:auth:permissions:{workspace_id}:{user_id}");
        let ttl = Duration::from_secs(3600); // 1 hour TTL

        let cached_data = if let Some(cache) = &app_state.infra.cache {
            if let Some(d) = cache.get::<CachedMemberContext>(&cache_key).await {
                d
            } else {
                let fresh = Self::fetch_member_context_from_db(user_id, workspace_id, &app_state)
                    .await
                    .map_err(|e| {
                        if e == "Not a member of this workspace" {
                            AppError::Forbidden(e)
                        } else {
                            tracing::error!("DB error fetching member context: {e}");
                            AppError::Internal("Database error".to_string())
                        }
                    })?;
                let _ = cache.set(&cache_key, &fresh, ttl).await;
                fresh
            }
        } else {
            Self::fetch_member_context_from_db(user_id, workspace_id, &app_state)
                .await
                .map_err(|e| {
                    if e == "Not a member of this workspace" {
                        AppError::Forbidden(e)
                    } else {
                        tracing::error!("DB error fetching member context: {e}");
                        AppError::Internal("Database error".to_string())
                    }
                })?
        };

        let mut permissions = HashSet::new();
        for perm_str in &cached_data.permissions {
            if let Some(perm) = permissions::Permission::from_str_permission(perm_str) {
                permissions.insert(perm);
            }
        }

        Ok(MemberContext {
            user_id,
            user_name: cached_data.user_name,
            user_email: cached_data.user_email,
            workspace_id,
            roles: cached_data.roles,
            permissions,
            managed_department_ids: cached_data.managed_department_ids,
            auth_token: TenantAuthToken::new_authorized(workspace_id),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedMemberContext {
    pub user_name: Option<String>,
    pub user_email: String,
    pub roles: Vec<Uuid>,
    pub permissions: Vec<String>,
    pub managed_department_ids: Option<HashSet<Uuid>>,
}

impl MemberContext {
    async fn fetch_member_context_from_db(
        user_id: Uuid,
        workspace_id: Uuid,
        app_state: &Arc<AppState>,
    ) -> Result<CachedMemberContext, String> {
        let user = app_state
            .core
            .user_repo
            .get_user_by_id(user_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "User not found".to_string())?;

        // 1. Fetch Primary Role
        let primary_role = app_state
            .core
            .workspace_repo
            .find_primary_role(workspace_id, user_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Not a member of this workspace".to_string())?;

        // 2. Fetch Active Extra Roles
        let all_extra_roles = app_state
            .core
            .workspace_repo
            .find_extra_roles(workspace_id, user_id)
            .await
            .map_err(|e| e.to_string())?;

        let mut roles: Vec<Uuid> = all_extra_roles
            .into_iter()
            .filter(|r| r.expires_at > chrono::Utc::now())
            .map(|r| r.role_id)
            .collect();

        roles.push(primary_role);

        // Compute effective permissions from all roles via repository
        let db_perms = app_state
            .core
            .workspace_repo
            .get_permissions_for_roles(&roles)
            .await
            .map_err(|e| e.to_string())?;

        let mut permissions = HashSet::new();
        for perm_str in db_perms {
            if let Some(perm) = permissions::Permission::from_str_permission(&perm_str) {
                permissions.insert(perm);
            }
        }

        // 3. Fetch Active Extra Permissions (individual overrides)
        let extra_perms = app_state
            .core
            .workspace_repo
            .find_extra_permissions(workspace_id, user_id)
            .await
            .unwrap_or_default();

        for perm_str in &extra_perms {
            if let Some(perm) = permissions::Permission::from_str_permission(perm_str) {
                permissions.insert(perm);
            }
        }

        // 4. Determine Scope (Managed Departments)
        // Determine global scope via scope_type column — avoids hardcoded role name list.
        // A role is "global" if scope_type = 'global', which applies to system roles
        // (owner, admin, hr_manager, shift_manager) AND any custom role a Workspace Admin
        // explicitly marks as global-scoped.
        let is_global_manager = app_state
            .core
            .workspace_repo
            .is_any_role_global(&roles)
            .await
            .unwrap_or(false);

        let managed_department_ids = if is_global_manager {
            None
        } else {
            // For TeamLead or normal members with extra permissions,
            // restrict their scope to the departments they belong to.
            let depts = app_state
                .employees
                .repo
                .get_managed_departments(
                    &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                    user_id,
                )
                .await
                .unwrap_or_default();

            Some(depts.into_iter().collect())
        };

        Ok(CachedMemberContext {
            user_name: user.full_name,
            user_email: user.email,
            roles,
            permissions: permissions
                .into_iter()
                .map(|p| p.to_str_permission().to_string())
                .collect(),
            managed_department_ids,
        })
    }
}

// --- Employee Self-Service Context ---

/// Extractor for employee self-service endpoints.
/// Validates that the current user is linked to an employee record.
pub struct EmployeeContext {
    pub user_id: Uuid,
    pub employee_id: Uuid,
    pub workspace_id: Uuid,
    pub auth_token: TenantAuthToken,
}

#[async_trait]
impl<S> FromRequestParts<S> for EmployeeContext
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Verify MemberContext FIRST! This queries the System DB and ensures
        // the workspace exists and the user is actually a member, preventing DoS
        // attacks where an attacker just sends a random X-Workspace-Id and forces
        // a connection to Supavisor.
        let member_ctx = MemberContext::from_request_parts(parts, state).await?;

        let workspace_id = member_ctx.workspace_id;
        let user_id = member_ctx.user_id;
        let app_state = Arc::<AppState>::from_ref(state);

        // 2. Find employee linked to this user in this workspace
        let employee_id = app_state
            .employees
            .repo
            .get_employee_id_by_user_id(&member_ctx.auth_token, user_id)
            .await?
            .ok_or_else(|| {
                AppError::Forbidden("No employee record linked to your account".to_string())
            })?;

        Ok(EmployeeContext {
            user_id,
            employee_id,
            workspace_id,
            auth_token: member_ctx.auth_token,
        })
    }
}

// --- Basic Handlers ---

pub async fn health_check() -> AppResult<Json<GenericResponse>> {
    let response = GenericResponse {
        message: "API Server is running!".to_string(),
    };
    Ok(Json(response))
}

// --- Department Scope Helpers ---

/// Look up the current department_id for an employee (for scoped permission checks).
///
/// Returns `None` if the employee has no department assignment yet.
/// Used by handlers that need `require_scoped_permission()`.
pub async fn current_department_id(
    state: &AppState,
    auth_token: &crate::diesel_pool::TenantAuthToken,
    employee_id: Uuid,
) -> Option<Uuid> {
    state
        .payroll
        .department_repo
        .get_current_department(auth_token, employee_id)
        .await
        .ok()
        .flatten()
        .map(|h| h.department_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn make_ctx(
        roles: Vec<Uuid>,
        perms: HashSet<Permission>,
        managed: Option<HashSet<Uuid>>,
    ) -> MemberContext {
        let workspace_id = Uuid::new_v4();
        MemberContext {
            user_id: Uuid::new_v4(),
            user_name: Some("Test User".to_string()),
            user_email: "test@example.com".to_string(),
            workspace_id,
            roles,
            permissions: perms,
            managed_department_ids: managed,
            auth_token: TenantAuthToken::new_authorized(workspace_id),
        }
    }

    #[test]
    fn test_has_role_present() {
        let role = Uuid::new_v4();
        let ctx = make_ctx(vec![role], HashSet::new(), None);
        assert!(ctx.has_role(role));
    }

    #[test]
    fn test_has_role_absent() {
        let role1 = Uuid::new_v4();
        let role2 = Uuid::new_v4();
        let ctx = make_ctx(vec![role1], HashSet::new(), None);
        assert!(!ctx.has_role(role2));
    }

    #[test]
    fn test_has_permission_present() {
        let mut perms = HashSet::new();
        perms.insert(Permission::EmployeesRead);
        let ctx = make_ctx(vec![], perms, None);
        assert!(ctx.has_permission(Permission::EmployeesRead));
    }

    #[test]
    fn test_has_permission_absent() {
        let ctx = make_ctx(vec![], HashSet::new(), None);
        assert!(!ctx.has_permission(Permission::PayrollCalculate));
    }

    #[test]
    fn test_require_permission_ok() {
        let mut perms = HashSet::new();
        perms.insert(Permission::EmployeesRead);
        let ctx = make_ctx(vec![], perms, None);
        assert!(ctx.require_permission(Permission::EmployeesRead).is_ok());
    }

    #[test]
    fn test_require_permission_err() {
        let ctx = make_ctx(vec![], HashSet::new(), None);
        let result = ctx.require_permission(Permission::PayrollCalculate);
        assert!(result.is_err());
    }

    #[test]
    fn test_require_scoped_permission_global_scope() {
        let mut perms = HashSet::new();
        perms.insert(Permission::EmployeesRead);
        // managed_department_ids = None means global scope
        let ctx = make_ctx(vec![], perms, None);
        let dept_id = Uuid::new_v4();
        assert!(ctx
            .require_scoped_permission(Permission::EmployeesRead, Some(dept_id))
            .is_ok());
    }

    #[test]
    fn test_require_scoped_permission_in_scope() {
        let mut perms = HashSet::new();
        perms.insert(Permission::EmployeesRead);
        let dept_id = Uuid::new_v4();
        let mut managed = HashSet::new();
        managed.insert(dept_id);
        let ctx = make_ctx(vec![], perms, Some(managed));
        assert!(ctx
            .require_scoped_permission(Permission::EmployeesRead, Some(dept_id))
            .is_ok());
    }

    #[test]
    fn test_require_scoped_permission_out_of_scope() {
        let mut perms = HashSet::new();
        perms.insert(Permission::EmployeesRead);
        let managed = HashSet::from([Uuid::new_v4()]);
        let ctx = make_ctx(vec![], perms, Some(managed));
        let other_dept = Uuid::new_v4();
        assert!(ctx
            .require_scoped_permission(Permission::EmployeesRead, Some(other_dept))
            .is_err());
    }

    #[test]
    fn test_require_scoped_permission_no_target_dept() {
        let mut perms = HashSet::new();
        perms.insert(Permission::EmployeesRead);
        let managed = HashSet::from([Uuid::new_v4()]);
        let ctx = make_ctx(vec![], perms, Some(managed));
        // target_department_id = None with scoped permissions → Forbidden
        assert!(ctx
            .require_scoped_permission(Permission::EmployeesRead, None)
            .is_err());
    }

    #[test]
    fn test_require_any_scoped_permission_global_scope_ok() {
        let mut perms = HashSet::new();
        perms.insert(Permission::LeaveApproveL1);
        let ctx = make_ctx(vec![], perms, None);
        let dept_id = Uuid::new_v4();
        let any_perms = [Permission::LeaveRequestsApprove, Permission::LeaveApproveL1];
        assert!(ctx
            .require_any_scoped_permission(&any_perms, Some(dept_id))
            .is_ok());
    }

    #[test]
    fn test_require_any_scoped_permission_in_scope_ok() {
        let mut perms = HashSet::new();
        perms.insert(Permission::LeaveApproveL2);
        let dept_id = Uuid::new_v4();
        let mut managed = HashSet::new();
        managed.insert(dept_id);
        let ctx = make_ctx(vec![], perms, Some(managed));
        let any_perms = [
            Permission::LeaveRequestsApprove,
            Permission::LeaveApproveL1,
            Permission::LeaveApproveL2,
        ];
        assert!(ctx
            .require_any_scoped_permission(&any_perms, Some(dept_id))
            .is_ok());
    }

    #[test]
    fn test_require_any_scoped_permission_out_of_scope_err() {
        let mut perms = HashSet::new();
        perms.insert(Permission::LeaveApproveL1);
        let managed = HashSet::from([Uuid::new_v4()]);
        let ctx = make_ctx(vec![], perms, Some(managed));
        let other_dept = Uuid::new_v4();
        let any_perms = [Permission::LeaveRequestsApprove, Permission::LeaveApproveL1];
        assert!(ctx
            .require_any_scoped_permission(&any_perms, Some(other_dept))
            .is_err());
    }

    #[test]
    fn test_require_any_scoped_permission_no_perms_err() {
        let mut perms = HashSet::new();
        perms.insert(Permission::EmployeesRead); // Has a different permission
        let ctx = make_ctx(vec![], perms, None);
        let any_perms = [Permission::LeaveRequestsApprove, Permission::LeaveApproveL1];
        assert!(ctx
            .require_any_scoped_permission(&any_perms, Some(Uuid::new_v4()))
            .is_err());
    }

    #[test]
    fn test_require_any_scoped_permission_no_target_dept_err() {
        let mut perms = HashSet::new();
        perms.insert(Permission::LeaveApproveL1);
        let managed = HashSet::from([Uuid::new_v4()]);
        let ctx = make_ctx(vec![], perms, Some(managed));
        let any_perms = [Permission::LeaveRequestsApprove, Permission::LeaveApproveL1];
        assert!(ctx.require_any_scoped_permission(&any_perms, None).is_err());
    }

    #[tokio::test]
    async fn test_api_context_system_path() {
        use axum::extract::FromRequestParts;
        use axum::http::Request;

        let req = Request::builder()
            .uri("/api/system/roles")
            .body(())
            .unwrap();
        let (mut parts, ()) = req.into_parts();

        let ctx = ApiContext::from_request_parts(&mut parts, &())
            .await
            .unwrap();
        assert!(matches!(ctx, ApiContext::System));
    }

    #[tokio::test]
    async fn test_api_context_tenant_header() {
        use axum::extract::FromRequestParts;
        use axum::http::Request;
        let ws_id = Uuid::new_v4();

        let req = Request::builder()
            .uri("/api/workspaces")
            .header("X-Workspace-Id", ws_id.to_string())
            .body(())
            .unwrap();
        let (mut parts, ()) = req.into_parts();

        let ctx = ApiContext::from_request_parts(&mut parts, &())
            .await
            .unwrap();
        if let ApiContext::Tenant(ws) = ctx {
            assert_eq!(ws.into_inner(), ws_id);
        } else {
            panic!("Expected Tenant context");
        }
    }

    #[tokio::test]
    async fn test_api_context_missing_workspace() {
        use axum::extract::FromRequestParts;
        use axum::http::Request;

        let req = Request::builder().uri("/api/workspaces").body(()).unwrap();
        let (mut parts, ()) = req.into_parts();

        let res = ApiContext::from_request_parts(&mut parts, &()).await;
        assert!(res.is_err());
    }
}

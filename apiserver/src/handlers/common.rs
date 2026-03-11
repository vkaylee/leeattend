use std::collections::HashSet;

use crate::domain::permissions::{self, Permission};
use crate::errors::{AppError, AppResult};
use crate::models::{GenericResponse, MemberRole};
use crate::{AppState, UserId};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

// --- Extractors ---

pub struct WorkspaceId(pub Uuid);

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
                    return Ok(WorkspaceId(ws_id));
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
            .map_err(|_| AppError::Validation("Invalid Workspace ID format".to_string()))?;

        Ok(WorkspaceId(workspace_id))
    }
}

pub struct MemberContext {
    pub user_id: Uuid,
    pub workspace_id: Uuid,
    pub roles: Vec<MemberRole>,
    pub permissions: HashSet<Permission>,
    /// None = global scope (Admin/Owner/etc). Some = restricted to specific departments.
    pub managed_department_ids: Option<HashSet<Uuid>>,
}

impl MemberContext {
    pub fn has_role(&self, role: MemberRole) -> bool {
        self.roles.contains(&role)
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
            Err(AppError::Forbidden(format!(
                "Missing permission: {perm:?}"
            )))
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
        let allowed_depts = match &self.managed_department_ids {
            None => return Ok(()),
            Some(depts) => depts,
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

    /// Backward-compatible check. Prefer `has_permission()` for new code.
    pub fn is_admin_or_owner(&self) -> bool {
        self.has_role(MemberRole::Admin) || self.has_role(MemberRole::Owner)
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
        let WorkspaceId(workspace_id) = WorkspaceId::from_request_parts(parts, state).await?;

        let app_state = Arc::<AppState>::from_ref(state);

        // 1. Fetch Primary Role
        let primary_role = app_state
            .core.workspace_repo
            .find_primary_role(workspace_id, user_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::Forbidden("Not a member of this workspace".to_string()))?;

        // 2. Fetch Active Extra Roles
        let all_extra_roles = app_state
            .core.workspace_repo
            .find_extra_roles(workspace_id, user_id)
            .await
            .map_err(AppError::from)?;

        let mut roles: Vec<MemberRole> = all_extra_roles
            .into_iter()
            .filter(|r| r.expires_at > chrono::Utc::now())
            .map(|r| r.role)
            .collect();

        roles.push(primary_role);

        // Compute effective permissions from all roles
        let mut permissions = permissions::effective_permissions(&roles);

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
        let is_global_manager = roles.iter().any(|r| {
            matches!(r, MemberRole::Owner | MemberRole::Admin | MemberRole::HrManager | MemberRole::ShiftManager)
        });

        let managed_department_ids = if is_global_manager {
            None
        } else {
            // For TeamLead or normal members with extra permissions,
            // restrict their scope to the departments they belong to.
            let depts = app_state
                .employees
                .repo
                .get_managed_departments(workspace_id, user_id)
                .await
                .unwrap_or_default();

            Some(depts.into_iter().collect())
        };

        Ok(MemberContext {
            user_id,
            workspace_id,
            roles,
            permissions,
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
}

#[async_trait]
impl<S> FromRequestParts<S> for EmployeeContext
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let UserId(user_id) = UserId::from_request_parts(parts, state).await?;
        let WorkspaceId(workspace_id) = WorkspaceId::from_request_parts(parts, state).await?;

        let app_state = Arc::<AppState>::from_ref(state);

        // Find employee linked to this user in this workspace
        let employee_id = app_state
            .employees
            .repo
            .get_employee_id_by_user_id(workspace_id, user_id)
            .await
            .map_err(AppError::Sqlx)?
            .ok_or_else(|| {
                AppError::Forbidden("No employee record linked to your account".to_string())
            })?;

        Ok(EmployeeContext {
            user_id,
            employee_id,
            workspace_id,
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
    workspace_id: Uuid,
    employee_id: Uuid,
) -> Option<Uuid> {
    state
        .payroll
        .department_repo
        .get_current_department(workspace_id, employee_id)
        .await
        .ok()
        .flatten()
        .map(|h| h.department_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn make_ctx(roles: Vec<MemberRole>, perms: HashSet<Permission>, managed: Option<HashSet<Uuid>>) -> MemberContext {
        MemberContext {
            user_id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            roles,
            permissions: perms,
            managed_department_ids: managed,
        }
    }

    #[test]
    fn test_has_role_present() {
        let ctx = make_ctx(vec![MemberRole::Admin], HashSet::new(), None);
        assert!(ctx.has_role(MemberRole::Admin));
    }

    #[test]
    fn test_has_role_absent() {
        let ctx = make_ctx(vec![MemberRole::Member], HashSet::new(), None);
        assert!(!ctx.has_role(MemberRole::Admin));
    }

    #[test]
    fn test_has_permission_present() {
        let mut perms = HashSet::new();
        perms.insert(Permission::EmployeesRead);
        let ctx = make_ctx(vec![MemberRole::Member], perms, None);
        assert!(ctx.has_permission(Permission::EmployeesRead));
    }

    #[test]
    fn test_has_permission_absent() {
        let ctx = make_ctx(vec![MemberRole::Member], HashSet::new(), None);
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
        assert!(ctx.require_scoped_permission(Permission::EmployeesRead, Some(dept_id)).is_ok());
    }

    #[test]
    fn test_require_scoped_permission_in_scope() {
        let mut perms = HashSet::new();
        perms.insert(Permission::EmployeesRead);
        let dept_id = Uuid::new_v4();
        let mut managed = HashSet::new();
        managed.insert(dept_id);
        let ctx = make_ctx(vec![], perms, Some(managed));
        assert!(ctx.require_scoped_permission(Permission::EmployeesRead, Some(dept_id)).is_ok());
    }

    #[test]
    fn test_require_scoped_permission_out_of_scope() {
        let mut perms = HashSet::new();
        perms.insert(Permission::EmployeesRead);
        let managed = HashSet::from([Uuid::new_v4()]);
        let ctx = make_ctx(vec![], perms, Some(managed));
        let other_dept = Uuid::new_v4();
        assert!(ctx.require_scoped_permission(Permission::EmployeesRead, Some(other_dept)).is_err());
    }

    #[test]
    fn test_require_scoped_permission_no_target_dept() {
        let mut perms = HashSet::new();
        perms.insert(Permission::EmployeesRead);
        let managed = HashSet::from([Uuid::new_v4()]);
        let ctx = make_ctx(vec![], perms, Some(managed));
        // target_department_id = None with scoped permissions → Forbidden
        assert!(ctx.require_scoped_permission(Permission::EmployeesRead, None).is_err());
    }

    #[test]
    fn test_is_admin_or_owner_true() {
        let ctx_admin = make_ctx(vec![MemberRole::Admin], HashSet::new(), None);
        assert!(ctx_admin.is_admin_or_owner());
        let ctx_owner = make_ctx(vec![MemberRole::Owner], HashSet::new(), None);
        assert!(ctx_owner.is_admin_or_owner());
    }

    #[test]
    fn test_is_admin_or_owner_false() {
        for role in [MemberRole::HrManager, MemberRole::ShiftManager, MemberRole::TeamLead, MemberRole::Member, MemberRole::Employee] {
            let ctx = make_ctx(vec![role], HashSet::new(), None);
            assert!(!ctx.is_admin_or_owner(), "Expected false for role {:?}", role);
        }
    }
}

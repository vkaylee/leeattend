use std::collections::HashSet;

use crate::diesel_pool::TenantAuthToken;
use crate::domain::permissions::{self, Permission};
use crate::errors::{AppError, AppResult};
use crate::models::GenericResponse;
use crate::repositories::CacheRepositoryExt;
use crate::utils::i18n::I18nKey;
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
        if path.starts_with("/system") || path.starts_with("/api/v1/system") {
            Ok(ApiContext::System)
        } else {
            // If it's not a system route, it means it's a tenant route that failed to provide a Workspace ID.
            Err(AppError::validation(I18nKey::MissingWorkspace))
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
            .ok_or_else(|| AppError::validation(I18nKey::MissingWorkspace))?;

        let workspace_id = Uuid::parse_str(workspace_id_str)
            .map_err(|_e| AppError::validation(I18nKey::InvalidWorkspaceId))?;

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
            tracing::warn!(user_id = ?self.user_id, missing_permission = ?perm, "Permission denied");
            Err(AppError::forbidden(I18nKey::Forbidden))
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

        Err(AppError::forbidden(I18nKey::Forbidden))
    }

    /// Check if the member has ANY of the given permissions, scoped to the department.
    pub fn require_any_scoped_permission(
        &self,
        perms: &[Permission],
        target_department_id: Option<Uuid>,
    ) -> AppResult<()> {
        let has_perm = perms.iter().any(|&p| self.has_permission(p));
        if !has_perm {
            tracing::warn!(user_id = ?self.user_id, missing_any_permissions = ?perms, "Permission denied");
            return Err(AppError::forbidden(I18nKey::Forbidden));
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

        Err(AppError::forbidden(I18nKey::Forbidden))
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
                return Err(AppError::forbidden(I18nKey::SystemRouteTenantContext))
            }
        };

        let app_state = Arc::<AppState>::from_ref(state);

        let cache_key = format!("cache:auth:permissions:{workspace_id}:{user_id}");
        let ttl = Duration::from_secs(3600); // 1 hour TTL

        let cached_data = if let Some(cache) = app_state.infra().cache() {
            if let Some(d) = cache.get::<CachedMemberContext>(&cache_key).await {
                d
            } else {
                let fresh =
                    Self::fetch_member_context_from_db(user_id, workspace_id, &app_state).await?;
                let _ = cache.set(&cache_key, &fresh, ttl).await;
                fresh
            }
        } else {
            Self::fetch_member_context_from_db(user_id, workspace_id, &app_state).await?
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
    ) -> Result<CachedMemberContext, AppError> {
        let user = app_state
            .core()
            .user_repo()
            .get_user_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::auth(I18nKey::UserNotFound))?;

        // 1. Fetch Primary Role
        let primary_role = app_state
            .core()
            .workspace_repo()
            .find_primary_role(workspace_id, user_id)
            .await?
            .ok_or_else(|| AppError::forbidden(I18nKey::NotWorkspaceMember))?;

        // 2. Fetch Active Extra Roles
        let all_extra_roles = app_state
            .core()
            .workspace_repo()
            .find_extra_roles(workspace_id, user_id)
            .await?;

        let mut roles: Vec<Uuid> = all_extra_roles
            .into_iter()
            .filter(|r| r.expires_at() > chrono::Utc::now())
            .map(|r| r.role_id())
            .collect();

        roles.push(primary_role);

        // Compute effective permissions from all roles via repository
        let db_perms = app_state
            .core()
            .workspace_repo()
            .get_permissions_for_roles(&roles)
            .await?;

        let mut permissions = HashSet::new();
        for perm_str in db_perms {
            if let Some(perm) = permissions::Permission::from_str_permission(&perm_str) {
                permissions.insert(perm);
            }
        }

        // 3. Fetch Active Extra Permissions (individual overrides)
        let extra_perms = app_state
            .core()
            .workspace_repo()
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
            .core()
            .workspace_repo()
            .is_any_role_global(&roles)
            .await
            .unwrap_or(false);

        let managed_department_ids = if is_global_manager {
            None
        } else {
            // For TeamLead or normal members with extra permissions,
            // restrict their scope to the departments they belong to.
            let depts = app_state
                .employees()
                .repo()
                .get_managed_departments(
                    &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                    user_id,
                )
                .await
                .unwrap_or_default();

            Some(depts.into_iter().collect())
        };

        Ok(CachedMemberContext {
            user_name: user.full_name().map(String::from),
            user_email: user.email().to_string(),
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
            .employees()
            .repo()
            .get_employee_id_by_user_id(&member_ctx.auth_token, user_id)
            .await?
            .ok_or_else(|| AppError::forbidden(I18nKey::NoEmployeeRecord))?;

        Ok(EmployeeContext {
            user_id,
            employee_id,
            workspace_id,
            auth_token: member_ctx.auth_token,
        })
    }
}

// --- Basic Handlers ---

static START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

pub async fn health_check() -> AppResult<Json<GenericResponse>> {
    let response = GenericResponse {
        message: "API Server is running!".to_string(),
    };
    Ok(Json(response))
}

pub async fn readiness_check(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> impl axum::response::IntoResponse {
    use diesel_async::SimpleAsyncConnection;

    let mut db_status = "ok";
    let mut redis_status = "ok";
    let mut has_error = false;

    // Check DB
    match state.infra().diesel().api.get().await {
        Ok(mut conn) => {
            if let Err(e) = conn.batch_execute("SELECT 1").await {
                tracing::error!("Readiness DB ping failed: {:?}", e);
                db_status = "error";
                has_error = true;
            }
        }
        Err(e) => {
            tracing::error!("Readiness DB connection failed: {:?}", e);
            db_status = "error";
            has_error = true;
        }
    }

    // Check Redis (if enabled)
    if let Some(redis) = state.infra().redis() {
        if let Err(e) = redis.health_check().await {
            tracing::error!("Readiness Redis health check failed: {:?}", e);
            redis_status = "error";
            has_error = true;
        }
    } else {
        redis_status = "disabled";
    }

    let status = if has_error { "unhealthy" } else { "healthy" };
    let start = *START_TIME.get_or_init(std::time::Instant::now);
    let uptime_seconds = start.elapsed().as_secs();
    let version = env!("CARGO_PKG_VERSION");

    let body = serde_json::json!({
        "status": status,
        "version": version,
        "uptime_seconds": uptime_seconds,
        "checks": {
            "database": db_status,
            "redis": redis_status,
        }
    });

    let status_code = if has_error {
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    } else {
        axum::http::StatusCode::OK
    };

    (status_code, Json(body))
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
        .payroll()
        .department_repo()
        .get_current_department(auth_token, employee_id)
        .await
        .ok()
        .flatten()
        .map(|h| h.department_id())
}

// --- Behalf Scope Helpers ---

/// Generic behalf scope check — replaces domain-specific copies
/// in leaves.rs, attendance/overtime.rs, attendance/adjustments.rs.
///
/// Validates that the caller has permission to act on behalf of the target employee:
/// 1. Caller must have at least one of `required_permissions`
/// 2. If caller has global scope (managed_department_ids = None) → Ok
/// 3. Check explicit behalf_assignment for the specific `domain_type`
/// 4. Fallback: check if target is in caller's managed departments
pub async fn check_behalf_scope(
    state: &AppState,
    ctx: &MemberContext,
    required_permissions: &[Permission],
    domain_type: &str,
    target_employee_id: Uuid,
) -> AppResult<()> {
    let has_perm = required_permissions.iter().any(|&p| ctx.has_permission(p));
    if !has_perm {
        return Err(AppError::forbidden_args(
            I18nKey::Forbidden,
            vec![(
                "details",
                format!("Missing permission: one of {required_permissions:?}"),
            )],
        ));
    }

    // Global scope — no department restriction
    if ctx.managed_department_ids.is_none() {
        return Ok(());
    }

    let target_dept_id = current_department_id(state, &ctx.auth_token, target_employee_id).await;

    // Check explicit behalf assignment
    let is_assigned = state
        .attendance()
        .behalf_assignment_repo()
        .check_behalf_permission(
            ctx.workspace_id,
            ctx.user_id,
            target_employee_id,
            target_dept_id,
            domain_type,
        )
        .await?;

    if is_assigned {
        return Ok(());
    }

    // Fallback: managed department check
    if let Some(allowed_depts) = &ctx.managed_department_ids {
        if let Some(dept_id) = target_dept_id {
            if allowed_depts.contains(&dept_id) {
                return Ok(());
            }
        }
    }

    Err(AppError::forbidden_args(
        I18nKey::Forbidden,
        vec![(
            "details",
            "Permission denied: Target is outside your on-behalf scope.".to_string(),
        )],
    ))
}

/// Generic handler: create a behalf assignment for any domain type.
pub async fn create_behalf_assignment_handler(
    state: &AppState,
    ctx: &MemberContext,
    req: &crate::models::CreateBehalfAssignmentRequest,
    required_permission: Permission,
    domain_type: &str,
) -> AppResult<crate::models::BehalfAssignment> {
    ctx.require_permission(required_permission)?;

    state
        .attendance()
        .behalf_assignment_repo()
        .create_assignment(
            ctx.workspace_id,
            req.assignee_user_id,
            req.target_employee_id,
            req.target_department_id,
            domain_type,
            req.employer_id,
        )
        .await
}

/// Generic handler: delete a behalf assignment for any domain type.
pub async fn delete_behalf_assignment_handler(
    state: &AppState,
    ctx: &MemberContext,
    assignment_id: Uuid,
    required_permission: Permission,
) -> AppResult<()> {
    ctx.require_permission(required_permission)?;

    let affected = state
        .attendance()
        .behalf_assignment_repo()
        .delete_assignment(ctx.workspace_id, assignment_id)
        .await?;

    if affected == 0 {
        return Err(AppError::not_found(I18nKey::NotFound));
    }

    Ok(())
}

/// Generic handler: list all behalf assignments for a domain type.
pub async fn list_behalf_assignments_handler(
    state: &AppState,
    ctx: &MemberContext,
    required_permission: Permission,
    domain_type: &str,
    page: Option<i64>,
    limit: Option<i64>,
) -> AppResult<(Vec<crate::models::BehalfAssignment>, i64)> {
    ctx.require_permission(required_permission)?;

    state
        .attendance()
        .behalf_assignment_repo()
        .list_all_assignments(ctx.workspace_id, domain_type, page, limit)
        .await
}

/// Generic handler: resolve the list of **target employees** that the current
/// user is allowed to act on behalf of, for a given domain type.
///
/// 1. Find the caller's employee_id from user_id
/// 2. Fetch behalf assignments where assignee = caller
/// 3. Resolve `target_employee_id` → Employee records
/// 4. Resolve `target_department_id` → all employees currently in that department
/// 5. Deduplicate and return
pub async fn get_my_behalf_targets_handler(
    state: &AppState,
    ctx: &MemberContext,
    domain_type: &str,
) -> AppResult<Vec<crate::models::Employee>> {
    let assignments = state
        .attendance()
        .behalf_assignment_repo()
        .list_assignments_for_assignee(ctx.workspace_id, ctx.user_id, domain_type)
        .await?;

    if assignments.is_empty() {
        return Ok(vec![]);
    }

    // Resolve department → employee IDs if any exist
    let has_dept = assignments
        .iter()
        .any(|a| a.target_department_id().is_some());

    let dept_map = if has_dept {
        let workspace_tz = state
            .core()
            .workspace_repo()
            .get_workspace_by_id(ctx.workspace_id)
            .await?
            .map(|ws| crate::utils::tz::parse_workspace_tz(ws.timezone()))
            .unwrap_or(chrono_tz::UTC);
        let today = crate::utils::tz::utc_now_date_in_tz(&workspace_tz);
        state
            .employees()
            .repo()
            .get_employee_departments_at(&ctx.auth_token, today)
            .await?
    } else {
        std::collections::HashMap::new()
    };

    let all_employee_ids = resolve_behalf_target_ids(&assignments, &dept_map);

    if all_employee_ids.is_empty() {
        return Ok(vec![]);
    }

    let ids_vec: Vec<Uuid> = all_employee_ids.into_iter().collect();
    let employees = state
        .employees()
        .repo()
        .get_employees_by_ids(&ids_vec, &ctx.auth_token)
        .await?;

    // Filter out inactive employees
    let active: Vec<crate::models::Employee> = employees
        .into_iter()
        .filter(crate::models::Employee::is_active)
        .collect();

    Ok(active)
}

/// Pure function: resolve behalf assignments into a deduplicated set of employee IDs.
///
/// - Collects `target_employee_id` from direct employee assignments
/// - Resolves `target_department_id` via the `dept_map` (employee → (dept_id, dept_name))
/// - Deduplicates using `HashSet`
pub(crate) fn resolve_behalf_target_ids(
    assignments: &[crate::models::BehalfAssignment],
    dept_map: &std::collections::HashMap<Uuid, (Option<Uuid>, String)>,
) -> std::collections::HashSet<Uuid> {
    let mut all_employee_ids: std::collections::HashSet<Uuid> = assignments
        .iter()
        .filter_map(crate::models::BehalfAssignment::target_employee_id)
        .collect();

    let dept_ids: std::collections::HashSet<Uuid> = assignments
        .iter()
        .filter_map(crate::models::BehalfAssignment::target_department_id)
        .collect();

    if !dept_ids.is_empty() {
        for (emp_id, (dept_id_opt, _dept_name)) in dept_map {
            if let Some(dept_id) = dept_id_opt {
                if dept_ids.contains(dept_id) {
                    all_employee_ids.insert(*emp_id);
                }
            }
        }
    }

    all_employee_ids
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
            .uri("/api/v1/system/roles")
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
            .uri("/api/v1/workspaces")
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

        let req = Request::builder()
            .uri("/api/v1/workspaces")
            .body(())
            .unwrap();
        let (mut parts, ()) = req.into_parts();

        let res = ApiContext::from_request_parts(&mut parts, &()).await;
        assert!(res.is_err());
    }

    // ── resolve_behalf_target_ids tests ──────────────────────────────────

    fn make_assignment(
        target_employee_id: Option<Uuid>,
        target_department_id: Option<Uuid>,
    ) -> crate::models::BehalfAssignment {
        crate::models::BehalfAssignment::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            target_employee_id,
            target_department_id,
            "leave_request".to_string(),
            chrono::Utc::now(),
            chrono::Utc::now(),
            None,
        )
    }

    #[test]
    fn test_resolve_empty_assignments() {
        let dept_map = std::collections::HashMap::new();
        let result = resolve_behalf_target_ids(&[], &dept_map);
        assert!(result.is_empty());
    }

    #[test]
    fn test_resolve_direct_employee_targets() {
        let emp_a = Uuid::new_v4();
        let emp_b = Uuid::new_v4();
        let assignments = vec![
            make_assignment(Some(emp_a), None),
            make_assignment(Some(emp_b), None),
        ];
        let dept_map = std::collections::HashMap::new();

        let result = resolve_behalf_target_ids(&assignments, &dept_map);

        assert_eq!(result.len(), 2);
        assert!(result.contains(&emp_a));
        assert!(result.contains(&emp_b));
    }

    #[test]
    fn test_resolve_department_targets() {
        let dept_x = Uuid::new_v4();
        let emp_in_dept = Uuid::new_v4();
        let emp_not_in_dept = Uuid::new_v4();

        let assignments = vec![make_assignment(None, Some(dept_x))];

        let mut dept_map = std::collections::HashMap::new();
        dept_map.insert(emp_in_dept, (Some(dept_x), "Sales".to_string()));
        dept_map.insert(
            emp_not_in_dept,
            (Some(Uuid::new_v4()), "Engineering".to_string()),
        );

        let result = resolve_behalf_target_ids(&assignments, &dept_map);

        assert_eq!(result.len(), 1);
        assert!(result.contains(&emp_in_dept));
        assert!(!result.contains(&emp_not_in_dept));
    }

    #[test]
    fn test_resolve_mixed_direct_and_department() {
        let emp_direct = Uuid::new_v4();
        let dept_x = Uuid::new_v4();
        let emp_in_dept = Uuid::new_v4();

        let assignments = vec![
            make_assignment(Some(emp_direct), None),
            make_assignment(None, Some(dept_x)),
        ];

        let mut dept_map = std::collections::HashMap::new();
        dept_map.insert(emp_in_dept, (Some(dept_x), "HR".to_string()));

        let result = resolve_behalf_target_ids(&assignments, &dept_map);

        assert_eq!(result.len(), 2);
        assert!(result.contains(&emp_direct));
        assert!(result.contains(&emp_in_dept));
    }

    #[test]
    fn test_resolve_deduplicates_overlapping_targets() {
        let emp_overlap = Uuid::new_v4();
        let dept_x = Uuid::new_v4();

        // Same employee is both a direct target AND in the target department
        let assignments = vec![
            make_assignment(Some(emp_overlap), None),
            make_assignment(None, Some(dept_x)),
        ];

        let mut dept_map = std::collections::HashMap::new();
        dept_map.insert(emp_overlap, (Some(dept_x), "Sales".to_string()));

        let result = resolve_behalf_target_ids(&assignments, &dept_map);

        // Should be 1, not 2 (deduplicated)
        assert_eq!(result.len(), 1);
        assert!(result.contains(&emp_overlap));
    }

    #[test]
    fn test_resolve_dept_target_skips_employees_without_department() {
        let dept_x = Uuid::new_v4();
        let emp_no_dept = Uuid::new_v4();

        let assignments = vec![make_assignment(None, Some(dept_x))];

        let mut dept_map = std::collections::HashMap::new();
        // Employee has no department (None)
        dept_map.insert(emp_no_dept, (None, "Unassigned".to_string()));

        let result = resolve_behalf_target_ids(&assignments, &dept_map);

        assert!(result.is_empty());
    }

    #[test]
    fn test_resolve_multiple_departments() {
        let dept_a = Uuid::new_v4();
        let dept_b = Uuid::new_v4();
        let emp_in_a = Uuid::new_v4();
        let emp_in_b = Uuid::new_v4();
        let emp_in_c = Uuid::new_v4();

        let assignments = vec![
            make_assignment(None, Some(dept_a)),
            make_assignment(None, Some(dept_b)),
        ];

        let mut dept_map = std::collections::HashMap::new();
        dept_map.insert(emp_in_a, (Some(dept_a), "Sales".to_string()));
        dept_map.insert(emp_in_b, (Some(dept_b), "Engineering".to_string()));
        dept_map.insert(emp_in_c, (Some(Uuid::new_v4()), "Marketing".to_string()));

        let result = resolve_behalf_target_ids(&assignments, &dept_map);

        assert_eq!(result.len(), 2);
        assert!(result.contains(&emp_in_a));
        assert!(result.contains(&emp_in_b));
        assert!(!result.contains(&emp_in_c));
    }
}

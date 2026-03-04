use crate::errors::{AppError, AppResult};
use crate::models::{GenericResponse, MemberRole};
use crate::{AppState, UserId};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    Json,
};
use sqlx;
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
}

impl MemberContext {
    pub fn has_role(&self, role: MemberRole) -> bool {
        self.roles.contains(&role)
    }

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
            .workspace_repo
            .find_primary_role(workspace_id, user_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::Forbidden("Not a member of this workspace".to_string()))?;

        // 2. Fetch Active Extra Roles
        let all_extra_roles = app_state
            .workspace_repo
            .find_extra_roles(workspace_id, user_id)
            .await
            .map_err(AppError::from)?;

        let mut roles: Vec<MemberRole> = all_extra_roles
            .into_iter()
            .filter(|r| r.expires_at > chrono::Utc::now())
            .map(|r| r.role)
            .collect();

        roles.push(primary_role);

        Ok(MemberContext {
            user_id,
            workspace_id,
            roles,
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
        let employee = sqlx::query_as::<_, (Uuid,)>(
            r#"
            SELECT id FROM employees
            WHERE user_id = $1 AND workspace_id = $2 AND is_active = true
            "#,
        )
        .bind(user_id)
        .bind(workspace_id)
        .fetch_optional(app_state.db.api())
        .await
        .map_err(AppError::Sqlx)?
        .ok_or_else(|| {
            AppError::Forbidden("No employee record linked to your account".to_string())
        })?;

        Ok(EmployeeContext {
            user_id,
            employee_id: employee.0,
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

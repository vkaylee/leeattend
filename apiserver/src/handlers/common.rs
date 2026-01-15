use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use crate::{AppState, UserId};
use crate::models::GenericResponse;

// --- Extractors ---

pub struct WorkspaceId(pub Uuid);

#[async_trait]
impl<S> FromRequestParts<S> for WorkspaceId
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let workspace_id_str = parts
            .headers
            .get("X-Workspace-Id")
            .and_then(|h| h.to_str().ok())
            .ok_or((
                StatusCode::BAD_REQUEST,
                "Missing X-Workspace-Id header".to_string(),
            ))?;

        let workspace_id = Uuid::parse_str(workspace_id_str).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Invalid X-Workspace-Id format".to_string(),
            )
        })?;

        Ok(WorkspaceId(workspace_id))
    }
}

pub struct MemberContext {
    pub user_id: Uuid,
    pub workspace_id: Uuid,
    pub roles: Vec<String>,
}

impl MemberContext {
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    pub fn is_admin_or_owner(&self) -> bool {
        self.has_role("admin") || self.has_role("owner")
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for MemberContext
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let UserId(user_id) = UserId::from_request_parts(parts, state).await?;
        let WorkspaceId(workspace_id) = WorkspaceId::from_request_parts(parts, state).await?;

        let app_state = Arc::<AppState>::from_ref(state);

        // 1. Fetch Primary Role
        let primary_role = app_state
            .workspace_repo
            .find_primary_role(workspace_id, user_id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
            .ok_or((
                StatusCode::FORBIDDEN,
                "Not a member of this workspace".to_string(),
            ))?;

        // 2. Fetch Active Extra Roles
        let all_extra_roles = app_state
            .workspace_repo
            .find_extra_roles(workspace_id, user_id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

        let mut roles: Vec<String> = all_extra_roles
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

// --- Basic Handlers ---

pub async fn health_check() -> impl IntoResponse {
    let response = GenericResponse {
        message: "API Server is running!".to_string(),
    };
    Json(response)
}

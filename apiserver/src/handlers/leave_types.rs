use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, put},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{LeaveType, LeaveTypeConfig};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateLeaveTypeRequest {
    pub code: String,
    pub name: String,
    pub is_paid: bool,
    pub policy_config: Option<LeaveTypeConfig>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLeaveTypeConfigRequest {
    pub policy_config: LeaveTypeConfig,
}

pub async fn list_leave_types(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<LeaveType>>> {
    let types = state
        .leaves
        .policy_repo
        .get_leave_types(workspace_id)
        .await?;
    Ok(Json(types))
}

pub async fn create_leave_type(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateLeaveTypeRequest>,
) -> AppResult<(StatusCode, Json<LeaveType>)> {
    ctx.require_permission(Permission::LeaveTypesManage)?;

    let config = req.policy_config.unwrap_or_default();
    let lt = state
        .leaves
        .policy_repo
        .create_leave_type(ctx.workspace_id, req.code, req.name, req.is_paid, config)
        .await?;

    Ok((StatusCode::CREATED, Json(lt)))
}

pub async fn update_leave_type_config(
    State(state): State<Arc<AppState>>,
    Path((_, id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<UpdateLeaveTypeConfigRequest>,
) -> AppResult<Json<LeaveType>> {
    ctx.require_permission(Permission::LeaveTypesManage)?;

    let lt = state
        .leaves
        .policy_repo
        .update_leave_type_config(id, ctx.workspace_id, req.policy_config)
        .await?;

    Ok(Json(lt))
}

pub async fn delete_leave_type(
    State(state): State<Arc<AppState>>,
    Path((_, id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::LeaveTypesManage)?;

    let affected = state
        .leaves
        .policy_repo
        .delete_leave_type(id, ctx.workspace_id)
        .await?;

    if affected > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Leave type not found".to_string()))
    }
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_leave_types).post(create_leave_type))
        .route("/:id", delete(delete_leave_type))
        .route("/:id/config", put(update_leave_type_config))
}

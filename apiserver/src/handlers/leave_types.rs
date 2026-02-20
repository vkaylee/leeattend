use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, get, put},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

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
    Path(workspace_id): Path<Uuid>,
    _ctx: MemberContext,
) -> AppResult<Json<Vec<LeaveType>>> {
    let types = sqlx::query_as!(
        LeaveType,
        r#"
        SELECT id, workspace_id, code, name, is_paid, 
               policy_config as "policy_config: sqlx::types::Json<LeaveTypeConfig>", 
               created_at
        FROM leave_types
        WHERE workspace_id = $1
        ORDER BY name ASC
        "#,
        workspace_id
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(types))
}

pub async fn create_leave_type(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
    ctx: MemberContext,
    Json(req): Json<CreateLeaveTypeRequest>,
) -> AppResult<impl IntoResponse> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden(
            "Only admins can create leave types".to_string(),
        ));
    }

    let config = req.policy_config.unwrap_or_default();

    let lt: LeaveType = sqlx::query_as!(
        LeaveType,
        r#"
        INSERT INTO leave_types (workspace_id, code, name, is_paid, policy_config)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, workspace_id, code, name, is_paid, 
                  policy_config as "policy_config: sqlx::types::Json<LeaveTypeConfig>", 
                  created_at
        "#,
        workspace_id,
        req.code,
        req.name,
        req.is_paid,
        sqlx::types::Json(config) as _
    )
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(lt)))
}

pub async fn update_leave_type_config(
    State(state): State<Arc<AppState>>,
    Path((workspace_id, id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<UpdateLeaveTypeConfigRequest>,
) -> AppResult<Json<LeaveType>> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden(
            "Only admins can configure leave types".to_string(),
        ));
    }

    let lt = sqlx::query_as!(
        LeaveType,
        r#"
        UPDATE leave_types
        SET policy_config = $1
        WHERE id = $2 AND workspace_id = $3
        RETURNING id, workspace_id, code, name, is_paid, 
                  policy_config as "policy_config: sqlx::types::Json<LeaveTypeConfig>", 
                  created_at
        "#,
        sqlx::types::Json(req.policy_config) as _,
        id,
        workspace_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Leave type not found".to_string()))?;

    Ok(Json(lt))
}

pub async fn delete_leave_type(
    State(state): State<Arc<AppState>>,
    Path((workspace_id, id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden(
            "Only admins can delete leave types".to_string(),
        ));
    }

    let in_balances = sqlx::query!(
        "SELECT 1 as has_data FROM leave_balances WHERE leave_type_id = $1 LIMIT 1",
        id
    )
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    if in_balances.is_some() {
        return Err(AppError::Conflict(
            "Cannot delete: Leave type is currently used in employee balances".to_string(),
        ));
    }

    let res = sqlx::query!(
        "DELETE FROM leave_types WHERE id = $1 AND workspace_id = $2",
        id,
        workspace_id
    )
    .execute(&state.db)
    .await?;

    if res.rows_affected() > 0 {
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

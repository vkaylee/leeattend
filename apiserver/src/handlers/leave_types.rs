use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, delete, put},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

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
) -> Result<Json<Vec<LeaveType>>, StatusCode> {
    sqlx::query_as!(
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
    .await
    .map(Json)
    .map_err(|e| {
        eprintln!("Error fetching leave types: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

pub async fn create_leave_type(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
    ctx: MemberContext,
    Json(req): Json<CreateLeaveTypeRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can create leave types").into_response();
    }

    let config = req.policy_config.unwrap_or_default();

    let result = sqlx::query_as!(
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
    .await;

    match result {
        Ok(lt) => (StatusCode::CREATED, Json(lt)).into_response(),
        Err(e) => {
            eprintln!("Error creating leave type: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn update_leave_type_config(
    State(state): State<Arc<AppState>>,
    Path((workspace_id, id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<UpdateLeaveTypeConfigRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can configure leave types").into_response();
    }

    let result = sqlx::query_as!(
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
    .await;

    match result {
        Ok(Some(lt)) => (StatusCode::OK, Json(lt)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Leave type not found").into_response(),
        Err(e) => {
            eprintln!("Error updating leave type: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn delete_leave_type(
    State(state): State<Arc<AppState>>,
    Path((workspace_id, id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can delete leave types").into_response();
    }

    let in_balances = sqlx::query!("SELECT 1 as has_data FROM leave_balances WHERE leave_type_id = $1 LIMIT 1", id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

    if in_balances.is_some() {
        return (StatusCode::CONFLICT, "Cannot delete: Leave type is currently used in employee balances").into_response();
    }

    let result = sqlx::query!("DELETE FROM leave_types WHERE id = $1 AND workspace_id = $2", id, workspace_id)
        .execute(&state.db)
        .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => StatusCode::NO_CONTENT.into_response(),
        Ok(_) => (StatusCode::NOT_FOUND, "Leave type not found").into_response(),
        Err(e) => {
            eprintln!("Error deleting leave type: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_leave_types).post(create_leave_type))
        .route("/:id", delete(delete_leave_type))
        .route("/:id/config", put(update_leave_type_config))
}
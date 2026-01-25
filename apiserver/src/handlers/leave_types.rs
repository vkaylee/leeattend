use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, put},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::handlers::common::MemberContext;
use crate::models::{LeaveType, LeaveTypeConfig};
use crate::AppState;

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
        eprintln!("Error fetching leave types: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
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
            eprintln!("Error updating leave type: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_leave_types))
        .route("/:id/config", put(update_leave_type_config))
}

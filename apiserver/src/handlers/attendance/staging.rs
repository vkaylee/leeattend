use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

use crate::handlers::common::MemberContext;
use crate::repositories::staging_log_repository::{self, StagingLog};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct StagingLogsQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, serde::Serialize)]
pub struct StagingLogResponse {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub device_id: Option<String>,
    pub raw_user_id: String,
    pub punch_time: DateTime<Utc>,
    pub raw_data: Option<serde_json::Value>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

impl From<StagingLog> for StagingLogResponse {
    fn from(log: StagingLog) -> Self {
        Self {
            id: log.id,
            workspace_id: log.workspace_id,
            device_id: log.device_id,
            raw_user_id: log.raw_user_id,
            punch_time: log.punch_time,
            raw_data: log.raw_data,
            status: log.status,
            created_at: log.created_at,
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct StagingLogsResponse {
    pub data: Vec<StagingLogResponse>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
}

/// GET /workspaces/:ws_id/attendance/staging-logs
/// List pending staging logs (unknown users)
pub async fn get_staging_logs(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<StagingLogsQuery>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Admin access required").into_response();
    }

    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(50);

    let logs = match staging_log_repository::get_pending_staging_logs(
        &state.db,
        ctx.workspace_id,
        limit,
    ).await {
        Ok(logs) => logs,
        Err(e) => {
            error!("Failed to fetch staging logs: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let total = match staging_log_repository::count_pending_staging_logs(&state.db, ctx.workspace_id).await {
        Ok(count) => count,
        Err(_) => logs.len() as i64,
    };

    let response = StagingLogsResponse {
        data: logs.into_iter().map(StagingLogResponse::from).collect(),
        total,
        page,
        limit,
    };

    Json(response).into_response()
}

#[derive(Debug, Deserialize)]
pub struct ResolveStagingLogRequest {
    pub employee_id: Uuid,
}

/// POST /workspaces/:ws_id/attendance/staging-logs/:id/resolve
/// Manually assign a staging log to an employee
pub async fn resolve_staging_log(
    Path((_ws_id, log_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<ResolveStagingLogRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Admin access required").into_response();
    }

    // 1. Get staging log
    let staged = match sqlx::query_as::<_, StagingLog>(
        "SELECT id, workspace_id, device_id, raw_user_id, punch_time, raw_data, fingerprint, status, created_at FROM attendance_staging_logs WHERE id = $1 AND workspace_id = $2"
    )
    .bind(log_id)
    .bind(ctx.workspace_id)
    .fetch_optional(&state.db)
    .await {
        Ok(Some(log)) => log,
        Ok(None) => return (StatusCode::NOT_FOUND, "Staging log not found").into_response(),
        Err(e) => {
            error!("Failed to fetch staging log: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if staged.status != "pending" {
        return (StatusCode::BAD_REQUEST, "Log already processed").into_response();
    }

    // 2. Create RawAttendanceLog
    use crate::models::{LogSource, RawAttendanceLog};
    use crate::repositories::raw_attendance_log_repository;
    use crate::services::pairing_service::PairingService;

    let raw_log = RawAttendanceLog {
        id: Uuid::new_v4(),
        workspace_id: ctx.workspace_id,
        employee_id: req.employee_id,
        punch_time: staged.punch_time,
        source: LogSource::Device,
        device_id: staged.device_id,
        raw_data: staged.raw_data,
        created_at: Utc::now(),
    };

    // 3. Insert to raw_attendance_logs
    match raw_attendance_log_repository::create_raw_log(&state.db, &raw_log).await {
        Ok(_) => {
            // Mark staging log as processed
            let _ = staging_log_repository::update_staging_status(&state.db, log_id, "processed").await;

            // Trigger pairing
            let _ = PairingService::process_new_raw_log(&state.db, &raw_log).await;

            (StatusCode::OK, "Log resolved successfully").into_response()
        }
        Err(e) => {
            error!("Failed to resolve staging log: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to resolve log").into_response()
        }
    }
}

/// DELETE /workspaces/:ws_id/attendance/staging-logs/:id
/// Dismiss a staging log (mark as dismissed without processing)
pub async fn dismiss_staging_log(
    Path((_ws_id, log_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return StatusCode::FORBIDDEN;
    }

    match staging_log_repository::update_staging_status(&state.db, log_id, "dismissed").await {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(e) => {
            error!("Failed to dismiss staging log: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

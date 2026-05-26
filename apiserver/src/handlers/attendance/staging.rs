use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::repositories::staging_log_repository::StagingLog;
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
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<StagingLogsQuery>,
) -> AppResult<Json<StagingLogsResponse>> {
    ctx.require_permission(Permission::AttendanceStagingManage)?;

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 100);

    let logs = state
        .attendance
        .staging_repo
        .get_pending_staging_logs(ctx.workspace_id, limit)
        .await?;

    let total = state
        .attendance
        .staging_repo
        .count_pending_staging_logs(ctx.workspace_id)
        .await?;

    let response = StagingLogsResponse {
        data: logs.into_iter().map(StagingLogResponse::from).collect(),
        total,
        page,
        limit,
    };

    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct ResolveStagingLogRequest {
    pub employee_id: Uuid,
}

/// POST /workspaces/:ws_id/attendance/staging-logs/:id/resolve
/// Manually assign a staging log to an employee.
/// Uses FOR UPDATE NOWAIT to prevent double-resolve race conditions.
pub async fn resolve_staging_log(
    Path((_, log_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<ResolveStagingLogRequest>,
) -> AppResult<(StatusCode, &'static str)> {
    ctx.require_permission(Permission::AttendanceStagingManage)?;

    let raw_log = state
        .attendance
        .staging_repo
        .resolve_staging_log(log_id, ctx.workspace_id, req.employee_id)
        .await?;

    // 6. Trigger pairing (post-commit side effect, log errors but don't fail the response)
    if let Err(e) = state
        .attendance
        .pairing_service
        .process_new_raw_log(&raw_log)
        .await
    {
        tracing::error!(log_id = %log_id, error = %e, "Failed to trigger pairing after staging resolve");
    }

    Ok((StatusCode::OK, "Log resolved successfully"))
}

/// DELETE /workspaces/:ws_id/attendance/staging-logs/:id
/// Dismiss a staging log (mark as dismissed without processing)
pub async fn dismiss_staging_log(
    Path((_, log_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::AttendanceStagingManage)?;

    let updated = state
        .attendance
        .staging_repo
        .update_staging_status(log_id, ctx.workspace_id, "dismissed")
        .await?;

    if !updated {
        return Err(AppError::NotFound("Staging log not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

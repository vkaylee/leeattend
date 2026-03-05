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
) -> AppResult<Json<StagingLogsResponse>> {
    ctx.require_permission(Permission::AttendanceStagingManage)?;

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 100);

    let logs = staging_log_repository::get_pending_staging_logs(state.db.api(), ctx.workspace_id, limit)
        .await?;

    let total =
        staging_log_repository::count_pending_staging_logs(state.db.api(), ctx.workspace_id).await?;

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
    Path((_ws_id, log_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<ResolveStagingLogRequest>,
) -> AppResult<(StatusCode, &'static str)> {
    ctx.require_permission(Permission::AttendanceStagingManage)?;

    use crate::models::{LogSource, RawAttendanceLog};
    use crate::repositories::raw_attendance_log_repository;
    use crate::services::pairing_service::PairingService;

    // Use transaction to ensure atomicity of read-check-insert-update
    let mut tx = state.db.api().begin().await?;

    // 1. Lock and fetch staging log (FOR UPDATE NOWAIT prevents concurrent resolve)
    let staged = sqlx::query_as::<_, StagingLog>(
        "SELECT id, workspace_id, device_id, raw_user_id, punch_time, raw_data, fingerprint, status, created_at \
         FROM attendance_staging_logs WHERE id = $1 AND workspace_id = $2 FOR UPDATE NOWAIT"
    )
    .bind(log_id)
    .bind(ctx.workspace_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        // NOWAIT returns a specific error if row is already locked
        if e.to_string().contains("could not obtain lock") {
            AppError::Conflict("Log is being processed by another request".to_string())
        } else {
            AppError::from(e)
        }
    })?
    .ok_or_else(|| AppError::NotFound("Staging log not found".to_string()))?;

    if staged.status != "pending" {
        return Err(AppError::Validation("Log already processed".to_string()));
    }

    // 2. Create RawAttendanceLog
    let punch = staged
        .raw_data
        .as_ref()
        .and_then(|d| d["punch"].as_i64())
        .map(|v| v as i32)
        .unwrap_or(-1);
    let status = staged
        .raw_data
        .as_ref()
        .and_then(|d| d["status"].as_i64())
        .map(|v| v as i32)
        .unwrap_or(-1);

    let raw_log = RawAttendanceLog {
        id: Uuid::new_v4(),
        workspace_id: ctx.workspace_id,
        employee_id: req.employee_id,
        punch_time: staged.punch_time,
        source: LogSource::Device,
        device_id: staged.device_id,
        raw_data: staged.raw_data,
        punch,
        status,
        created_at: Utc::now(),
    };

    // 3. Insert raw log within transaction
    raw_attendance_log_repository::create_raw_log(state.db.api(), &raw_log).await?;

    // 4. Mark staging log as processed within transaction
    staging_log_repository::update_staging_status_tx(
        &mut tx,
        log_id,
        ctx.workspace_id,
        "processed",
    )
    .await?;

    // 5. Commit transaction
    tx.commit().await?;

    // 6. Trigger pairing (post-commit side effect, log errors but don't fail the response)
    if let Err(e) = PairingService::process_new_raw_log(state.db.api(), &raw_log).await {
        tracing::error!(log_id = %log_id, error = %e, "Failed to trigger pairing after staging resolve");
    }

    Ok((StatusCode::OK, "Log resolved successfully"))
}

/// DELETE /workspaces/:ws_id/attendance/staging-logs/:id
/// Dismiss a staging log (mark as dismissed without processing)
pub async fn dismiss_staging_log(
    Path((_ws_id, log_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::AttendanceStagingManage)?;

    let updated = staging_log_repository::update_staging_status(
        state.db.api(),
        log_id,
        ctx.workspace_id,
        "dismissed",
    )
    .await?;

    if !updated {
        return Err(AppError::NotFound("Staging log not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

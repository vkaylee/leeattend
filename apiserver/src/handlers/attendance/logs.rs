use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::repositories::workspace_repository::WorkspaceRepository;
use crate::AppState;

#[derive(Deserialize)]
pub struct AttendanceLogQuery {
    pub employee_id: Option<Uuid>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

pub async fn get_attendance_logs(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(query): Query<AttendanceLogQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;

    let logs = state.attendance.time_entry_repo.get_paginated_logs(workspace_id,
        query.employee_id,
        query.from,
        query.to,
        limit,
        offset,
    )
    .await?;

    let total = state.attendance.time_entry_repo.count_logs(workspace_id,
        query.employee_id,
        query.from,
        query.to,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "data": logs,
        "total": total,
        "page": page,
        "limit": limit,
    })))
}

pub async fn get_raw_attendance_logs(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(query): Query<AttendanceLogQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;

    let logs = state.attendance.raw_log_repo.get_paginated_raw_logs(workspace_id,
        query.employee_id,
        query.from,
        query.to,
        limit,
        offset,
    )
    .await?;

    let total = state.attendance.raw_log_repo.count_raw_logs(workspace_id,
        query.employee_id,
        query.from,
        query.to,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "data": logs,
        "total": total,
        "page": page,
        "limit": limit,
    })))
}

#[derive(Deserialize)]
pub struct ReprocessQuery {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

pub async fn force_reprocess_attendance(
    State(state): State<Arc<AppState>>,
    Path((_, employee_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Query(query): Query<ReprocessQuery>,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::AttendanceReprocess)?;

    const MAX_REPROCESS_DAYS: i64 = 90;

    let date_range = (query.end_date - query.start_date).num_days();
    if date_range < 0 {
        return Err(AppError::Validation(
            "end_date must be after start_date".to_string(),
        ));
    }
    if date_range > MAX_REPROCESS_DAYS {
        return Err(AppError::Validation(format!(
            "Max reprocess range is {MAX_REPROCESS_DAYS} days"
        )));
    }

    let workspace = crate::repositories::workspace_repository::SqlxWorkspaceRepository {
        pool: state.infra.db.api().clone(),
    }
    .get_workspace_by_id(ctx.workspace_id)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound("Workspace not found".to_string()))?;

    let tz: chrono_tz::Tz = workspace
        .timezone
        .parse()
        .map_err(|_| AppError::from("Invalid timezone configuration"))?;

    let mut curr_date = query.start_date;
    while curr_date <= query.end_date {
        state
            .attendance.pairing_service
            .reprocess_daily_attendance(
                ctx.workspace_id,
                employee_id,
                curr_date,
                tz,
            )
            .await
        .map_err(|e| AppError::from(format!("Processing failed for {curr_date}: {e}")))?;

        if let Some(next) = curr_date.succ_opt() {
            curr_date = next;
        } else {
            break;
        }
    }

    Ok(StatusCode::OK)
}

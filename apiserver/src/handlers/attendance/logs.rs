use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Deserialize;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

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
    Path(_ws_id): Path<Uuid>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(query): Query<AttendanceLogQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let limit = query.limit.unwrap_or(20);
    let page = query.page.unwrap_or(1);
    let offset = (page - 1) * limit;

    let logs = crate::repositories::time_entry_repository::get_paginated_logs(
        &state.db,
        workspace_id,
        query.employee_id,
        query.from,
        query.to,
        limit,
        offset,
    )
    .await
    .map_err(|e| {
        error!("GET_LOGS_ERROR: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let total = crate::repositories::time_entry_repository::count_logs(
        &state.db,
        workspace_id,
        query.employee_id,
        query.from,
        query.to,
    )
    .await
    .map_err(|e| {
        error!("COUNT_LOGS_ERROR: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(serde_json::json!({
        "data": logs,
        "total": total,
        "page": page,
        "limit": limit,
    })))
}

pub async fn get_raw_attendance_logs(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(query): Query<AttendanceLogQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let limit = query.limit.unwrap_or(20);
    let page = query.page.unwrap_or(1);
    let offset = (page - 1) * limit;

    let logs = crate::repositories::raw_attendance_log_repository::get_paginated_raw_logs(
        &state.db,
        workspace_id,
        query.employee_id,
        query.from,
        query.to,
        limit,
        offset,
    )
    .await
    .map_err(|e| {
        error!("GET_RAW_LOGS_ERROR: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let total = crate::repositories::raw_attendance_log_repository::count_raw_logs(
        &state.db,
        workspace_id,
        query.employee_id,
        query.from,
        query.to,
    )
    .await
    .map_err(|e| {
        error!("COUNT_RAW_LOGS_ERROR: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

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
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Query(query): Query<ReprocessQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Err(StatusCode::FORBIDDEN);
    }

    let workspace = crate::repositories::workspace_repository::SqlxWorkspaceRepository {
        pool: state.db.clone(),
    }
    .get_workspace_by_id(ctx.workspace_id)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let tz: chrono_tz::Tz = workspace
        .timezone
        .parse()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut curr_date = query.start_date;
    while curr_date <= query.end_date {
        crate::services::pairing_service::PairingService::reprocess_daily_attendance(
            &state.db,
            employee_id,
            curr_date,
            tz,
            &workspace.pairing_algorithm,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if let Some(next) = curr_date.succ_opt() {
            curr_date = next;
        } else {
            break;
        }
    }

    Ok(StatusCode::OK)
}

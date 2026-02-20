use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{PunchInRequest, PunchOutRequest, TimeEntry, UpdateTimeEntryRequest};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

pub async fn punch_in(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
    MemberContext { .. }: MemberContext,
    Json(req): Json<PunchInRequest>,
) -> AppResult<(StatusCode, Json<TimeEntry>)> {
    let entry = crate::services::time_tracking_service::TimeTrackingService::punch_in(
        &state.db,
        workspace_id,
        req,
    )
    .await
    .map_err(|s| AppError::from((s, "Failed to punch in".to_string())))?;

    Ok((StatusCode::CREATED, Json(entry)))
}

pub async fn punch_out(
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<PunchOutRequest>,
) -> AppResult<Json<TimeEntry>> {
    let entry = crate::services::time_tracking_service::TimeTrackingService::punch_out(
        &state.db,
        workspace_id,
        employee_id,
        req,
    )
    .await
    .map_err(|s| AppError::from((s, "Failed to punch out".to_string())))?;

    Ok(Json(entry))
}

pub async fn get_time_history(
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(query): Query<HistoryQuery>,
) -> AppResult<Json<Vec<TimeEntry>>> {
    let entries =
        crate::services::time_tracking_service::TimeTrackingService::get_employee_history(
            &state.db,
            workspace_id,
            employee_id,
            query.start,
            query.end,
        )
        .await
        .map_err(|s| AppError::from((s, "Failed to get time history".to_string())))?;

    Ok(Json(entries))
}

pub async fn update_time_entry(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<UpdateTimeEntryRequest>,
) -> AppResult<Json<TimeEntry>> {
    let entry = crate::services::time_tracking_service::TimeTrackingService::update_time_entry(
        &state.db,
        workspace_id,
        id,
        req,
    )
    .await
    .map_err(|s| AppError::from((s, "Failed to update time entry".to_string())))?;

    Ok(Json(entry))
}

pub async fn delete_time_entry(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<StatusCode> {
    crate::services::time_tracking_service::TimeTrackingService::delete_time_entry(
        &state.db,
        workspace_id,
        id,
    )
    .await
    .map_err(|s| AppError::from((s, "Failed to delete time entry".to_string())))?;

    Ok(StatusCode::NO_CONTENT)
}

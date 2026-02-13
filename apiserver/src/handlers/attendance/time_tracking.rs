use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::handlers::common::MemberContext;
use crate::models::{PunchInRequest, PunchOutRequest, UpdateTimeEntryRequest};
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
) -> Result<impl IntoResponse, StatusCode> {
    match crate::services::time_tracking_service::TimeTrackingService::punch_in(
        &state.db,
        workspace_id,
        req,
    )
    .await
    {
        Ok(entry) => Ok((StatusCode::CREATED, Json(entry))),
        Err(status) => Err(status),
    }
}

pub async fn punch_out(
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<PunchOutRequest>,
) -> impl IntoResponse {
    match crate::services::time_tracking_service::TimeTrackingService::punch_out(
        &state.db,
        workspace_id,
        employee_id,
        req,
    )
    .await
    {
        Ok(entry) => (StatusCode::OK, Json(entry)).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn get_time_history(
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(query): Query<HistoryQuery>,
) -> impl IntoResponse {
    match crate::services::time_tracking_service::TimeTrackingService::get_employee_history(
        &state.db,
        workspace_id,
        employee_id,
        query.start,
        query.end,
    )
    .await
    {
        Ok(entries) => Json(entries).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn update_time_entry(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<UpdateTimeEntryRequest>,
) -> impl IntoResponse {
    match crate::services::time_tracking_service::TimeTrackingService::update_time_entry(
        &state.db,
        workspace_id,
        id,
        req,
    )
    .await
    {
        Ok(entry) => Json(entry).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn delete_time_entry(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> impl IntoResponse {
    match crate::services::time_tracking_service::TimeTrackingService::delete_time_entry(
        &state.db,
        workspace_id,
        id,
    )
    .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(status) => status.into_response(),
    }
}

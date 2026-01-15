use axum::{
    extract::{State, Path, Query},
    response::{IntoResponse, Json},
    http::StatusCode,
};
use std::sync::Arc;
use uuid::Uuid;
use chrono::NaiveDate;
use std::collections::HashMap;
use tracing::error;

use crate::AppState;
use crate::handlers::common::MemberContext;
use crate::models::{
    CreateShiftRequest, UpdateShiftRequest, AssignShiftRequest, CreateRosterRequest,
    BulkRosterRequest, ShiftRoster, EmployeeSchedule, AssignmentHistory
};

// --- Shift Handlers ---

pub async fn get_shifts(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> impl IntoResponse {
    match crate::services::shift_service::ShiftService::get_workspace_shifts(
        &state.db,
        workspace_id,
    )
    .await
    {
        Ok(shifts) => Json(shifts).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn create_shift(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    ctx: MemberContext,
    Json(req): Json<CreateShiftRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Err(StatusCode::FORBIDDEN);
    }
    
    let workspace_id = ctx.workspace_id;
    match crate::services::shift_service::ShiftService::create_shift(
        &state.db,
        workspace_id,
        req.name,
        req.start_time,
        req.end_time,
        req.break_duration_minutes,
        req.break_start,
        req.break_end,
        req.is_break_auto_deduct,
        req.break_is_overtime,
        req.early_punch_in_grace_period_minutes,
        req.allow_early_ot,
    )
    .await
    {
        Ok(shift) => Ok((StatusCode::CREATED, Json(shift))),
        Err(e) => Err(e),
    }
}

pub async fn update_shift(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, shift_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<UpdateShiftRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Err(StatusCode::FORBIDDEN);
    }
    match crate::services::shift_service::ShiftService::update_shift(
        &state.db,
        shift_id,
        req,
    )
    .await
    {
        Ok(shift) => Ok(Json(shift)),
        Err(status) => Err(status),
    }
}


pub async fn assign_shift(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    MemberContext { .. }: MemberContext,
    Json(req): Json<AssignShiftRequest>,
) -> impl IntoResponse {
    match crate::services::shift_service::ShiftService::assign_shift(
        &state.db,
        req.employee_id,
        req.shift_id,
        req.start_date,
        req.end_date,
    )
    .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(status) => status.into_response(),
    }
}

// --- Roster Handlers ---

pub async fn get_workspace_schedules(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<EmployeeSchedule>>, StatusCode> {
    let start_date = params.get("start_date")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let end_date = params.get("end_date")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let schedules = crate::repositories::roster_repository::get_workspace_schedules(
        &state.db,
        workspace_id,
        start_date,
        end_date
    )
    .await
    .map_err(|e| {
        error!("Failed to get workspace schedules: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(schedules))
}

pub async fn get_assignment_history(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
) -> Result<Json<Vec<AssignmentHistory>>, StatusCode> {
    let history = crate::repositories::roster_repository::get_assignment_history(
        &state.db,
        workspace_id,
        50 // Limit to last 50 entries
    )
    .await
    .map_err(|e| {
        error!("Failed to get assignment history: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(history))
}

pub async fn get_employee_rosters(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<ShiftRoster>>, StatusCode> {
    let start_date = params
        .get("start_date")
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let end_date = params
        .get("end_date")
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .unwrap_or_else(|| start_date + chrono::Duration::days(30));

    let rosters = crate::repositories::roster_repository::get_employee_rosters(
        &state.db,
        employee_id,
        start_date,
        end_date,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rosters))
}

pub async fn upsert_roster(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<CreateRosterRequest>,
) -> Result<Json<ShiftRoster>, StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Err(StatusCode::FORBIDDEN);
    }
    let roster = ShiftRoster {
        id: Uuid::new_v4(),
        employee_id,
        shift_id: req.shift_id,
        date: req.date,
        status: "confirmed".to_string(),
        notes: req.notes,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let saved = crate::repositories::roster_repository::upsert_roster(&state.db, &roster)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(saved))
}

pub async fn bulk_upsert_roster(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<BulkRosterRequest>,
) -> Result<StatusCode, StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Err(StatusCode::FORBIDDEN);
    }
    crate::repositories::roster_repository::bulk_upsert_roster(&state.db, employee_id, req.items)
        .await
        .map_err(|e| {
            error!("Failed to bulk upsert roster: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::OK)
}

pub async fn delete_roster(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, employee_id, date)): Path<(Uuid, Uuid, NaiveDate)>,
    ctx: MemberContext,
) -> Result<StatusCode, StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Err(StatusCode::FORBIDDEN);
    }
    crate::repositories::roster_repository::delete_roster(&state.db, employee_id, date)
        .await
        .map_err(|e| {
            error!("Failed to delete roster: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::NO_CONTENT)
}

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::NaiveDate;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{
    AssignShiftRequest, AssignmentHistory, BulkRosterRequest, CreateRosterRequest,
    CreateShiftRequest, EmployeeSchedule, Shift, ShiftRoster, UpdateShiftRequest,
};
use crate::AppState;

// --- Shift Handlers ---

pub async fn get_shifts(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<Shift>>> {
    let shifts =
        state.shift_service.get_workspace_shifts(workspace_id)
            .await
            .map_err(|s| AppError::from((s, "Failed to get workspace shifts".to_string())))?;

    Ok(Json(shifts))
}

pub async fn create_shift(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    ctx: MemberContext,
    Json(req): Json<CreateShiftRequest>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::ShiftsCreate)?;

    let workspace_id = ctx.workspace_id;
    let shift = state.shift_service.create_shift(
        workspace_id,
        req.name,
        req.report_symbol,
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
    .map_err(|s| AppError::from((s, "Failed to create shift".to_string())))?;

    if let Some(cache) = &state.roster_cache {
        cache.invalidate(workspace_id).await;
    }

    Ok((StatusCode::CREATED, Json(shift)))
}

pub async fn update_shift(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, shift_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<UpdateShiftRequest>,
) -> AppResult<Json<Shift>> {
    ctx.require_permission(Permission::ShiftsUpdate)?;
    let shift =
        state.shift_service.update_shift(ctx.workspace_id, shift_id, req)
            .await
            .map_err(|s| AppError::from((s, "Failed to update shift".to_string())))?;

    if let Some(cache) = &state.roster_cache {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok(Json(shift))
}

pub async fn assign_shift(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    ctx: MemberContext,
    Json(req): Json<AssignShiftRequest>,
) -> AppResult<StatusCode> {
    state.shift_service.assign_shift(
        req.employee_id,
        req.shift_id,
        req.start_date,
        req.end_date,
    )
    .await
    .map_err(|s| AppError::from((s, "Failed to assign shift".to_string())))?;

    if let Some(cache) = &state.roster_cache {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok(StatusCode::OK)
}

// --- Roster Handlers ---

pub async fn get_workspace_schedules(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Json<Vec<EmployeeSchedule>>> {
    let start_date = params
        .get("start_date")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| AppError::Validation("Missing or invalid start_date".to_string()))?;

    let end_date = params
        .get("end_date")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| AppError::Validation("Missing or invalid end_date".to_string()))?;

    let schedules = state.roster_repo.get_workspace_schedules(
        state.roster_cache.as_deref(),
        workspace_id,
        start_date,
        end_date,
        None,
    )
    .await?;

    Ok(Json(schedules))
}

pub async fn get_assignment_history(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<Uuid>,
) -> AppResult<Json<Vec<AssignmentHistory>>> {
    let history = state.roster_repo.get_assignment_history(
        workspace_id,
        50, // Limit to last 50 entries
    )
    .await?;

    Ok(Json(history))
}

pub async fn get_employee_rosters(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> AppResult<Json<Vec<ShiftRoster>>> {
    let start_date = params
        .get("start_date")
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let end_date = params
        .get("end_date")
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .unwrap_or_else(|| start_date + chrono::Duration::days(30));

    let rosters = state.roster_repo.get_employee_rosters(
        employee_id,
        start_date,
        end_date,
    )
    .await?;

    Ok(Json(rosters))
}

pub async fn upsert_roster(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<CreateRosterRequest>,
) -> AppResult<Json<ShiftRoster>> {
    ctx.require_permission(Permission::RostersManage)?;
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

    let saved = state.roster_repo.upsert_roster(&roster).await?;

    if let Some(cache) = &state.roster_cache {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok(Json(saved))
}

pub async fn bulk_upsert_roster(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<BulkRosterRequest>,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::RostersManage)?;
    state.roster_repo.bulk_upsert_roster( employee_id, req.items)
        .await?;

    if let Some(cache) = &state.roster_cache {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok(StatusCode::OK)
}

pub async fn delete_roster(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, employee_id, date)): Path<(Uuid, Uuid, NaiveDate)>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::RostersManage)?;
    state.roster_repo.delete_roster( employee_id, date).await?;

    if let Some(cache) = &state.roster_cache {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok(StatusCode::NO_CONTENT)
}

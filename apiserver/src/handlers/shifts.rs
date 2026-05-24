#![allow(unused_variables)]
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

/// Reprocess attendance for a single (employee, date) after a roster change.
///
/// Fetches the workspace timezone from DB and delegates to `PairingService`.
/// Errors are logged but not propagated — the roster mutation itself already
/// succeeded, so we don't want to roll it back.
async fn reprocess_after_roster_change(
    state: &AppState,
    workspace_id: Uuid,
    employee_id: Uuid,
    date: NaiveDate,
) {
    let tz_str = match state
        .core
        .workspace_repo
        .get_workspace_by_id(workspace_id)
        .await
    {
        Ok(Some(ws)) => ws.timezone,
        _ => "UTC".to_string(),
    };
    let tz = crate::utils::tz::parse_workspace_tz(&tz_str);

    if let Err(e) = state
        .attendance
        .pairing_service
        .reprocess_daily_attendance(workspace_id, employee_id, date, tz)
        .await
    {
        tracing::error!(
            employee_id = %employee_id,
            date = %date,
            "Failed to reprocess attendance after roster change: {e}"
        );
    }
}

// --- Shift Handlers ---

pub async fn get_shifts(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<Shift>>> {
    let workspace_id = ctx.workspace_id;

    let shifts = state
        .shifts
        .service
        .get_workspace_shifts(workspace_id)
        .await?;

    Ok(Json(shifts))
}

pub async fn create_shift(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateShiftRequest>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::ShiftsCreate)?;

    let workspace_id = ctx.workspace_id;
    let shift = state
        .shifts
        .service
        .create_shift(workspace_id, &req)
        .await?;

    if let Some(cache) = &state.infra.roster_cache {
        cache.invalidate(workspace_id).await;
    }

    Ok((StatusCode::CREATED, Json(shift)))
}

pub async fn update_shift(
    State(state): State<Arc<AppState>>,
    Path((_, shift_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<UpdateShiftRequest>,
) -> AppResult<Json<Shift>> {
    ctx.require_permission(Permission::ShiftsUpdate)?;
    let shift = state
        .shifts
        .service
        .update_shift(ctx.workspace_id, shift_id, req)
        .await?;

    if let Some(cache) = &state.infra.roster_cache {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok(Json(shift))
}

pub async fn assign_shift(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<AssignShiftRequest>,
) -> AppResult<StatusCode> {
    state
        .shifts
        .service
        .assign_shift(
            ctx.workspace_id,
            req.employee_id,
            req.shift_id,
            req.start_date,
            req.end_date,
        )
        .await?;

    if let Some(cache) = &state.infra.roster_cache {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok(StatusCode::OK)
}

// --- Roster Handlers ---

pub async fn get_workspace_schedules(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Json<Vec<EmployeeSchedule>>> {
    let workspace_id = ctx.workspace_id;

    let start_date = params
        .get("start_date")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| AppError::Validation("Missing or invalid start_date".to_string()))?;

    let end_date = params
        .get("end_date")
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| AppError::Validation("Missing or invalid end_date".to_string()))?;

    let schedules = state
        .shifts
        .roster_repo
        .get_workspace_schedules(
            state.infra.roster_cache.as_deref(),
            &ctx.auth_token,
            start_date,
            end_date,
            None,
        )
        .await?;

    Ok(Json(schedules))
}

pub async fn get_assignment_history(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<AssignmentHistory>>> {
    let workspace_id = ctx.workspace_id;

    let history = state
        .shifts
        .roster_repo
        .get_assignment_history(
            &ctx.auth_token,
            50, // Limit to last 50 entries
        )
        .await?;

    Ok(Json(history))
}

pub async fn get_employee_rosters(
    State(state): State<Arc<AppState>>,
    Path((_, employee_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
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

    let rosters = state
        .shifts
        .roster_repo
        .get_employee_rosters(&ctx.auth_token, employee_id, start_date, end_date)
        .await?;

    Ok(Json(rosters))
}

pub async fn upsert_roster(
    State(state): State<Arc<AppState>>,
    Path((_, employee_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<CreateRosterRequest>,
) -> AppResult<Json<ShiftRoster>> {
    ctx.require_permission(Permission::RostersManage)?;
    let roster = ShiftRoster {
        id: crate::utils::id::new_id(),
        employee_id,
        shift_id: req.shift_id,
        date: req.date,
        status: "confirmed".to_string(),
        notes: req.notes,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let saved = state
        .shifts
        .roster_repo
        .upsert_roster(&ctx.auth_token, &roster)
        .await?;

    if let Some(cache) = &state.infra.roster_cache {
        cache.invalidate(ctx.workspace_id).await;
    }

    reprocess_after_roster_change(&state, ctx.workspace_id, employee_id, req.date).await;

    Ok(Json(saved))
}

pub async fn bulk_upsert_roster(
    State(state): State<Arc<AppState>>,
    Path((_, employee_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<BulkRosterRequest>,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::RostersManage)?;
    let dates: Vec<NaiveDate> = req.items.iter().map(|i| i.date).collect();
    state
        .shifts
        .roster_repo
        .bulk_upsert_roster(&ctx.auth_token, employee_id, req.items)
        .await?;

    if let Some(cache) = &state.infra.roster_cache {
        cache.invalidate(ctx.workspace_id).await;
    }

    for date in dates {
        reprocess_after_roster_change(&state, ctx.workspace_id, employee_id, date).await;
    }

    Ok(StatusCode::OK)
}

pub async fn delete_roster(
    State(state): State<Arc<AppState>>,
    Path((_, employee_id, date)): Path<(Uuid, Uuid, NaiveDate)>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::RostersManage)?;
    state
        .shifts
        .roster_repo
        .delete_roster(&ctx.auth_token, employee_id, date)
        .await?;

    if let Some(cache) = &state.infra.roster_cache {
        cache.invalidate(ctx.workspace_id).await;
    }

    reprocess_after_roster_change(&state, ctx.workspace_id, employee_id, date).await;

    Ok(StatusCode::NO_CONTENT)
}

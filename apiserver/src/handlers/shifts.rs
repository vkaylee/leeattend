use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::NaiveDate;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{
    AssignShiftRequest, AssignmentHistory, BulkRosterRequest, CreateRosterRequest,
    CreateShiftRequest, EmployeeSchedule, Shift, ShiftRoster, UpdateShiftRequest,
};
use crate::utils::i18n::I18nKey;
use crate::AppState;

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct WorkspaceSchedulesQuery {
    pub start_date: String,
    pub end_date: String,
}

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct EmployeeRostersQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

/// Reprocess attendance for a single (employee, date) after a roster change.
async fn reprocess_after_roster_change(
    state: &AppState,
    workspace_id: Uuid,
    employment_id: Uuid,
    date: NaiveDate,
) {
    let tz_str = match state
        .core()
        .workspace_repo()
        .get_workspace_by_id(workspace_id)
        .await
    {
        Ok(Some(ws)) => ws.timezone().to_string(),
        _ => "UTC".to_string(),
    };
    let tz = crate::utils::tz::parse_workspace_tz(&tz_str);

    if let Err(e) = state
        .attendance()
        .pairing_service()
        .reprocess_daily_attendance(workspace_id, employment_id, date, tz)
        .await
    {
        tracing::error!(
            employment_id = %employment_id,
            date = %date,
            "Failed to reprocess attendance after roster change: {e}"
        );
    }
}

// --- Shift Handlers ---

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct GetShiftsQuery {
    pub location_id: Option<Uuid>,
    pub employer_id: Option<Uuid>,
}

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/shifts",
    responses(
        (status = 200, description = "List of shifts", body = [Shift])
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        GetShiftsQuery
    )
)]
pub async fn get_shifts(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetShiftsQuery>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<Shift>>> {
    let workspace_id = ctx.workspace_id;

    let shifts = state
        .shifts()
        .service()
        .get_workspace_shifts(workspace_id, query.location_id, query.employer_id)
        .await?;

    Ok(Json(shifts))
}

#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/shifts",
    request_body = CreateShiftRequest,
    responses(
        (status = 201, description = "Shift created", body = Shift)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn create_shift(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateShiftRequest>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::ShiftsCreate)?;

    let workspace_id = ctx.workspace_id;
    let shift = state
        .shifts()
        .service()
        .create_shift(workspace_id, &req)
        .await?;

    if let Some(cache) = state.infra().roster_cache() {
        cache.invalidate(workspace_id).await;
    }

    Ok((StatusCode::CREATED, Json(shift)))
}

#[utoipa::path(
    put,
    path = "/workspaces/{workspace_id}/shifts/{shift_id}",
    request_body = UpdateShiftRequest,
    responses(
        (status = 200, description = "Shift updated", body = Shift)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("shift_id" = Uuid, Path, description = "Shift ID")
    )
)]
pub async fn update_shift(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<UpdateShiftRequest>,
) -> AppResult<Json<Shift>> {
    let (_workspace_id, shift_id) = params;
    ctx.require_permission(Permission::ShiftsUpdate)?;
    let shift = state
        .shifts()
        .service()
        .update_shift(ctx.workspace_id, shift_id, req)
        .await?;

    if let Some(cache) = state.infra().roster_cache() {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok(Json(shift))
}

#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/shifts/assign",
    request_body = AssignShiftRequest,
    responses(
        (status = 200, description = "Shift assigned successfully")
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn assign_shift(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<AssignShiftRequest>,
) -> AppResult<StatusCode> {
    state
        .shifts()
        .service()
        .assign_shift(
            ctx.workspace_id,
            req.employee_id,
            req.shift_id,
            req.start_date,
            req.end_date,
        )
        .await?;

    if let Some(cache) = state.infra().roster_cache() {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok(StatusCode::OK)
}

// --- Roster Handlers ---

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/schedules",
    responses(
        (status = 200, description = "Workspace schedules", body = [EmployeeSchedule])
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        WorkspaceSchedulesQuery
    )
)]
pub async fn get_workspace_schedules(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<WorkspaceSchedulesQuery>,
) -> AppResult<Json<Vec<EmployeeSchedule>>> {
    let _workspace_id = ctx.workspace_id;

    let start_date = NaiveDate::parse_from_str(&query.start_date, "%Y-%m-%d")
        .map_err(|_err| AppError::validation(I18nKey::InvalidStartDateValue))?;

    let end_date = NaiveDate::parse_from_str(&query.end_date, "%Y-%m-%d")
        .map_err(|_err| AppError::validation(I18nKey::InvalidEndDateValue))?;

    let schedules = state
        .shifts()
        .roster_repo()
        .get_workspace_schedules(
            state.infra().roster_cache().map(|v| &**v),
            &ctx.auth_token,
            start_date,
            end_date,
            None,
        )
        .await?;

    Ok(Json(schedules))
}

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/assignment-history",
    responses(
        (status = 200, description = "Assignment history", body = [AssignmentHistory])
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn get_assignment_history(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<AssignmentHistory>>> {
    let _workspace_id = ctx.workspace_id;

    let history = state
        .shifts()
        .roster_repo()
        .get_assignment_history(
            &ctx.auth_token,
            50, // Limit to last 50 entries
        )
        .await?;

    Ok(Json(history))
}

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/employees/{id}/rosters",
    responses(
        (status = 200, description = "Employee rosters", body = [ShiftRoster])
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employee ID"),
        EmployeeRostersQuery
    )
)]
pub async fn get_employee_rosters(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Query(query): Query<EmployeeRostersQuery>,
) -> AppResult<Json<Vec<ShiftRoster>>> {
    let (_workspace_id, employee_id) = params;
    let start_date = query
        .start_date
        .as_deref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let end_date = query
        .end_date
        .as_deref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .unwrap_or_else(|| start_date + chrono::Duration::days(30));

    let emp = state
        .employees()
        .repo()
        .get_employee_by_id(employee_id, &ctx.auth_token)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::EmployeeNotFound))?;
    let active_employment_id = emp
        .active_employment_id()
        .ok_or_else(|| AppError::validation(I18nKey::NoActiveEmployment))?;

    let rosters = state
        .shifts()
        .roster_repo()
        .get_employment_rosters(&ctx.auth_token, active_employment_id, start_date, end_date)
        .await?;

    Ok(Json(rosters))
}

#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/employees/{id}/rosters",
    request_body = CreateRosterRequest,
    responses(
        (status = 200, description = "Roster created/updated", body = ShiftRoster)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employee ID")
    )
)]
pub async fn upsert_roster(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<CreateRosterRequest>,
) -> AppResult<Json<ShiftRoster>> {
    let (_workspace_id, employee_id) = params;
    ctx.require_permission(Permission::RostersManage)?;

    let emp = state
        .employees()
        .repo()
        .get_employee_by_id(employee_id, &ctx.auth_token)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::EmployeeNotFound))?;
    let active_employment_id = emp
        .active_employment_id()
        .ok_or_else(|| AppError::validation(I18nKey::NoActiveEmployment))?;

    let roster = ShiftRoster {
        id: crate::utils::id::new_id(),
        employment_id: active_employment_id,
        shift_id: req.shift_id,
        date: req.date,
        status: "confirmed".to_string(),
        notes: req.notes,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let saved = state
        .shifts()
        .roster_repo()
        .upsert_roster(&ctx.auth_token, &roster)
        .await?;

    if let Some(cache) = state.infra().roster_cache() {
        cache.invalidate(ctx.workspace_id).await;
    }

    reprocess_after_roster_change(&state, ctx.workspace_id, active_employment_id, req.date).await;

    Ok(Json(saved))
}

#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/employees/{id}/rosters/bulk",
    request_body = BulkRosterRequest,
    responses(
        (status = 200, description = "Bulk rosters created/updated")
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employee ID")
    )
)]
pub async fn bulk_upsert_roster(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<BulkRosterRequest>,
) -> AppResult<StatusCode> {
    let (_workspace_id, employee_id) = params;
    ctx.require_permission(Permission::RostersManage)?;

    let emp = state
        .employees()
        .repo()
        .get_employee_by_id(employee_id, &ctx.auth_token)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::EmployeeNotFound))?;
    let active_employment_id = emp
        .active_employment_id()
        .ok_or_else(|| AppError::validation(I18nKey::NoActiveEmployment))?;

    let dates: Vec<NaiveDate> = req.items.iter().map(|i| i.date).collect();
    state
        .shifts()
        .roster_repo()
        .bulk_upsert_roster(&ctx.auth_token, active_employment_id, req.items)
        .await?;

    if let Some(cache) = state.infra().roster_cache() {
        cache.invalidate(ctx.workspace_id).await;
    }

    for date in dates {
        reprocess_after_roster_change(&state, ctx.workspace_id, active_employment_id, date).await;
    }

    Ok(StatusCode::OK)
}

#[utoipa::path(
    delete,
    path = "/workspaces/{workspace_id}/employees/{id}/rosters/{date}",
    responses(
        (status = 204, description = "Roster deleted")
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employee ID"),
        ("date" = NaiveDate, Path, description = "Roster Date")
    )
)]
pub async fn delete_roster(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid, NaiveDate)>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    let (_workspace_id, employee_id, date) = params;
    ctx.require_permission(Permission::RostersManage)?;

    let emp = state
        .employees()
        .repo()
        .get_employee_by_id(employee_id, &ctx.auth_token)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::EmployeeNotFound))?;
    let active_employment_id = emp
        .active_employment_id()
        .ok_or_else(|| AppError::validation(I18nKey::NoActiveEmployment))?;

    state
        .shifts()
        .roster_repo()
        .delete_roster(&ctx.auth_token, active_employment_id, date)
        .await?;

    if let Some(cache) = state.infra().roster_cache() {
        cache.invalidate(ctx.workspace_id).await;
    }

    reprocess_after_roster_change(&state, ctx.workspace_id, active_employment_id, date).await;

    Ok(StatusCode::NO_CONTENT)
}

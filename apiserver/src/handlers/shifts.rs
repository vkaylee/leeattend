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
    CreateShiftRequest, EmployeeSchedule, PaginatedResponse, PaginationParams, Shift, ShiftRoster,
    UpdateShiftRequest,
};
use crate::utils::i18n::I18nKey;
use crate::AppState;

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct WorkspaceSchedulesQuery {
    pub start_date: String,
    pub end_date: String,
    pub employment_ids: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct EmployeeRostersQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct PaginatedEmployeeSchedules {
    pub data: Vec<EmployeeSchedule>,
    pub total: usize,
    pub page: i64,
    pub limit: i64,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct PaginatedAssignmentHistory {
    pub data: Vec<AssignmentHistory>,
    pub total: usize,
    pub page: i64,
    pub limit: i64,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct PaginatedShiftRosters {
    pub data: Vec<ShiftRoster>,
    pub total: usize,
    pub page: i64,
    pub limit: i64,
}

/// Upper bound on how far back a shift reassignment triggers recalculation.
const MAX_ASSIGN_REPROCESS_DAYS: i64 = 90;

/// Upper bound on how many roster items a single bulk-upsert request may carry.
/// The DB write is a single multi-row upsert and attendance reprocessing is now
/// enqueued (not run inline), so this no longer bounds request cost — it only
/// rejects pathological payloads. A full year is 366 days, generous headroom for
/// any single employee's selection.
const MAX_ROSTER_BULK_ITEMS: usize = 366;

/// Clamp a shift-assignment window to the range that actually needs attendance
/// recalculation: nothing in the future (no punches yet) and no further back than
/// [`MAX_ASSIGN_REPROCESS_DAYS`]. `end` is `None` for open-ended assignments.
///
/// Returns `None` when the window is entirely in the future — the common case for
/// HR pre-assigning next month's shifts — so no job is enqueued.
fn compute_assign_reprocess_range(
    start: NaiveDate,
    end: Option<NaiveDate>,
    today: NaiveDate,
) -> Option<(NaiveDate, NaiveDate)> {
    let recalc_end = end.unwrap_or(today).min(today);
    let recalc_start = start.max(today - chrono::Duration::days(MAX_ASSIGN_REPROCESS_DAYS));
    // Empty window: either entirely in the future (start beyond end) or entirely
    // older than the reprocess floor (floor pushed start past end).
    if recalc_start > recalc_end {
        return None;
    }
    Some((recalc_start, recalc_end))
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

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct PaginatedShifts {
    pub data: Vec<Shift>,
    pub total: usize,
    pub page: i64,
    pub limit: i64,
}

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct GetShiftsQuery {
    pub location_id: Option<Uuid>,
    pub employer_id: Option<Uuid>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/shifts",
    responses(
        (status = 200, description = "List of shifts", body = PaginatedShifts)
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
) -> AppResult<Json<crate::models::PaginatedResponse<Shift>>> {
    let workspace_id = ctx.workspace_id;
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 3000);

    let (mut shifts, total) = state
        .shifts()
        .service()
        .get_workspace_shifts(
            workspace_id,
            query.location_id,
            query.employer_id,
            Some(page),
            Some(limit),
        )
        .await?;

    // Attach intervals to each shift so the frontend can display time/break info
    if let Some(cache) = state.infra().roster_cache() {
        if let Ok(roster_data) = cache
            .get_roster_data(workspace_id, state.infra().diesel())
            .await
        {
            for shift in &mut shifts {
                if let Some(intervals) = roster_data.shift_intervals.get(&shift.id()) {
                    let entities: Vec<crate::models::diesel::roster::ShiftDefaultIntervalEntity> =
                        intervals
                            .iter()
                            .map(
                                |iv| crate::models::diesel::roster::ShiftDefaultIntervalEntity {
                                    id: iv.interval_id,
                                    version_id: Uuid::nil(), // resolved from cache, version_id not stored in cache
                                    sort_order: iv.sort_order,
                                    interval_name: iv.interval_name.clone(),
                                    start_time: iv.start_time,
                                    end_time: iv.end_time,
                                    break_duration_minutes: iv.break_duration_minutes,
                                    break_start: iv.break_start,
                                    break_end: iv.break_end,
                                    is_break_auto_deduct: iv.is_break_auto_deduct,
                                    break_is_overtime: iv.break_is_overtime,
                                    early_punch_in_grace_period_minutes: iv
                                        .early_punch_in_grace_period_minutes,
                                    allow_early_ot: iv.allow_early_ot,
                                    max_early_ot_minutes: iv.max_early_ot_minutes,
                                    ot_aggregation_mode: iv.ot_aggregation_mode.clone(),
                                    is_night_shift: iv.is_night_shift,
                                    created_at: None,
                                    updated_at: None,
                                },
                            )
                            .collect();
                    shift.intervals = Some(entities);
                }
            }
        }
    }

    Ok(Json(crate::models::PaginatedResponse {
        data: shifts,
        total: usize::try_from(total).unwrap_or(0),
        page,
        limit,
    }))
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
            req.employment_id,
            req.shift_id,
            req.start_date,
            req.end_date,
        )
        .await?;

    if let Some(cache) = state.infra().roster_cache() {
        cache.invalidate(ctx.workspace_id).await;
    }

    // Recalculate attendance for days already worked under the reassigned shift.
    // Best-effort: the assignment is already persisted, so a queue failure must
    // not fail the request — it is logged and can be re-driven via bulk reprocess.
    let today = chrono::Utc::now()
        .with_timezone(&workspace_tz(&state, ctx.workspace_id).await)
        .date_naive();
    if let Some((recalc_start, recalc_end)) =
        compute_assign_reprocess_range(req.start_date, req.end_date, today)
    {
        match resolve_and_enqueue_reprocess(
            &state,
            &ctx,
            req.employment_id,
            recalc_start,
            recalc_end,
        )
        .await
        {
            Ok(job_id) => tracing::info!(
                job_id = %job_id,
                employment_id = %req.employment_id,
                start = %recalc_start,
                end = %recalc_end,
                "Enqueued attendance reprocess after shift assignment"
            ),
            Err(e) => tracing::error!(
                employment_id = %req.employment_id,
                "Failed to enqueue reprocess after shift assignment: {e}"
            ),
        }
    }

    Ok(StatusCode::OK)
}

/// Resolve the timezone for a workspace, defaulting to UTC on any lookup failure.
async fn workspace_tz(state: &AppState, workspace_id: Uuid) -> chrono_tz::Tz {
    let tz_str = match state
        .core()
        .workspace_repo()
        .get_workspace_by_id(workspace_id)
        .await
    {
        Ok(Some(ws)) => ws.timezone().to_string(),
        _ => "UTC".to_string(),
    };
    crate::utils::tz::parse_workspace_tz(&tz_str)
}

/// Resolve the owning employee for an employment and enqueue a reprocess job
/// pinned to that exact employment.
async fn resolve_and_enqueue_reprocess(
    state: &AppState,
    ctx: &MemberContext,
    employment_id: Uuid,
    start: NaiveDate,
    end: NaiveDate,
) -> Result<Uuid, String> {
    let employment = state
        .employees()
        .employment_repo()
        .get_employment_by_id(&ctx.auth_token, employment_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Employment not found: {employment_id}"))?;

    let queue = state.infra().job_queue().map_err(|e| e.to_string())?;

    crate::handlers::attendance::logs::enqueue_reprocess_jobs(
        &queue,
        ctx.workspace_id,
        ctx.user_id,
        &[(employment.employee_id(), Some(employment_id))],
        crate::queue::DateRange { start, end },
    )
    .await
}

// --- Roster Handlers ---

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/schedules",
    responses(
        (status = 200, description = "Workspace schedules", body = PaginatedEmployeeSchedules)
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
) -> AppResult<Json<PaginatedResponse<EmployeeSchedule>>> {
    let _workspace_id = ctx.workspace_id;

    let start_date = NaiveDate::parse_from_str(&query.start_date, "%Y-%m-%d")
        .map_err(|_err| AppError::validation(I18nKey::InvalidStartDateValue))?;

    let end_date = NaiveDate::parse_from_str(&query.end_date, "%Y-%m-%d")
        .map_err(|_err| AppError::validation(I18nKey::InvalidEndDateValue))?;

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 3000);

    let parsed_employment_ids = if let Some(ids_str) = &query.employment_ids {
        if ids_str.trim().is_empty() {
            None
        } else {
            let ids: Result<Vec<Uuid>, _> = ids_str
                .split(',')
                .map(|s| Uuid::parse_str(s.trim()))
                .collect();
            Some(ids.map_err(|_err| AppError::validation(I18nKey::Validation))?)
        }
    } else {
        None
    };

    let (schedules, total) = state
        .shifts()
        .roster_repo()
        .get_workspace_schedules(
            state.infra().roster_cache().map(|v| &**v),
            &ctx.auth_token,
            start_date,
            end_date,
            parsed_employment_ids.as_ref(),
            Some(page),
            Some(limit),
        )
        .await?;

    Ok(Json(PaginatedResponse {
        data: schedules,
        total: usize::try_from(total).unwrap_or(0),
        page,
        limit,
    }))
}

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/assignment-history",
    responses(
        (status = 200, description = "Assignment history", body = PaginatedAssignmentHistory)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        PaginationParams
    )
)]
pub async fn get_assignment_history(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<AssignmentHistory>>> {
    let _workspace_id = ctx.workspace_id;
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 3000);

    let (history, total) = state
        .shifts()
        .roster_repo()
        .get_assignment_history(&ctx.auth_token, Some(page), Some(limit))
        .await?;

    Ok(Json(PaginatedResponse {
        data: history,
        total: usize::try_from(total).unwrap_or(0),
        page,
        limit,
    }))
}

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/employees/{id}/rosters",
    responses(
        (status = 200, description = "Employee rosters", body = PaginatedShiftRosters)
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
) -> AppResult<Json<PaginatedResponse<ShiftRoster>>> {
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

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 3000);

    let (rosters, total) = state
        .shifts()
        .roster_repo()
        .get_employment_rosters(
            &ctx.auth_token,
            active_employment_id,
            start_date,
            end_date,
            Some(page),
            Some(limit),
        )
        .await?;

    Ok(Json(PaginatedResponse {
        data: rosters,
        total: usize::try_from(total).unwrap_or(0),
        page,
        limit,
    }))
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
        (status = 202, description = "Bulk rosters saved; attendance reprocess enqueued")
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

    if req.items.len() > MAX_ROSTER_BULK_ITEMS {
        return Err(AppError::validation(I18nKey::RosterBatchTooLarge));
    }

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

    // The roster write above is durable; attendance recalculation is offloaded to
    // the worker so the request returns immediately. The range is inclusive of every
    // day between the earliest and latest selected date; reprocessing untouched days
    // in between is idempotent. Best-effort: an enqueue failure must not fail the
    // request — the roster is already persisted and can be re-driven via bulk reprocess.
    if let (Some(&start_date), Some(&end_date)) = (dates.iter().min(), dates.iter().max()) {
        match resolve_and_enqueue_reprocess(
            &state,
            &ctx,
            active_employment_id,
            start_date,
            end_date,
        )
        .await
        {
            Ok(job_id) => tracing::info!(
                job_id = %job_id,
                employment_id = %active_employment_id,
                start = %start_date,
                end = %end_date,
                "Enqueued attendance reprocess after bulk roster upsert"
            ),
            Err(e) => tracing::error!(
                employment_id = %active_employment_id,
                "Failed to enqueue reprocess after bulk roster upsert: {e}"
            ),
        }
    }

    Ok(StatusCode::ACCEPTED)
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

#[cfg(test)]
mod tests {
    use super::compute_assign_reprocess_range;
    use chrono::NaiveDate;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn all_future_assignment_is_skipped() {
        let today = d(2026, 7, 17);
        // HR assigns next month's shifts.
        assert_eq!(
            compute_assign_reprocess_range(d(2026, 8, 1), Some(d(2026, 8, 31)), today),
            None
        );
    }

    #[test]
    fn open_ended_future_assignment_is_skipped() {
        let today = d(2026, 7, 17);
        assert_eq!(
            compute_assign_reprocess_range(d(2026, 7, 18), None, today),
            None
        );
    }

    #[test]
    fn bridging_assignment_clamps_end_to_today() {
        let today = d(2026, 7, 17);
        // Spans past into future; only the worked part [10, today] is recalculated.
        assert_eq!(
            compute_assign_reprocess_range(d(2026, 7, 10), Some(d(2026, 8, 10)), today),
            Some((d(2026, 7, 10), today))
        );
    }

    #[test]
    fn open_ended_from_past_clamps_end_to_today() {
        let today = d(2026, 7, 17);
        assert_eq!(
            compute_assign_reprocess_range(d(2026, 7, 1), None, today),
            Some((d(2026, 7, 1), today))
        );
    }

    #[test]
    fn fully_past_assignment_is_reprocessed_in_full() {
        let today = d(2026, 7, 17);
        assert_eq!(
            compute_assign_reprocess_range(d(2026, 6, 1), Some(d(2026, 6, 30)), today),
            Some((d(2026, 6, 1), d(2026, 6, 30)))
        );
    }

    #[test]
    fn far_past_start_is_clamped_to_90_days() {
        let today = d(2026, 7, 17);
        // Start one year ago -> clamped to today - 90d.
        let (start, end) =
            compute_assign_reprocess_range(d(2025, 7, 17), Some(today), today).unwrap();
        assert_eq!(start, today - chrono::Duration::days(90));
        assert_eq!(end, today);
    }

    #[test]
    fn fully_past_window_older_than_floor_is_skipped() {
        let today = d(2026, 7, 17);
        // Whole window sits before today - 90d (2026-04-18): nothing recent to
        // recalculate, so no job — must not produce an inverted range.
        assert_eq!(
            compute_assign_reprocess_range(d(2026, 3, 1), Some(d(2026, 3, 31)), today),
            None
        );
    }

    #[test]
    fn single_day_today_is_included() {
        let today = d(2026, 7, 17);
        assert_eq!(
            compute_assign_reprocess_range(today, Some(today), today),
            Some((today, today))
        );
    }
}

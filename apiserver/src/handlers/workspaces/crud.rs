use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::handlers::system_auth::SystemContext;
use crate::models::enums::SystemRoleType;
use crate::models::{
    AttendanceTrendPoint, CreateWorkspaceRequest, DepartmentAttendancePoint, LatenessByDay,
    LeaveTypeBreakdown, OvertimeTrendPoint, PeakTimeSlot, UnexcusedAbsencesPoint,
    UpdateWorkspaceRequest, Workspace, WorkspaceStats,
};

use crate::{AppState, UserId};

#[utoipa::path(
    get,
    path = "/dashboard/attendance-trend",
    responses(
        (status = 200, description = "Get attendance trend analysis data", body = Vec<AttendanceTrendPoint>)
    )
)]
pub async fn get_attendance_trend(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<AttendanceTrendPoint>>> {
    let _workspace_id = ctx.workspace_id;

    let result = state
        .reports
        .stats_repo
        .fetch_attendance_trend(&ctx.auth_token)
        .await?;
    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct DeptAttendanceQuery {
    pub range: Option<String>,
}

#[utoipa::path(
    get,
    path = "/dashboard/department-attendance",
    responses(
        (status = 200, description = "Get department-scoped attendance analytics", body = Vec<DepartmentAttendancePoint>)
    ),
    params(
        ("range" = Option<String>, Query, description = "Analytics date range (e.g. today, week, month)")
    )
)]
pub async fn get_department_attendance(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(params): Query<DeptAttendanceQuery>,
) -> AppResult<Json<Vec<DepartmentAttendancePoint>>> {
    let _workspace_id = ctx.workspace_id;

    let range = params.range.unwrap_or_else(|| "today".to_string());
    let result = state
        .reports
        .stats_repo
        .fetch_department_attendance(&ctx.auth_token, &range)
        .await?;
    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct UnexcusedAbsencesQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub group_by: Option<String>,
}

#[allow(clippy::too_many_lines)]
#[utoipa::path(
    get,
    path = "/dashboard/unexcused-absences",
    responses(
        (status = 200, description = "Get unexcused absences list/stats", body = Vec<UnexcusedAbsencesPoint>)
    ),
    params(
        ("start_date" = Option<String>, Query, description = "Start Date (YYYY-MM-DD)"),
        ("end_date" = Option<String>, Query, description = "End Date (YYYY-MM-DD)"),
        ("group_by" = Option<String>, Query, description = "Group results by employee or department")
    )
)]
pub async fn get_unexcused_absences(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(params): Query<UnexcusedAbsencesQuery>,
) -> AppResult<Json<Vec<UnexcusedAbsencesPoint>>> {
    let workspace_id = ctx.workspace_id;

    use crate::services::attendance_utils::{
        self, determine_daily_status, AttendanceStatus, DailyStatusInput,
    };
    use chrono_tz::Tz;
    use std::collections::HashMap;

    let today = Utc::now().date_naive();

    let end_date = params
        .end_date
        .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
        .unwrap_or(today)
        .min(today); // Don't look into the future

    let start_date = params
        .start_date
        .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| end_date - chrono::Duration::days(29));

    let group_by = params.group_by.unwrap_or_else(|| "department".to_string());

    // 1. Get workspace timezone
    let workspace = state
        .core
        .workspace_repo
        .get_workspace_by_id(workspace_id)
        .await
        .map_err(|e| AppError::Internal(format!("Error fetching workspace: {e}")))?
        .ok_or_else(|| AppError::NotFound("Workspace not found".to_string()))?;

    let tz: Tz = crate::utils::tz::parse_workspace_tz(&workspace.timezone);

    // 2. Get schedules for ALL active employees (reuses 6-priority chain logic)
    let schedules = state
        .shifts
        .roster_repo
        .get_workspace_schedules(
            state.infra.roster_cache.as_deref(),
            &ctx.auth_token,
            start_date,
            end_date,
            None, // all employees
        )
        .await
        .map_err(|e| AppError::Internal(format!("Error fetching schedules: {e}")))?;

    // 3. Batch-fetch time entries (using shared date_range_to_utc)
    let emp_ids: Vec<Uuid> = schedules.iter().map(|s| s.employee_id).collect();
    let (start_dt, end_dt) = attendance_utils::date_range_to_utc(tz, start_date, end_date)
        .map_err(|e| AppError::Validation(format!("Invalid date range: {e}")))?;

    let entries_map = state
        .attendance
        .time_entry_repo
        .get_workspace_time_entries_batch(&ctx.auth_token, &emp_ids, start_dt, end_dt)
        .await
        .map_err(|e| AppError::Internal(format!("Error fetching time entries: {e}")))?;

    // 4. Holiday info is already embedded in EffectiveSchedule.is_holiday from get_workspace_schedules.

    // 5. Batch-fetch approved leave requests (using shared build_leave_date_set)
    let mut conn = state
        .infra
        .diesel
        .api
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let leave_date_set = attendance_utils::build_leave_date_set(
        &mut conn,
        workspace_id,
        &emp_ids,
        start_date,
        end_date,
    )
    .await?;

    // 6. Batch-fetch department info for each employee
    let dept_map = state
        .employees
        .repo
        .get_employee_departments_at(&ctx.auth_token, end_date)
        .await?;

    // 7. Determine status per day using attendance_utils::determine_daily_status
    struct EmployeeStats {
        employee_name: String,
        department_id: Option<Uuid>,
        department_name: String,
        unexcused_count: i64,
        total_absent_count: i64,
        consecutive_count: i64,
    }

    let mut emp_stats: HashMap<Uuid, EmployeeStats> = HashMap::new();
    let unassigned_id = Uuid::nil();

    for emp_sched in &schedules {
        let entries = entries_map
            .get(&emp_sched.employee_id)
            .cloned()
            .unwrap_or_default();

        // Pre-group entries by date using shared utility
        let entries_by_date = attendance_utils::group_entries_by_date(&entries, tz);

        let (dept_id, dept_name) = dept_map
            .get(&emp_sched.employee_id)
            .cloned()
            .unwrap_or_else(|| (None, "Unassigned".to_string()));

        let mut unexcused = 0i64;
        let mut total_absent = 0i64;
        let mut consecutive = 0i64;
        let mut last_present_date: Option<NaiveDate> = None;

        // Find last present date for consecutive calculation
        for entry in &entries {
            let d = entry.punch_in_time.with_timezone(&tz).date_naive();
            if last_present_date.is_none_or(|lpd| d > lpd) {
                last_present_date = Some(d);
            }
        }

        for daily in &emp_sched.schedules {
            let date = daily.date;
            if date > today {
                continue;
            }

            let is_on_leave = leave_date_set.contains(&(emp_sched.employee_id, date));

            let status = determine_daily_status(&DailyStatusInput {
                has_punch_in: entries_by_date.contains_key(&date),
                punch_in_time: None, // Not needed for unexcused calculation
                scheduled_start: daily.start_time,
                is_holiday: daily.is_holiday,
                is_holiday_paid: false, // Not needed for unexcused calculation
                has_shift: daily.shift_id.is_some(),
                is_on_leave,
            });

            if status == AttendanceStatus::Absent {
                total_absent += 1;

                if !is_on_leave {
                    unexcused += 1;

                    // Consecutive: days after last punch-in without leave
                    let is_after_last_punch = last_present_date.is_none_or(|lpd| date > lpd);
                    if is_after_last_punch {
                        consecutive += 1;
                    }
                }
            }
        }

        emp_stats.insert(
            emp_sched.employee_id,
            EmployeeStats {
                employee_name: emp_sched.full_name.clone(),
                department_id: dept_id,
                department_name: dept_name,
                unexcused_count: unexcused,
                total_absent_count: total_absent,
                consecutive_count: consecutive,
            },
        );
    }

    // 8. Build result based on group_by
    if group_by == "employee" {
        let mut result: Vec<UnexcusedAbsencesPoint> = emp_stats
            .into_iter()
            .map(|(emp_id, stats)| UnexcusedAbsencesPoint {
                department_id: stats.department_id.unwrap_or(unassigned_id),
                department_name: stats.department_name,
                employee_id: Some(emp_id),
                employee_name: Some(stats.employee_name),
                unexcused_count: stats.unexcused_count,
                consecutive_count: stats.consecutive_count,
                total_absent_count: stats.total_absent_count,
                total_leave_count: 0,
            })
            .collect();
        result.sort_by(|a, b| b.unexcused_count.cmp(&a.unexcused_count));
        Ok(Json(result))
    } else {
        // Aggregate by department
        let mut dept_agg: HashMap<(Uuid, String), (i64, i64, i64)> = HashMap::new();
        for stats in emp_stats.values() {
            let key = (
                stats.department_id.unwrap_or(unassigned_id),
                stats.department_name.clone(),
            );
            let entry = dept_agg.entry(key).or_insert((0, 0, 0));
            entry.0 += stats.unexcused_count;
            entry.1 += stats.total_absent_count;
            entry.2 += stats.consecutive_count;
        }

        let mut result: Vec<UnexcusedAbsencesPoint> = dept_agg
            .into_iter()
            .map(
                |((dept_id, dept_name), (unexcused, total_absent, consecutive))| {
                    UnexcusedAbsencesPoint {
                        department_id: dept_id,
                        department_name: dept_name,
                        employee_id: None,
                        employee_name: None,
                        unexcused_count: unexcused,
                        consecutive_count: consecutive,
                        total_absent_count: total_absent,
                        total_leave_count: 0,
                    }
                },
            )
            .collect();
        result.sort_by(|a, b| b.unexcused_count.cmp(&a.unexcused_count));
        Ok(Json(result))
    }
}

#[utoipa::path(
    get,
    path = "/dashboard/overtime-trend",
    responses(
        (status = 200, description = "Get overtime trend analysis data", body = Vec<OvertimeTrendPoint>)
    )
)]
pub async fn get_overtime_trend(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<OvertimeTrendPoint>>> {
    let _workspace_id = ctx.workspace_id;

    let result = state
        .reports
        .stats_repo
        .fetch_overtime_trend(&ctx.auth_token)
        .await?;
    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/dashboard/workspace-stats",
    responses(
        (status = 200, description = "Get workspace key metrics and status summary", body = WorkspaceStats)
    )
)]
pub async fn get_workspace_stats(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<WorkspaceStats>> {
    let _workspace_id = ctx.workspace_id;

    let result = state
        .reports
        .stats_repo
        .fetch_workspace_stats(&ctx.auth_token)
        .await?;
    Ok(Json(result))
}

// ── Peak Check-in/Check-out Times ──────────────────────────────────

#[utoipa::path(
    get,
    path = "/dashboard/peak-times",
    responses(
        (status = 200, description = "Get peak check-in and check-out times slots", body = Vec<PeakTimeSlot>)
    )
)]
pub async fn get_peak_times(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<PeakTimeSlot>>> {
    let _workspace_id = ctx.workspace_id;

    let result = state
        .reports
        .stats_repo
        .fetch_peak_times(&ctx.auth_token)
        .await?;
    Ok(Json(result))
}

// ── Leave Types Breakdown ──────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/dashboard/leave-types",
    responses(
        (status = 200, description = "Get leave requests breakdown by leave type", body = Vec<LeaveTypeBreakdown>)
    )
)]
pub async fn get_leave_types_breakdown(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<LeaveTypeBreakdown>>> {
    let _workspace_id = ctx.workspace_id;

    let result = state
        .reports
        .stats_repo
        .fetch_leave_types_breakdown(&ctx.auth_token)
        .await?;
    Ok(Json(result))
}

// ── Lateness by Day of Week ────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/dashboard/lateness-by-day",
    responses(
        (status = 200, description = "Get lateness breakdown by day of week", body = Vec<LatenessByDay>)
    )
)]
pub async fn get_lateness_by_day(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<LatenessByDay>>> {
    let _workspace_id = ctx.workspace_id;

    let result = state
        .reports
        .stats_repo
        .fetch_lateness_by_day(&ctx.auth_token)
        .await?;
    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/workspaces",
    responses(
        (status = 200, description = "List all workspaces the user is member of", body = Vec<Workspace>)
    )
)]
pub async fn get_workspaces(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
) -> AppResult<Json<Vec<Workspace>>> {
    let workspaces = state
        .core
        .workspace_repo
        .get_user_workspaces(user_id)
        .await?;

    Ok(Json(workspaces))
}

#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}",
    responses(
        (status = 200, description = "Get workspace details by ID", body = Workspace)
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn get_workspace_by_id(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    UserId(_user_id): UserId,
) -> AppResult<Json<Workspace>> {
    let workspace = state
        .core
        .workspace_repo
        .get_workspace_by_id(ws_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Workspace not found".to_string()))?;

    Ok(Json(workspace))
}

/// POST /workspaces
/// Creates a new workspace.
/// Requires: SystemRole::PlatformAdmin — prevents arbitrary users from creating workspaces.
#[utoipa::path(
    post,
    path = "/workspaces",
    request_body = CreateWorkspaceRequest,
    responses(
        (status = 201, description = "Workspace created successfully", body = Workspace)
    )
)]
pub async fn create_workspace(
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    Json(req): Json<CreateWorkspaceRequest>,
) -> AppResult<Json<Workspace>> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    let ws_id = crate::utils::id::new_id();

    // Encrypt database_url as custom_db_url_encrypted for BYOD support across restarts
    let encrypted_url = if let Some(url) = &req.database_url {
        if let Some(crypto) = &state.infra.crypto_service {
            Some(crypto.encrypt(url).map_err(|e| {
                tracing::error!("Failed to encrypt DB URL: {}", e);
                AppError::Internal("Encryption failed".into())
            })?)
        } else {
            tracing::warn!("No crypto service available, storing URL in plaintext!");
            Some(url.clone())
        }
    } else {
        None
    };

    let ws = state
        .core
        .workspace_service
        .create_workspace(
            ws_id,
            &req.name,
            ctx.user_id,
            req.database_url.clone(),
            encrypted_url,
        )
        .await?;

    Ok(Json(ws))
}

#[utoipa::path(
    patch,
    path = "/workspaces/{ws_id}",
    request_body = UpdateWorkspaceRequest,
    responses(
        (status = 200, description = "Workspace updated successfully", body = Workspace)
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn update_workspace(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateWorkspaceRequest>,
) -> AppResult<Json<Workspace>> {
    ctx.require_permission(Permission::WorkspaceSettingsWrite)?;
    state
        .core
        .workspace_service
        .update_workspace(crate::services::workspace_service::UpdateWorkspaceArgs {
            ws_id: ctx.auth_token.workspace_id(),
            name: req.name,
            date_format: req.date_format,
            time_format: req.time_format,
            timezone: req.timezone,
            theme_primary_color: req.theme_primary_color,
            default_language: req.default_language,
            logo_url: req.logo_url,
            tagline: req.tagline,
            holiday_overnight_strategy: req.holiday_overnight_strategy,
            default_holiday_ot_multiplier: req.default_holiday_ot_multiplier,
        })
        .await
        .map(Json)
}

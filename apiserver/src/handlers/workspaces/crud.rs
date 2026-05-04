use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use chrono::{Datelike, NaiveDate, Utc};
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

pub async fn get_attendance_trend(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<AttendanceTrendPoint>>> {
    let result = state
        .reports
        .stats_repo
        .fetch_attendance_trend(workspace_id)
        .await?;
    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct DeptAttendanceQuery {
    pub range: Option<String>,
}

pub async fn get_department_attendance(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(params): Query<DeptAttendanceQuery>,
) -> AppResult<Json<Vec<DepartmentAttendancePoint>>> {
    let range = params.range.unwrap_or_else(|| "today".to_string());
    let result = state
        .reports
        .stats_repo
        .fetch_department_attendance(workspace_id, &range)
        .await?;
    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct UnexcusedAbsencesQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub group_by: Option<String>,
}

pub async fn get_unexcused_absences(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(params): Query<UnexcusedAbsencesQuery>,
) -> AppResult<Json<Vec<UnexcusedAbsencesPoint>>> {
    use crate::repositories::workspace_repository::WorkspaceRepository;
    use crate::repositories::SqlxWorkspaceRepository;
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
    let workspace_repo = SqlxWorkspaceRepository {
        pool: state.infra.db.reader().clone(),
    };
    let workspace = workspace_repo
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
            workspace_id,
            start_date,
            end_date,
            None, // all employees
        )
        .await
        .map_err(|e| AppError::Internal(format!("Error fetching schedules: {e}")))?;

    // 3. Batch-fetch time entries (using shared date_range_to_utc)
    let emp_ids: Vec<Uuid> = schedules.iter().map(|s| s.employee_id).collect();
    let (start_dt, end_dt) = attendance_utils::date_range_to_utc(tz, start_date, end_date);

    let entries_map = state
        .attendance
        .time_entry_repo
        .get_workspace_time_entries_batch(workspace_id, &emp_ids, start_dt, end_dt)
        .await
        .map_err(|e| AppError::Internal(format!("Error fetching time entries: {e}")))?;

    // 4. Holiday info is already embedded in EffectiveSchedule.is_holiday from get_workspace_schedules.

    // 5. Batch-fetch approved leave requests (using shared build_leave_date_set)
    let leave_date_set = attendance_utils::build_leave_date_set(
        state.infra.db.reader(),
        workspace_id,
        &emp_ids,
        start_date,
        end_date,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Error fetching leave requests: {e}")))?;

    // 6. Batch-fetch department info for each employee
    let dept_rows = sqlx::query!(
        r#"SELECT e.id AS employee_id,
                  edh.department_id AS "department_id?",
                  COALESCE(d.name, 'Unassigned') AS "department_name!"
           FROM employees e
           LEFT JOIN LATERAL (
               SELECT edh2.department_id
               FROM employee_department_history edh2
               WHERE edh2.employee_id = e.id AND edh2.effective_date <= $2
               ORDER BY edh2.effective_date DESC
               LIMIT 1
           ) edh ON TRUE
           LEFT JOIN departments d ON d.id = edh.department_id
           WHERE e.is_active = TRUE AND e.workspace_id = $1"#,
        workspace_id,
        end_date
    )
    .fetch_all(state.infra.db.reader())
    .await
    .map_err(|e| AppError::Internal(format!("Error fetching departments: {e}")))?;

    let dept_map: HashMap<Uuid, (Option<Uuid>, String)> = dept_rows
        .into_iter()
        .map(|r| (r.employee_id, (r.department_id, r.department_name)))
        .collect();

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
            .unwrap_or((None, "Unassigned".to_string()));

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

pub async fn get_overtime_trend(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<OvertimeTrendPoint>>> {
    let result = state
        .reports
        .stats_repo
        .fetch_overtime_trend(workspace_id)
        .await?;
    Ok(Json(result))
}

pub async fn get_workspace_stats(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<WorkspaceStats>> {
    let result = state
        .reports
        .stats_repo
        .fetch_workspace_stats(workspace_id)
        .await?;
    Ok(Json(result))
}

// ── Peak Check-in/Check-out Times ──────────────────────────────────

pub async fn get_peak_times(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<PeakTimeSlot>>> {
    let result = state
        .reports
        .stats_repo
        .fetch_peak_times(workspace_id)
        .await?;
    Ok(Json(result))
}

// ── Leave Types Breakdown ──────────────────────────────────────────

pub async fn get_leave_types_breakdown(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<LeaveTypeBreakdown>>> {
    let result = state
        .reports
        .stats_repo
        .fetch_leave_types_breakdown(workspace_id)
        .await?;
    Ok(Json(result))
}

// ── Lateness by Day of Week ────────────────────────────────────────

pub async fn get_lateness_by_day(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<LatenessByDay>>> {
    let result = state
        .reports
        .stats_repo
        .fetch_lateness_by_day(workspace_id)
        .await?;
    Ok(Json(result))
}

pub async fn get_workspaces(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
) -> AppResult<Json<Vec<Workspace>>> {
    let workspaces = sqlx::query_as::<_, Workspace>(
        "SELECT w.* FROM workspaces w
         JOIN workspace_members wm ON w.id = wm.workspace_id
         WHERE wm.user_id = $1",
    )
    .bind(user_id)
    .fetch_all(state.infra.db.api())
    .await?;

    Ok(Json(workspaces))
}

pub async fn get_workspace_by_id(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    UserId(_user_id): UserId,
) -> AppResult<Json<Workspace>> {
    let workspace = state
        .core
        .workspace_repo
        .get_workspace_by_id(ws_id)
        .await
        .map_err(AppError::Internal)?
        .ok_or_else(|| AppError::NotFound("Workspace not found".to_string()))?;

    Ok(Json(workspace))
}

/// POST /workspaces
/// Creates a new workspace.
/// Requires: SystemRole::PlatformAdmin — prevents arbitrary users from creating workspaces.
pub async fn create_workspace(
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    Json(req): Json<CreateWorkspaceRequest>,
) -> AppResult<Json<Workspace>> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    state
        .core
        .workspace_service
        .create_workspace(&req.name, ctx.user_id)
        .await
        .map(Json)
        .map_err(AppError::from)
}

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
            ws_id: ctx.workspace_id,
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
        .map_err(AppError::from)
}

use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use chrono::NaiveDate;
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
use crate::utils::i18n::I18nKey;

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
        .reports()
        .stats_repo()
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
        .reports()
        .stats_repo()
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

    // Resolve workspace tz for correct local "today"
    let workspace_tz = state
        .core()
        .workspace_repo()
        .get_workspace_by_id(workspace_id)
        .await?
        .map(|ws| crate::utils::tz::parse_workspace_tz(ws.timezone()))
        .unwrap_or(chrono_tz::UTC);
    let today = crate::utils::tz::utc_now_date_in_tz(&workspace_tz);

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

    let result = state
        .reports()
        .service()
        .get_unexcused_absences_stats(
            &ctx.auth_token,
            workspace_id,
            start_date,
            end_date,
            &group_by,
        )
        .await?;

    Ok(Json(result))
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
        .reports()
        .stats_repo()
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
        .reports()
        .stats_repo()
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
        .reports()
        .stats_repo()
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
        .reports()
        .stats_repo()
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
        .reports()
        .stats_repo()
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
        .core()
        .workspace_repo()
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
        .core()
        .workspace_repo()
        .get_workspace_by_id(ws_id)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::WorkspaceNotFound))?;

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
        if let Some(crypto) = state.infra().crypto_service() {
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
        .core()
        .workspace_service()
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
    let updated = state
        .core()
        .workspace_service()
        .update_workspace(
            &ctx.auth_token,
            crate::services::workspace_service::UpdateWorkspaceArgs {
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
            },
        )
        .await?;

    // Drop the stale workspace config (timezone) from cache so the Excel-export
    // worker and the report paths never compute with different timezones for the
    // cache TTL after a settings change.
    state
        .infra()
        .invalidate_workspace_cache(ctx.auth_token.workspace_id())
        .await;

    Ok(Json(updated))
}

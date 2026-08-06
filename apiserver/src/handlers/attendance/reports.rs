use axum::{
    extract::{Query, State},
    response::Json,
};
use chrono::NaiveDate;

use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::{
    validate_report_date_range, MemberContext, SafePagination, DEFAULT_REPORT_PAGE_LIMIT,
};
use crate::models::AttendanceReportSort;
use crate::services::report_service::AttendanceReportArgs;
use crate::utils::i18n::I18nKey;
use crate::AppState;

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct AttendanceReportQuery {
    pub start_date: String,
    pub end_date: String,
    pub r#type: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub sort: Option<String>,
    pub employee_ids: Option<String>,
    pub department_id: Option<uuid::Uuid>,
    pub employee_status: Option<String>,
    pub employer_id: Option<uuid::Uuid>,
    pub employment_id: Option<uuid::Uuid>,
    /// Comma-separated shift names to filter no-show rows by shift type.
    #[serde(default)]
    pub shift_type: Option<String>,
    /// Opt-in query parameter for segment-centric daily reports.
    /// When `true`, the handler uses
    /// [`ReportService::get_segment_attendance_report`] instead of the
    /// legacy path.
    #[serde(default)]
    pub segment_centric: Option<bool>,
}

/// Build an empty paginated response as JSON, keyed by report type.
fn empty_report_json(
    report_type: &str,
    page: Option<i64>,
    limit: Option<i64>,
) -> AppResult<Json<serde_json::Value>> {
    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(20);
    let json = if report_type == "daily" {
        serde_json::to_value(crate::models::PaginatedResponse::<
            crate::models::DailyAttendanceReport,
        > {
            data: Vec::new(),
            total: 0,
            page,
            limit,
            truncated: None,
        })
    } else {
        serde_json::to_value(
            crate::models::PaginatedResponse::<crate::models::AttendanceSummary> {
                data: Vec::new(),
                total: 0,
                page,
                limit,
                truncated: None,
            },
        )
    }
    .map_err(|e| AppError::from(e.to_string()))?;
    Ok(Json(json))
}

/// Build `AttendanceReportArgs` from the query parameters.
///
/// `pagination` carries the already-clamped page/limit. Raw `params.page` /
/// `params.limit` MUST NOT be used here — they flow straight into the SQL
/// `LIMIT` clause, so an unclamped value lets one request exhaust process memory.
fn build_report_args(
    workspace_id: Uuid,
    params: &AttendanceReportQuery,
    start_date: NaiveDate,
    end_date: NaiveDate,
    employee_ids: Option<Vec<Uuid>>,
    sort: Option<AttendanceReportSort>,
    pagination: SafePagination,
) -> AttendanceReportArgs {
    AttendanceReportArgs {
        workspace_id,
        start_date,
        end_date,
        page: Some(pagination.page()),
        limit: Some(pagination.limit()),
        employee_ids,
        department_id: params.department_id,
        sort,
        employee_status: params.employee_status.clone(),
        employer_id: params.employer_id,
        employment_id: params.employment_id,
        shift_type: params.shift_type.clone(),
    }
}

/// Resolve employee IDs visible to a non-global caller via behalf assignments.
/// Returns `None` when the caller has no assignments (caller should early-return empty).
async fn resolve_behalf_employee_ids(
    state: &Arc<AppState>,
    ctx: &MemberContext,
    requested_ids: Option<Vec<Uuid>>,
) -> AppResult<Option<Vec<Uuid>>> {
    let caller_assignments = state
        .attendance()
        .behalf_assignment_repo()
        .list_assignments_for_assignee(ctx.workspace_id, ctx.user_id, "overtime_claim")
        .await?;

    if caller_assignments.is_empty() {
        return Ok(None);
    }

    let mut allowed_employee_ids = std::collections::HashSet::new();
    let mut assigned_dept_ids = Vec::new();

    for a in &caller_assignments {
        if let Some(emp_id) = a.target_employee_id() {
            allowed_employee_ids.insert(emp_id);
        }
        if let Some(dept_id) = a.target_department_id() {
            assigned_dept_ids.push(dept_id);
        }
    }

    if !assigned_dept_ids.is_empty() {
        let active_employees = state
            .employees()
            .repo()
            .get_active_employees(&ctx.auth_token)
            .await?;

        let workspace_tz = state
            .core()
            .workspace_repo()
            .get_workspace_by_id(ctx.workspace_id)
            .await?
            .map(|ws| crate::utils::tz::parse_workspace_tz(ws.timezone()))
            .unwrap_or(chrono_tz::UTC);

        let dept_map = state
            .employees()
            .repo()
            .get_employee_departments_at(
                &ctx.auth_token,
                crate::utils::tz::utc_now_date_in_tz(&workspace_tz),
            )
            .await?;

        for emp in &active_employees {
            if let Some((Some(dept_id), _)) = dept_map.get(&emp.id()) {
                if assigned_dept_ids.contains(dept_id) {
                    allowed_employee_ids.insert(emp.id());
                }
            }
        }
    }

    // Intersect with requested IDs if any, otherwise return all allowed
    let final_ids = if let Some(req_ids) = requested_ids {
        let intersected: Vec<Uuid> = req_ids
            .into_iter()
            .filter(|id| allowed_employee_ids.contains(id))
            .collect();
        if intersected.is_empty() {
            return Ok(None); // No overlap → empty result
        }
        intersected
    } else {
        allowed_employee_ids.into_iter().collect()
    };

    Ok(Some(final_ids))
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/reports/attendance",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        AttendanceReportQuery
    ),
    responses(
        (status = 200, description = "Attendance report data")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn get_attendance_report(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(params): Query<AttendanceReportQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let workspace_id = ctx.workspace_id;

    // RBAC: Check permissions
    let is_global = ctx.has_permission(Permission::ReportsRead)
        || ctx.has_permission(Permission::OvertimeApprove)
        || ctx.has_permission(Permission::AttendanceSummariesRead);

    if !is_global {
        ctx.require_permission(Permission::OvertimeClaimOnBehalf)?;
    }

    let report_type = params.r#type.as_deref().unwrap_or("summary");

    // Clamp client-supplied pagination before it can reach the SQL LIMIT clause.
    // Oversized limits are clamped, not rejected, so existing clients keep working.
    let pagination = SafePagination::from_params_with_default(
        params.page,
        params.limit,
        DEFAULT_REPORT_PAGE_LIMIT,
    );
    if pagination.was_clamped() {
        tracing::warn!(
            requested_limit = ?params.limit,
            effective_limit = %pagination.limit(),
            report_type = %report_type,
            "Attendance report limit exceeded the allowed range — clamped"
        );
    }

    let sort = params.sort.as_ref().and_then(|s| {
        serde_json::from_value::<AttendanceReportSort>(serde_json::Value::String(s.clone())).ok()
    });

    // Parse and validate the date range before doing any work — an unbounded
    // range fans out to one row per employee per day and can exhaust memory.
    let start_date = NaiveDate::parse_from_str(&params.start_date, "%Y-%m-%d")
        .map_err(|_e| AppError::validation(I18nKey::InvalidStartDateValue))?;
    let end_date = NaiveDate::parse_from_str(&params.end_date, "%Y-%m-%d")
        .map_err(|_e| AppError::validation(I18nKey::InvalidEndDateValue))?;
    validate_report_date_range(start_date, end_date)?;

    let requested_ids: Option<Vec<Uuid>> = params.employee_ids.as_ref().map(|s| {
        s.split(',')
            .filter_map(|id| Uuid::parse_str(id).ok())
            .collect()
    });

    // Resolve visible employee IDs (with behalf-claim filtering for non-global callers)
    let employee_ids = if is_global {
        requested_ids
    } else {
        match resolve_behalf_employee_ids(&state, &ctx, requested_ids).await? {
            Some(ids) => Some(ids),
            None => {
                return empty_report_json(
                    report_type,
                    Some(pagination.page()),
                    Some(pagination.limit()),
                )
            }
        }
    };

    let use_segments = params.segment_centric.unwrap_or(true)
        || state
            .payroll()
            .feature_flag_repo()
            .is_enabled_flag(&ctx.auth_token, "segment_centric_reports")
            .await
            .unwrap_or(false);

    if report_type == "daily" {
        let service = state.reports().service();
        let args = build_report_args(
            workspace_id,
            &params,
            start_date,
            end_date,
            employee_ids,
            None,
            pagination,
        );
        let report = if use_segments {
            service
                .get_segment_attendance_report(&ctx.auth_token, args)
                .await?
        } else {
            service
                .get_daily_attendance_report(&ctx.auth_token, args)
                .await?
        };
        Ok(Json(
            serde_json::to_value(report).map_err(|e| AppError::from(e.to_string()))?,
        ))
    } else {
        let args = build_report_args(
            workspace_id,
            &params,
            start_date,
            end_date,
            employee_ids,
            sort,
            pagination,
        );
        let summary = state
            .reports()
            .service()
            .get_workspace_attendance_summary(&ctx.auth_token, args)
            .await?;
        Ok(Json(
            serde_json::to_value(summary).map_err(|e| AppError::from(e.to_string()))?,
        ))
    }
}

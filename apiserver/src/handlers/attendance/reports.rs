use axum::{
    extract::{Query, State},
    response::Json,
};
use chrono::NaiveDate;

use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
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
#[allow(clippy::too_many_lines)]
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

    let start_date_str = &params.start_date;
    let end_date_str = &params.end_date;
    let report_type = params.r#type.as_deref().unwrap_or("summary");

    let page = params.page;
    let limit = params.limit;
    let sort = params.sort.as_ref().and_then(|s| {
        serde_json::from_value::<crate::models::AttendanceReportSort>(serde_json::Value::String(
            s.clone(),
        ))
        .ok()
    });

    let mut employee_ids = params.employee_ids.as_ref().map(|s| {
        s.split(',')
            .filter_map(|id| Uuid::parse_str(id).ok())
            .collect::<Vec<Uuid>>()
    });
    let department_id = params.department_id;

    // Apply behalf claim restrictions if not global
    if !is_global {
        let caller_assignments = state
            .attendance()
            .behalf_assignment_repo()
            .list_assignments_for_assignee(workspace_id, ctx.user_id, "overtime_claim")
            .await?;

        if caller_assignments.is_empty() {
            if report_type == "daily" {
                let report =
                    crate::models::PaginatedResponse::<crate::models::DailyAttendanceReport> {
                        data: Vec::new(),
                        total: 0,
                        page: page.unwrap_or(1),
                        limit: limit.unwrap_or(20),
                    };
                return Ok(Json(
                    serde_json::to_value(report).map_err(|e| AppError::from(e.to_string()))?,
                ));
            }
            let summary = crate::models::PaginatedResponse::<crate::models::AttendanceSummary> {
                data: Vec::new(),
                total: 0,
                page: page.unwrap_or(1),
                limit: limit.unwrap_or(20),
            };
            return Ok(Json(
                serde_json::to_value(summary).map_err(|e| AppError::from(e.to_string()))?,
            ));
        }

        let mut allowed_employee_ids = std::collections::HashSet::new();
        let mut assigned_dept_ids = Vec::new();

        for a in caller_assignments {
            if let Some(emp_id) = a.target_employee_id {
                allowed_employee_ids.insert(emp_id);
            }
            if let Some(dept_id) = a.target_department_id {
                assigned_dept_ids.push(dept_id);
            }
        }

        if !assigned_dept_ids.is_empty() {
            let active_employees = state
                .employees()
                .repo()
                .get_active_employees(&ctx.auth_token)
                .await?;

            let dept_map = state
                .employees()
                .repo()
                .get_employee_departments_at(&ctx.auth_token, chrono::Utc::now().date_naive())
                .await?;

            for emp in active_employees {
                if let Some((Some(dept_id), _)) = dept_map.get(&emp.id) {
                    if assigned_dept_ids.contains(dept_id) {
                        allowed_employee_ids.insert(emp.id);
                    }
                }
            }
        }

        if let Some(ref req_ids) = employee_ids {
            let intersected: Vec<Uuid> = req_ids
                .iter()
                .filter(|id| allowed_employee_ids.contains(id))
                .copied()
                .collect();

            if intersected.is_empty() {
                if report_type == "daily" {
                    let report =
                        crate::models::PaginatedResponse::<crate::models::DailyAttendanceReport> {
                            data: Vec::new(),
                            total: 0,
                            page: page.unwrap_or(1),
                            limit: limit.unwrap_or(20),
                        };
                    return Ok(Json(
                        serde_json::to_value(report).map_err(|e| AppError::from(e.to_string()))?,
                    ));
                }
                let summary = crate::models::PaginatedResponse::<crate::models::AttendanceSummary> {
                    data: Vec::new(),
                    total: 0,
                    page: page.unwrap_or(1),
                    limit: limit.unwrap_or(20),
                };
                return Ok(Json(
                    serde_json::to_value(summary).map_err(|e| AppError::from(e.to_string()))?,
                ));
            }
            employee_ids = Some(intersected);
        } else {
            employee_ids = Some(allowed_employee_ids.into_iter().collect());
        }
    }

    let start_date = NaiveDate::parse_from_str(start_date_str, "%Y-%m-%d")
        .map_err(|_e| AppError::Validation("Invalid start_date format".to_string()))?;
    let end_date = NaiveDate::parse_from_str(end_date_str, "%Y-%m-%d")
        .map_err(|_e| AppError::Validation("Invalid end_date format".to_string()))?;

    if report_type == "daily" {
        let report = state
            .reports()
            .service()
            .get_daily_attendance_report(crate::services::report_service::AttendanceReportArgs {
                workspace_id,
                start_date,
                end_date,
                page,
                limit,
                employee_ids,
                department_id,
                sort: None,
            })
            .await?;

        Ok(Json(
            serde_json::to_value(report).map_err(|e| AppError::from(e.to_string()))?,
        ))
    } else {
        let summary = state
            .reports()
            .service()
            .get_workspace_attendance_summary(
                crate::services::report_service::AttendanceReportArgs {
                    workspace_id,
                    start_date,
                    end_date,
                    page,
                    limit,
                    employee_ids,
                    department_id,
                    sort,
                },
            )
            .await?;

        Ok(Json(
            serde_json::to_value(summary).map_err(|e| AppError::from(e.to_string()))?,
        ))
    }
}

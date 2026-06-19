use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;

use crate::AppState;

#[derive(Deserialize, utoipa::IntoParams)]
pub struct AttendanceLogQuery {
    pub employee_id: Option<Uuid>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/attendance/logs",
    responses(
        (status = 200, description = "Logs fetched", body = serde_json::Value)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        AttendanceLogQuery
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn get_attendance_logs(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<AttendanceLogQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let _workspace_id = ctx.workspace_id;

    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;

    let target_employment_id = if let Some(emp_id) = query.employee_id {
        let active_id = if let Some(employee) = state
            .employees()
            .repo()
            .get_employee_by_id(emp_id, &ctx.auth_token)
            .await?
        {
            employee.active_employment_id()
        } else {
            None
        };
        Some(active_id.unwrap_or_else(Uuid::nil))
    } else {
        None
    };

    let logs = state
        .attendance()
        .time_entry_repo()
        .get_paginated_logs(
            &ctx.auth_token,
            crate::models::TimeEntryLogFilter::new(
                target_employment_id,
                query.from,
                query.to,
                limit,
                offset,
            ),
        )
        .await?;

    let total = state
        .attendance()
        .time_entry_repo()
        .count_logs(&ctx.auth_token, target_employment_id, query.from, query.to)
        .await?;

    Ok(Json(serde_json::json!({
        "data": logs,
        "total": total,
        "page": page,
        "limit": limit,
    })))
}

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/attendance/raw-logs",
    responses(
        (status = 200, description = "Raw logs fetched", body = serde_json::Value)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        AttendanceLogQuery
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn get_raw_attendance_logs(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<AttendanceLogQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let workspace_id = ctx.workspace_id;

    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;

    let target_employment_id = if let Some(emp_id) = query.employee_id {
        let active_id = if let Some(employee) = state
            .employees()
            .repo()
            .get_employee_by_id(emp_id, &ctx.auth_token)
            .await?
        {
            employee.active_employment_id()
        } else {
            None
        };
        Some(active_id.unwrap_or_else(Uuid::nil))
    } else {
        None
    };

    let logs = state
        .attendance()
        .raw_log_repo()
        .get_paginated_raw_logs(
            workspace_id,
            target_employment_id,
            query.from,
            query.to,
            limit,
            offset,
        )
        .await?;

    let total = state
        .attendance()
        .raw_log_repo()
        .count_raw_logs(ctx.workspace_id, target_employment_id, query.from, query.to)
        .await?;

    Ok(Json(serde_json::json!({
        "data": logs,
        "total": total,
        "page": page,
        "limit": limit,
    })))
}

#[derive(Deserialize)]
pub struct ReprocessQuery {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/attendance/reprocess/{employee_id}",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("employee_id" = Uuid, Path, description = "Employee ID"),
        ("start_date" = NaiveDate, Query, description = "Start date"),
        ("end_date" = NaiveDate, Query, description = "End date")
    ),
    responses(
        (status = 200, description = "Attendance reprocessed")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id, employee_id = %__path_params.1))]
pub async fn force_reprocess_attendance(
    State(state): State<Arc<AppState>>,
    Path(__path_params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Query(query): Query<ReprocessQuery>,
) -> AppResult<StatusCode> {
    let (_, path_id) = __path_params;
    ctx.require_permission(Permission::AttendanceReprocess)?;

    const MAX_REPROCESS_DAYS: i64 = 90;

    let date_range = (query.end_date - query.start_date).num_days();
    if date_range < 0 {
        return Err(AppError::validation(
            crate::utils::i18n::I18nKey::InvalidDateRange,
        ));
    }
    if date_range > MAX_REPROCESS_DAYS {
        return Err(AppError::validation_args(
            crate::utils::i18n::I18nKey::Validation,
            vec![(
                "details",
                format!("Max reprocess range is {MAX_REPROCESS_DAYS} days"),
            )],
        ));
    }

    // Accept both employee_id and employment_id for resilience.
    // Try employee_id first; if not found, fall back to employment_id lookup.
    tracing::info!(
        path_id = %path_id,
        workspace_id = %ctx.workspace_id,
        auth_workspace = %ctx.auth_token.workspace_id(),
        "Reprocess: looking up employee"
    );

    let emp_result = state
        .employees()
        .repo()
        .get_employee_by_id(path_id, &ctx.auth_token)
        .await;

    tracing::info!(
        path_id = %path_id,
        found = emp_result.as_ref().map(Option::is_some).unwrap_or(false),
        err = ?emp_result.as_ref().err(),
        "Reprocess: employee lookup result"
    );

    let employee = if let Some(emp) = emp_result? {
        emp
    } else {
        let employment_result = state
            .employees()
            .employment_repo()
            .get_employment_by_id(&ctx.auth_token, path_id)
            .await;

        tracing::info!(
            path_id = %path_id,
            found_employment = employment_result.as_ref().map(Option::is_some).unwrap_or(false),
            err = ?employment_result.as_ref().err(),
            "Reprocess: employment fallback result"
        );

        if let Some(employment) = employment_result? {
            state
                .employees()
                .repo()
                .get_employee_by_id(employment.employee_id(), &ctx.auth_token)
                .await?
                .ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::EmployeeNotFound))?
        } else {
            return Err(AppError::not_found(
                crate::utils::i18n::I18nKey::EmployeeNotFound,
            ));
        }
    };

    let active_employment_id = employee
        .active_employment_id()
        .ok_or_else(|| AppError::validation(crate::utils::i18n::I18nKey::NoActiveEmployment))?;

    let tz: chrono_tz::Tz = state
        .employees()
        .repo()
        .get_employee_timezone_fallback(&ctx.auth_token, active_employment_id)
        .await?;

    state
        .attendance()
        .pairing_service()
        .reprocess_attendance_range(
            ctx.workspace_id,
            active_employment_id,
            query.start_date,
            query.end_date,
            tz,
        )
        .await
        .map_err(|e| AppError::from(format!("Processing failed: {e}")))?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/attendance/reprocess-all",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("start_date" = NaiveDate, Query, description = "Start date"),
        ("end_date" = NaiveDate, Query, description = "End date")
    ),
    responses(
        (status = 200, description = "Bulk reprocess queued", body = serde_json::Value)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn force_reprocess_all_attendance(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<ReprocessQuery>,
) -> AppResult<Json<serde_json::Value>> {
    ctx.require_permission(Permission::AttendanceReprocess)?;

    const MAX_REPROCESS_DAYS: i64 = 90;

    let date_range = (query.end_date - query.start_date).num_days();
    if date_range < 0 {
        return Err(AppError::validation(
            crate::utils::i18n::I18nKey::InvalidDateRange,
        ));
    }
    if date_range > MAX_REPROCESS_DAYS {
        return Err(AppError::validation_args(
            crate::utils::i18n::I18nKey::Validation,
            vec![(
                "details",
                format!("Max reprocess range is {MAX_REPROCESS_DAYS} days"),
            )],
        ));
    }

    let queue = state.infra().job_queue()?;

    // Fetch all active employees
    let employees = state
        .employees()
        .repo()
        .get_active_employees(&ctx.auth_token)
        .await?;

    tracing::info!(
        employee_count = employees.len(),
        workspace_id = %ctx.workspace_id,
        employee_ids = ?employees.iter().map(crate::models::Employee::id).collect::<Vec<_>>(),
        "Found employees for bulk reprocess"
    );

    if employees.is_empty() {
        return Err(AppError::validation(
            crate::utils::i18n::I18nKey::NoActiveEmployeesFound,
        ));
    }

    let parent_job_id = crate::utils::id::new_id();
    let total_batches = u32::try_from(employees.len()).unwrap_or(0);

    tracing::info!(
        workspace_id = %ctx.workspace_id,
        employee_count = total_batches,
        job_id = %parent_job_id,
        "Enqueuing bulk reprocess jobs"
    );

    let meta = crate::queue::JobMetadata::with_date_range(
        parent_job_id,
        total_batches,
        ctx.workspace_id,
        ctx.user_id,
        Some(crate::queue::TaskType::ReprocessAttendance),
        None, // No report type
        crate::queue::DateRange {
            start: query.start_date,
            end: query.end_date,
        },
        Some(serde_json::json!({
            "start_date": query.start_date.to_string(),
            "end_date": query.end_date.to_string(),
            "employee_count": total_batches
        })),
    );

    queue
        .create_job_metadata(&meta)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create job metadata: {e}")))?;

    let jobs: Vec<crate::queue::Job> = employees
        .into_iter()
        .enumerate()
        .map(|(idx, emp)| {
            crate::queue::Job::with_priority(
                parent_job_id,
                u32::try_from(idx).unwrap_or(0),
                crate::queue::TaskType::ReprocessAttendance,
                ctx.workspace_id,
                crate::queue::JobPayload::ReprocessAttendance(crate::queue::ReprocessPayload {
                    employee_id: emp.id(),
                    date_range: crate::queue::DateRange {
                        start: query.start_date,
                        end: query.end_date,
                    },
                }),
                crate::queue::JobPriority::Normal,
            )
        })
        .collect();

    tracing::debug!(job_count = jobs.len(), "Enqueuing jobs to Redis");

    queue
        .enqueue_jobs(jobs)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to enqueue jobs: {e}")))?;

    tracing::info!(job_id = %parent_job_id, "Bulk reprocess jobs enqueued successfully");

    Ok(Json(serde_json::json!({
        "message": "Reprocess jobs queued successfully",
        "job_id": parent_job_id
    })))
}

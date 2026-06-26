use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::NaiveDate;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{AttendanceDailySummary, OtStatus, PaginatedResponse};
use crate::utils::i18n::I18nKey;
use crate::AppState;

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ClaimOtRequest {
    pub date: NaiveDate,
    pub claimed_minutes: i32,
    pub reason: Option<String>,
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/attendance/claim-ot/{employee_id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employee_id" = Uuid, Path, description = "Employee ID")
    ),
    request_body = ClaimOtRequest,
    responses(
        (status = 200, description = "Overtime claimed")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id, employee_id = %__path_params.1))]
pub async fn claim_ot(
    State(state): State<Arc<AppState>>,
    Path(__path_params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<ClaimOtRequest>,
) -> AppResult<StatusCode> {
    let (_, employee_id) = __path_params;
    // 1. Verify ownership (Employee can only claim for themselves) or has OvertimeClaim permission
    let employee = state
        .employees()
        .repo()
        .get_employee_by_id(employee_id, &ctx.auth_token)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::EmployeeNotFound))?;

    if let Some(uid) = employee.user_id() {
        if uid != ctx.user_id {
            check_claim_behalf_scope(&state, &ctx, employee_id).await?;
        }
    } else {
        check_claim_behalf_scope(&state, &ctx, employee_id).await?;
    }

    let active_employment_id = employee
        .active_employment_id()
        .ok_or_else(|| AppError::validation(I18nKey::NoActiveEmployment))?;

    state
        .attendance()
        .summary_service()
        .claim_ot(
            ctx.workspace_id,
            active_employment_id,
            req.date,
            req.claimed_minutes,
            req.reason,
        )
        .await?;

    let summary = state
        .attendance()
        .attendance_daily_summary_repo()
        .get_summary(&ctx.auth_token, active_employment_id, req.date)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::AttendanceSummaryNotFound))?;

    state
        .core()
        .approval_workflow_service()
        .submit_to_workflow(
            &ctx.auth_token,
            "ot_claim".to_string(),
            summary.id,
            employee_id,
        )
        .await?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ApproveOtRequest {
    pub date: NaiveDate,
    pub approved_minutes: i32,
    pub manager_note: Option<String>,
    pub status: String, // "approved" or "rejected"
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/attendance/approve-ot/{employee_id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employee_id" = Uuid, Path, description = "Employee ID")
    ),
    request_body = ApproveOtRequest,
    responses(
        (status = 200, description = "Overtime approved/rejected")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id, employee_id = %__path_params.1))]
pub async fn approve_ot(
    State(state): State<Arc<AppState>>,
    Path(__path_params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<ApproveOtRequest>,
) -> AppResult<StatusCode> {
    let (_, employee_id) = __path_params;
    // RBAC: Only users with OvertimeApprove permission, scoped to department if TeamLead
    let target_dept_id =
        crate::handlers::common::current_department_id(&state, &ctx.auth_token, employee_id).await;

    ctx.require_scoped_permission(Permission::OvertimeApprove, target_dept_id)?;

    let status_enum = match req.status.as_str() {
        "approved" => crate::models::OtStatus::Approved,
        "rejected" => crate::models::OtStatus::Rejected,
        _ => return Err(AppError::validation(I18nKey::InvalidStatus)),
    };

    let employee = state
        .employees()
        .repo()
        .get_employee_by_id(employee_id, &ctx.auth_token)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::EmployeeNotFound))?;

    let active_employment_id = employee
        .active_employment_id()
        .ok_or_else(|| AppError::validation(I18nKey::NoActiveEmployment))?;

    state
        .attendance()
        .summary_service()
        .approve_ot(
            ctx.workspace_id,
            active_employment_id,
            req.date,
            req.approved_minutes,
            req.manager_note,
            status_enum,
        )
        .await?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct SummaryQuery {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub employee_id: Option<Uuid>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub ot_status: Option<String>,
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/attendance/summaries",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("start_date" = NaiveDate, Query, description = "Start date"),
        ("end_date" = NaiveDate, Query, description = "End date"),
        ("employee_id" = Option<Uuid>, Query, description = "Optional Employee ID"),
        ("page" = Option<i64>, Query, description = "Page number (1-based)"),
        ("limit" = Option<i64>, Query, description = "Items per page"),
        ("ot_status" = Option<String>, Query, description = "Filter by OT status")
    ),
    responses(
        (status = 200, description = "Attendance summaries")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn get_attendance_summaries(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<SummaryQuery>,
) -> AppResult<Json<PaginatedResponse<AttendanceDailySummary>>> {
    // RBAC: if user has AttendanceSummariesRead, they can query any employee
    // Otherwise, they can only query their own data
    let has_read_perm = ctx.has_permission(Permission::AttendanceSummariesRead);

    let target_employee_id = if has_read_perm {
        query.employee_id
    } else {
        let my_id = state
            .employees()
            .repo()
            .get_employee_id_by_user_id(&ctx.auth_token, ctx.user_id)
            .await?
            .ok_or_else(|| AppError::forbidden(I18nKey::Forbidden))?;

        if let Some(req_id) = query.employee_id {
            if req_id != my_id {
                return Err(AppError::forbidden(I18nKey::Forbidden));
            }
        }
        Some(my_id)
    };

    let target_employment_id = if let Some(emp_id) = target_employee_id {
        let employee = state
            .employees()
            .repo()
            .get_employee_by_id(emp_id, &ctx.auth_token)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::EmployeeNotFound))?;

        let active_emp_id = employee.active_employment_id();
        if active_emp_id.is_none() {
            return Ok(Json(PaginatedResponse {
                data: vec![],
                total: 0,
                page: query.page.unwrap_or(1),
                limit: query.limit.unwrap_or(50),
            }));
        }
        active_emp_id
    } else {
        None
    };

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 3000);

    let ot_status = match query.ot_status.as_deref() {
        Some("pending_manager") => Some(OtStatus::PendingManager),
        Some("pending_employee") => Some(OtStatus::PendingEmployee),
        Some("approved") => Some(OtStatus::Approved),
        Some("rejected") => Some(OtStatus::Rejected),
        Some("none") => Some(OtStatus::None),
        _ => None,
    };

    let (summaries, total) = state
        .attendance()
        .attendance_daily_summary_repo()
        .get_summaries_paginated(
            &ctx.auth_token,
            target_employment_id,
            query.start_date,
            query.end_date,
            ot_status,
            page,
            limit,
        )
        .await?;

    Ok(Json(PaginatedResponse {
        data: summaries,
        total,
        page,
        limit,
    }))
}

async fn check_claim_behalf_scope(
    state: &AppState,
    ctx: &MemberContext,
    target_employee_id: Uuid,
) -> AppResult<()> {
    crate::handlers::common::check_behalf_scope(
        state,
        ctx,
        &[
            Permission::OvertimeApprove,
            Permission::OvertimeClaimOnBehalf,
        ],
        "overtime_claim",
        target_employee_id,
    )
    .await
}

use crate::models::{BehalfAssignment, CreateBehalfAssignmentRequest};

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/attendance/claim-ot/behalf-assignments",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = CreateBehalfAssignmentRequest,
    responses(
        (status = 200, description = "Assignment created", body = BehalfAssignment)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn create_behalf_assignment(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    ctx: MemberContext,
    Json(req): Json<CreateBehalfAssignmentRequest>,
) -> AppResult<Json<BehalfAssignment>> {
    let result = crate::handlers::common::create_behalf_assignment_handler(
        &state,
        &ctx,
        &req,
        Permission::OvertimePoliciesManage,
        "overtime_claim",
    )
    .await?;
    Ok(Json(result))
}

#[utoipa::path(
    delete,
    path = "/v1/workspaces/{ws_id}/attendance/claim-ot/behalf-assignments/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Assignment ID")
    ),
    responses(
        (status = 200, description = "Assignment deleted")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id, assignment_id = %params.1))]
pub async fn delete_behalf_assignment(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    let (_ws_id, id) = params;
    crate::handlers::common::delete_behalf_assignment_handler(
        &state,
        &ctx,
        id,
        Permission::OvertimePoliciesManage,
    )
    .await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/attendance/claim-ot/behalf-assignments",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "List of assignments", body = Vec<BehalfAssignment>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn list_behalf_assignments(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<BehalfAssignment>>> {
    let (result, _) = crate::handlers::common::list_behalf_assignments_handler(
        &state,
        &ctx,
        Permission::OvertimePoliciesManage,
        "overtime_claim",
        None,
        None,
    )
    .await?;
    Ok(Json(result))
}

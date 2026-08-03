use crate::{
    domain::permissions::Permission,
    errors::{AppError, AppResult},
    handlers::common::MemberContext,
    models::{
        BehalfAssignment, CreateBehalfAssignmentRequest, Employee, OtRequestStatus,
        OvertimeRequest, OvertimeRequestEmployee, PaginatedResponse,
    },
    utils::i18n::I18nKey,
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use chrono::{NaiveDate, NaiveTime};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct PaginatedOvertimeRequests {
    pub data: Vec<OvertimeRequest>,
    pub total: usize,
    pub page: i64,
    pub limit: i64,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct PaginatedBehalfAssignments {
    pub data: Vec<BehalfAssignment>,
    pub total: usize,
    pub page: i64,
    pub limit: i64,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CreateOvertimeRequestPayload {
    pub department_id: Option<Uuid>,
    pub planned_date: NaiveDate,
    pub planned_start_time: Option<NaiveTime>,
    pub planned_end_time: Option<NaiveTime>,
    pub planned_minutes: i32,
    pub reason: String,
    pub employees: Vec<Uuid>,
    pub attachment_file_ids: Option<Vec<Uuid>>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct OvertimeRequestDetail {
    pub request: OvertimeRequest,
    pub employees: Vec<OvertimeRequestEmployee>,
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/overtime/requests",
    request_body = CreateOvertimeRequestPayload,
    responses(
        (status = 200, description = "Overtime request created", body = OvertimeRequest)
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn create_ot_request(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateOvertimeRequestPayload>,
) -> AppResult<Json<OvertimeRequest>> {
    ctx.require_permission(Permission::OvertimeRequestCreate)?;

    let caller_employee_id = state
        .employees()
        .repo()
        .get_employee_id_by_user_id(&ctx.auth_token, ctx.user_id)
        .await?;

    for &emp_id in &req.employees {
        let is_on_behalf = match caller_employee_id {
            Some(caller_id) => caller_id != emp_id,
            None => true,
        };

        if is_on_behalf {
            check_request_behalf_scope(&state, &ctx, emp_id).await?;
        }
    }

    let attachment_file_ids = req
        .attachment_file_ids
        .map(|ids| ids.into_iter().map(Some).collect());

    let request = OvertimeRequest::new(
        crate::utils::id::new_id(),
        ctx.workspace_id,
        ctx.user_id,
        req.department_id,
        req.planned_date,
        req.planned_start_time,
        req.planned_end_time,
        req.planned_minutes,
        req.reason,
        OtRequestStatus::Draft,
        None,
        0,
        None,
        None,
        None,
        chrono::Utc::now(),
        chrono::Utc::now(),
        attachment_file_ids,
    );

    let employments = state
        .employees()
        .employment_repo()
        .get_active_employments_for_employees(&ctx.auth_token, &req.employees, req.planned_date)
        .await?;

    let mut employees = Vec::new();
    for emp_id in req.employees {
        let employment = employments
            .iter()
            .find(|e| e.employee_id() == emp_id)
            .ok_or_else(|| AppError::not_found(I18nKey::EmploymentNotFound))?;

        employees.push(OvertimeRequestEmployee::new(
            crate::utils::id::new_id(),
            request.id(),
            employment.id(),
            "pending".to_string(),
            None,
            None,
            None,
            None,
            chrono::Utc::now(),
        ));
    }

    let created = state
        .payroll()
        .overtime_request_service()
        .create_request(&ctx.auth_token, request, employees)
        .await?;

    Ok(Json(created))
}

#[derive(Deserialize, utoipa::IntoParams)]
pub struct ListOtRequestsQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/overtime/requests",
    responses(
        (status = 200, description = "List overtime requests", body = PaginatedOvertimeRequests)
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ListOtRequestsQuery
    )
)]
pub async fn list_ot_requests(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    axum::extract::Query(query): axum::extract::Query<ListOtRequestsQuery>,
) -> AppResult<Json<PaginatedResponse<OvertimeRequest>>> {
    ctx.require_permission(Permission::OvertimeRequestRead)?;

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 3000);

    let (requests, total) = state
        .payroll()
        .overtime_request_service()
        .list_requests(&ctx.auth_token, page, limit)
        .await?;

    Ok(Json(PaginatedResponse {
        data: requests,
        total: usize::try_from(total).unwrap_or(0),
        page,
        limit,

        truncated: None,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/overtime/requests/{id}",
    responses(
        (status = 200, description = "Overtime request details", body = OvertimeRequestDetail)
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Request ID")
    )
)]
pub async fn get_ot_request(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<Json<OvertimeRequestDetail>> {
    let (_, id) = params;
    ctx.require_permission(Permission::OvertimeRequestRead)?;

    let request = state
        .payroll()
        .overtime_request_service()
        .get_request(&ctx.auth_token, id)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::OvertimeRequestNotFound))?;

    let employees = state
        .payroll()
        .overtime_request_service()
        .get_request_employees(&ctx.auth_token, id)
        .await?;

    Ok(Json(OvertimeRequestDetail { request, employees }))
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/overtime/requests/{id}/submit",
    responses(
        (status = 200, description = "Overtime request submitted for approval")
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Request ID")
    )
)]
pub async fn submit_for_approval(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    let (_, id) = params;
    ctx.require_permission(Permission::OvertimeRequestCreate)?;

    state
        .payroll()
        .overtime_request_service()
        .update_request_status(
            &ctx.auth_token,
            id,
            OtRequestStatus::PendingApproval,
            Some(1),
        )
        .await?;

    let employees = state
        .payroll()
        .overtime_request_service()
        .get_request_employees(&ctx.auth_token, id)
        .await?;
    let beneficiary_employment_id = employees
        .first()
        .map(crate::models::overtime_request::OvertimeRequestEmployee::employment_id)
        .ok_or_else(|| AppError::validation(I18nKey::OvertimeRequestNoEmployees))?;

    let employment = state
        .employees()
        .employment_repo()
        .get_employment_by_id(&ctx.auth_token, beneficiary_employment_id)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::NotFound))?;
    let beneficiary_employee_id = employment.employee_id();

    state
        .core()
        .approval_workflow_service()
        .submit_to_workflow(
            &ctx.auth_token,
            "ot_request".to_string(),
            id,
            beneficiary_employee_id,
        )
        .await?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct EmployeeConsentRequest {
    pub status: String,
    pub decline_reason: Option<String>,
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/overtime/requests/{id}/consent/{emp_id}",
    request_body = EmployeeConsentRequest,
    responses(
        (status = 200, description = "Employee consent status updated")
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Request ID"),
        ("emp_id" = Uuid, Path, description = "Employee ID")
    )
)]
pub async fn employee_consent(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<EmployeeConsentRequest>,
) -> AppResult<StatusCode> {
    let (_, id, emp_id) = params;

    let caller_employee_id = state
        .employees()
        .repo()
        .get_employee_id_by_user_id(&ctx.auth_token, ctx.user_id)
        .await?;

    let is_self = caller_employee_id == Some(emp_id);
    if !is_self {
        check_request_behalf_scope(&state, &ctx, emp_id).await?;
    }

    state
        .payroll()
        .overtime_request_service()
        .update_employee_consent(&ctx.auth_token, id, emp_id, &req.status, req.decline_reason)
        .await?;

    Ok(StatusCode::OK)
}

async fn check_request_behalf_scope(
    state: &AppState,
    ctx: &MemberContext,
    target_employee_id: Uuid,
) -> AppResult<()> {
    crate::handlers::common::check_behalf_scope(
        state,
        ctx,
        &[Permission::OvertimeRequestCreateOnBehalf],
        "overtime_request",
        target_employee_id,
    )
    .await
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/overtime/requests/behalf-assignments",
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
pub async fn create_request_behalf_assignment(
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
        "overtime_request",
    )
    .await?;
    Ok(Json(result))
}

#[utoipa::path(
    delete,
    path = "/v1/workspaces/{ws_id}/overtime/requests/behalf-assignments/{id}",
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
pub async fn delete_request_behalf_assignment(
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

#[derive(Deserialize, utoipa::IntoParams)]
pub struct ListBehalfAssignmentsQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/overtime/requests/behalf-assignments",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ListBehalfAssignmentsQuery
    ),
    responses(
        (status = 200, description = "List of assignments", body = PaginatedBehalfAssignments)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_request_behalf_assignments(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    ctx: MemberContext,
    axum::extract::Query(query): axum::extract::Query<ListBehalfAssignmentsQuery>,
) -> AppResult<Json<PaginatedResponse<BehalfAssignment>>> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 3000);

    let (result, total) = crate::handlers::common::list_behalf_assignments_handler(
        &state,
        &ctx,
        Permission::OvertimePoliciesManage,
        "overtime_request",
        Some(page),
        Some(limit),
    )
    .await?;

    Ok(Json(PaginatedResponse {
        data: result,
        total: usize::try_from(total).unwrap_or(0),
        page,
        limit,

        truncated: None,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/overtime/requests/behalf-assignments/my-targets",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "Target employees for OT request behalf", body = Vec<Employee>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_my_request_behalf_targets(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<crate::models::Employee>>> {
    let result =
        crate::handlers::common::get_my_behalf_targets_handler(&state, &ctx, "overtime_request")
            .await?;
    Ok(Json(result))
}

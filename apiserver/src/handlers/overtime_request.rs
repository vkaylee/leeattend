use crate::{
    domain::permissions::Permission,
    errors::{AppError, AppResult},
    handlers::common::MemberContext,
    models::{OtRequestStatus, OvertimeRequest, OvertimeRequestEmployee},
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

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CreateOvertimeRequestPayload {
    pub department_id: Option<Uuid>,
    pub planned_date: NaiveDate,
    pub planned_start_time: Option<NaiveTime>,
    pub planned_end_time: Option<NaiveTime>,
    pub planned_minutes: i32,
    pub reason: String,
    pub employees: Vec<Uuid>,
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

    let request = OvertimeRequest {
        id: crate::utils::id::new_id(),
        workspace_id: ctx.workspace_id,
        requested_by_user_id: ctx.user_id,
        department_id: req.department_id,
        planned_date: req.planned_date,
        planned_start_time: req.planned_start_time,
        planned_end_time: req.planned_end_time,
        planned_minutes: req.planned_minutes,
        reason: req.reason,
        status: OtRequestStatus::Draft,
        workflow_id: None,
        current_step: 0,
        quota_warning: None,
        reconciled_at: None,
        reconciled_by: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let mut employees = Vec::new();
    for emp_id in req.employees {
        employees.push(OvertimeRequestEmployee {
            id: crate::utils::id::new_id(),
            overtime_request_id: request.id,
            employee_id: emp_id,
            consent_status: "pending".to_string(),
            consent_at: None,
            decline_reason: None,
            actual_ot_minutes: None,
            matched_summary_id: None,
            created_at: chrono::Utc::now(),
        });
    }

    let created = state
        .payroll
        .overtime_request_service
        .create_request(&ctx.auth_token, request, employees)
        .await?;

    Ok(Json(created))
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/overtime/requests",
    responses(
        (status = 200, description = "List overtime requests", body = [OvertimeRequest])
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn list_ot_requests(
    State(_state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<OvertimeRequest>>> {
    ctx.require_permission(Permission::OvertimeRequestRead)?;

    // We haven't implemented list_requests yet in repo
    Ok(Json(vec![]))
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
        .payroll
        .overtime_request_service
        .get_request(&ctx.auth_token, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Request not found".into()))?;

    let employees = state
        .payroll
        .overtime_request_service
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
        .payroll
        .overtime_request_service
        .update_request_status(
            &ctx.auth_token,
            id,
            OtRequestStatus::PendingApproval,
            Some(1),
        )
        .await?;

    let employees = state
        .payroll
        .overtime_request_service
        .get_request_employees(&ctx.auth_token, id)
        .await?;
    let beneficiary_employee_id = employees.first().map(|e| e.employee_id).ok_or_else(|| {
        AppError::Validation("Overtime request has no employees assigned".to_string())
    })?;

    state
        .core
        .approval_workflow_service
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
    state
        .payroll
        .overtime_request_service
        .update_employee_consent(&ctx.auth_token, id, emp_id, &req.status, req.decline_reason)
        .await?;

    Ok(StatusCode::OK)
}

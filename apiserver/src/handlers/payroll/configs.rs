use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::AppResult;
use crate::handlers::common::MemberContext;
use crate::models::{
    DepartmentPayrollConfig, EmployeePayrollConfig, SetDepartmentPayrollConfigRequest,
    SetEmployeePayrollConfigRequest,
};
use crate::AppState;

/// POST /workspaces/{ws_id}/departments/{id}/payroll-config
#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/departments/{id}/payroll-config",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Department ID")
    ),
    request_body = SetDepartmentPayrollConfigRequest,
    responses(
        (status = 201, description = "Set department payroll config", body = DepartmentPayrollConfig)
    )
)]
pub async fn set_department_payroll_config(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<SetDepartmentPayrollConfigRequest>,
) -> AppResult<(StatusCode, Json<DepartmentPayrollConfig>)> {
    let (_, department_id) = params;
    let config = state
        .payroll()
        .department_repo()
        .upsert_department_payroll_config(
            &ctx.auth_token,
            department_id,
            req.std_working_days,
            req.hours_per_day,
            req.effective_date,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(config)))
}

/// POST /workspaces/{ws_id}/employees/{id}/payroll-config
#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/employees/{id}/payroll-config",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employee ID")
    ),
    request_body = SetEmployeePayrollConfigRequest,
    responses(
        (status = 201, description = "Set employee payroll config", body = EmployeePayrollConfig)
    )
)]
pub async fn set_employee_payroll_config(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<SetEmployeePayrollConfigRequest>,
) -> AppResult<(StatusCode, Json<EmployeePayrollConfig>)> {
    let (_, employee_id) = params;
    // Resolve active employment_id from employee_id
    let employee = state
        .employees()
        .repo()
        .get_employee_by_id(employee_id, &ctx.auth_token)
        .await?
        .ok_or_else(|| crate::errors::AppError::not_found(crate::utils::i18n::I18nKey::EmployeeNotFound))?;
    let employment_id = employee
        .active_employment_id()
        .ok_or_else(|| crate::errors::AppError::validation_raw("No active employment found"))?;

    let config = state
        .employees()
        .repo()
        .upsert_employee_payroll_config(
            &ctx.auth_token,
            employment_id,
            req.std_working_days,
            req.hours_per_day,
            req.effective_date,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(config)))
}

/// GET /workspaces/{ws_id}/departments/{id}/payroll-config
#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/departments/{id}/payroll-config",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Department ID")
    ),
    responses(
        (status = 200, description = "Department payroll configs history list", body = [DepartmentPayrollConfig])
    )
)]
pub async fn get_department_payroll_configs(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<DepartmentPayrollConfig>>> {
    let (_, department_id) = params;
    let configs = state
        .payroll()
        .department_repo()
        .get_department_payroll_configs(&ctx.auth_token, department_id)
        .await?;

    Ok(Json(configs))
}

/// GET /workspaces/{ws_id}/employees/{id}/payroll-config
#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/employees/{id}/payroll-config",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employee ID")
    ),
    responses(
        (status = 200, description = "Employee payroll configs history list", body = [EmployeePayrollConfig])
    )
)]
pub async fn get_employee_payroll_configs(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<EmployeePayrollConfig>>> {
    let (_, employee_id) = params;
    // Resolve active employment_id from employee_id
    let employee = state
        .employees()
        .repo()
        .get_employee_by_id(employee_id, &ctx.auth_token)
        .await?
        .ok_or_else(|| crate::errors::AppError::not_found(crate::utils::i18n::I18nKey::EmployeeNotFound))?;
    let employment_id = employee
        .active_employment_id()
        .ok_or_else(|| crate::errors::AppError::validation_raw("No active employment found"))?;

    let configs = state
        .employees()
        .repo()
        .get_employee_payroll_configs(&ctx.auth_token, employment_id)
        .await?;

    Ok(Json(configs))
}


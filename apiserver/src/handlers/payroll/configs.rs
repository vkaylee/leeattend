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

pub async fn set_department_payroll_config(
    Path((_, department_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
    Json(req): Json<SetDepartmentPayrollConfigRequest>,
) -> AppResult<(StatusCode, Json<DepartmentPayrollConfig>)> {
    let config = state
        .payroll
        .department_repo
        .upsert_department_payroll_config(
            department_id,
            req.std_working_days,
            req.hours_per_day,
            req.effective_date,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(config)))
}

pub async fn set_employee_payroll_config(
    Path((_, employee_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
    Json(req): Json<SetEmployeePayrollConfigRequest>,
) -> AppResult<(StatusCode, Json<EmployeePayrollConfig>)> {
    let config = state
        .employees
        .repo
        .upsert_employee_payroll_config(
            employee_id,
            req.std_working_days,
            req.hours_per_day,
            req.effective_date,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(config)))
}

pub async fn get_department_payroll_configs(
    Path((_, department_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<DepartmentPayrollConfig>>> {
    let configs = state
        .payroll
        .department_repo
        .get_department_payroll_configs(workspace_id, department_id)
        .await?;

    Ok(Json(configs))
}

pub async fn get_employee_payroll_configs(
    Path((_, employee_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<EmployeePayrollConfig>>> {
    let configs = state
        .employees
        .repo
        .get_employee_payroll_configs(workspace_id, employee_id)
        .await?;

    Ok(Json(configs))
}

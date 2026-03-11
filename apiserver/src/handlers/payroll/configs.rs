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
    let config = state.payroll.department_repo.upsert_department_payroll_config(department_id,
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
    let config = state.employees.repo.upsert_employee_payroll_config(employee_id,
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
    let configs = sqlx::query_as!(
        crate::models::DepartmentPayrollConfig,
        r#"SELECT dpc.id, dpc.department_id, dpc.std_working_days, dpc.hours_per_day, dpc.effective_date, dpc.created_at as "created_at!"
           FROM department_payroll_configs dpc
           JOIN departments d ON d.id = dpc.department_id
           WHERE dpc.department_id = $1 AND d.workspace_id = $2
           ORDER BY dpc.effective_date DESC"#,
        department_id,
        workspace_id
    )
    .fetch_all(state.infra.db.api())
    .await?;

    Ok(Json(configs))
}

pub async fn get_employee_payroll_configs(
    Path((_, employee_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<EmployeePayrollConfig>>> {
    let configs = sqlx::query_as!(
        crate::models::EmployeePayrollConfig,
        r#"SELECT epc.id, epc.employee_id, epc.std_working_days, epc.hours_per_day, epc.effective_date, epc.created_at as "created_at!"
           FROM employee_payroll_configs epc
           JOIN employees e ON e.id = epc.employee_id
           WHERE epc.employee_id = $1 AND e.workspace_id = $2
           ORDER BY epc.effective_date DESC"#,
        employee_id,
        workspace_id
    )
    .fetch_all(state.infra.db.api())
    .await?;

    Ok(Json(configs))
}

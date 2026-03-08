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
    Path((_ws_id, department_id)): Path<(Uuid, Uuid)>,
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
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
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
    Path((_ws_id, department_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
) -> AppResult<Json<Vec<DepartmentPayrollConfig>>> {
    let configs = sqlx::query_as!(
        crate::models::DepartmentPayrollConfig,
        r#"SELECT id, department_id, std_working_days, hours_per_day, effective_date, created_at as "created_at!"
           FROM department_payroll_configs
           WHERE department_id = $1
           ORDER BY effective_date DESC"#,
        department_id
    )
    .fetch_all(state.infra.db.api())
    .await?;

    Ok(Json(configs))
}

pub async fn get_employee_payroll_configs(
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
) -> AppResult<Json<Vec<EmployeePayrollConfig>>> {
    let configs = sqlx::query_as!(
        crate::models::EmployeePayrollConfig,
        r#"SELECT id, employee_id, std_working_days, hours_per_day, effective_date, created_at as "created_at!"
           FROM employee_payroll_configs
           WHERE employee_id = $1
           ORDER BY effective_date DESC"#,
        employee_id
    )
    .fetch_all(state.infra.db.api())
    .await?;

    Ok(Json(configs))
}

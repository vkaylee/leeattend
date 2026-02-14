use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::handlers::common::MemberContext;
use crate::models::{SetDepartmentPayrollConfigRequest, SetEmployeePayrollConfigRequest};
use crate::AppState;

pub async fn set_department_payroll_config(
    Path((_ws_id, department_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
    Json(req): Json<SetDepartmentPayrollConfigRequest>,
) -> impl IntoResponse {
    match crate::repositories::department_repository::upsert_department_payroll_config(
        &state.db,
        department_id,
        req.std_working_days,
        req.hours_per_day,
        req.effective_date,
    )
    .await
    {
        Ok(config) => (StatusCode::CREATED, Json(config)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn set_employee_payroll_config(
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
    Json(req): Json<SetEmployeePayrollConfigRequest>,
) -> impl IntoResponse {
    match crate::repositories::employee_repository::upsert_employee_payroll_config(
        &state.db,
        employee_id,
        req.std_working_days,
        req.hours_per_day,
        req.effective_date,
    )
    .await
    {
        Ok(config) => (StatusCode::CREATED, Json(config)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn get_department_payroll_configs(
    Path((_ws_id, department_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
) -> impl IntoResponse {
    match sqlx::query_as!(
        crate::models::DepartmentPayrollConfig,
        r#"SELECT id, department_id, std_working_days, hours_per_day, effective_date, created_at as "created_at!"
           FROM department_payroll_configs
           WHERE department_id = $1
           ORDER BY effective_date DESC"#,
        department_id
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(configs) => Json::<Vec<crate::models::DepartmentPayrollConfig>>(configs).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn get_employee_payroll_configs(
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
) -> impl IntoResponse {
    match sqlx::query_as!(
        crate::models::EmployeePayrollConfig,
        r#"SELECT id, employee_id, std_working_days, hours_per_day, effective_date, created_at as "created_at!"
           FROM employee_payroll_configs
           WHERE employee_id = $1
           ORDER BY effective_date DESC"#,
        employee_id
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(configs) => Json::<Vec<crate::models::EmployeePayrollConfig>>(configs).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

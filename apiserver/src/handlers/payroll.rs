use axum::{
    extract::{State, Path, Query},
    response::{IntoResponse, Json},
    http::StatusCode,
};
use std::sync::Arc;
use uuid::Uuid;
use serde::Deserialize;

use crate::AppState;
use crate::handlers::common::MemberContext;
use crate::models::{CreateSalaryHistoryRequest};

// --- Payroll Handlers ---

pub async fn get_salary_history(
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
) -> impl IntoResponse {
    match crate::repositories::payroll_repository::get_salary_history(&state.db, employee_id).await
    {
        Ok(history) => Json(history).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn add_salary_history(
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
    Json(req): Json<CreateSalaryHistoryRequest>,
) -> impl IntoResponse {
    match crate::repositories::payroll_repository::add_salary_history(
        &state.db,
        employee_id,
        req.base_salary,
        req.salary_type,
        req.currency,
        req.effective_date,
    )
    .await
    {
        Ok(history) => (StatusCode::CREATED, Json(history)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct PayrollQuery {
    pub year: i32,
    pub month: u32,
}

pub async fn calculate_payroll(
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(query): Query<PayrollQuery>,
) -> impl IntoResponse {
    match crate::services::payroll_service::PayrollService::calculate_monthly_payslip(
        &state.db,
        employee_id,
        workspace_id,
        query.year,
        query.month,
    )
    .await
    {
        Ok(payslip) => Json(payslip).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn get_employee_payslips(
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
) -> impl IntoResponse {
    match crate::repositories::payroll_repository::get_payslips_by_employee(&state.db, employee_id)
        .await
    {
        Ok(payslips) => Json(payslips).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::handlers::common::MemberContext;
use crate::models::{
    AssignSalaryComponentRequest, CreateSalaryComponentRequest, CreateSalaryHistoryRequest,
    UpdatePayrollSettingsRequest,
};
use crate::AppState;

// --- Payroll Settings Handlers ---

pub async fn get_payroll_settings(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
) -> impl IntoResponse {
    match crate::repositories::payroll_repository::get_payroll_settings(&state.db, workspace_id)
        .await
    {
        Ok(Some(settings)) => Json(settings).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn update_payroll_settings(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
    Json(req): Json<UpdatePayrollSettingsRequest>,
) -> impl IntoResponse {
    // Get current settings first to fill in missing fields if needed
    let current =
        crate::repositories::payroll_repository::get_payroll_settings(&state.db, workspace_id)
            .await
            .unwrap_or(None);

    let mode = req.std_working_day_mode.unwrap_or_else(|| {
        current
            .as_ref()
            .map(|s| s.std_working_day_mode.clone())
            .unwrap_or("fixed".to_string())
    });
    let fixed_days = req.fixed_std_days.unwrap_or_else(|| {
        current
            .as_ref()
            .map(|s| s.fixed_std_days)
            .unwrap_or(rust_decimal::Decimal::from(26))
    });
    let days_per_week = req.working_days_per_week.unwrap_or_else(|| {
        current
            .as_ref()
            .map(|s| s.working_days_per_week)
            .unwrap_or(6)
    });
    let hours_per_day = req.hours_per_day.unwrap_or_else(|| {
        current
            .as_ref()
            .map(|s| s.hours_per_day)
            .unwrap_or(rust_decimal::Decimal::from(8))
    });

    match crate::repositories::payroll_repository::update_payroll_settings(
        &state.db,
        workspace_id,
        mode,
        fixed_days,
        days_per_week,
        hours_per_day,
    )
    .await
    {
        Ok(settings) => Json(settings).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// --- Salary Component Handlers ---

pub async fn get_salary_components(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
) -> impl IntoResponse {
    match crate::repositories::payroll_repository::get_salary_component_definitions(
        &state.db,
        workspace_id,
    )
    .await
    {
        Ok(components) => Json(components).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn create_salary_component(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
    Json(req): Json<CreateSalaryComponentRequest>,
) -> impl IntoResponse {
    match crate::repositories::payroll_repository::create_salary_component_definition(
        &state.db,
        workspace_id,
        req.name,
        req.code,
        req.is_base_for_ot,
        req.is_taxable,
    )
    .await
    {
        Ok(component) => (StatusCode::CREATED, Json(component)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn get_employee_salary_components(
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
) -> impl IntoResponse {
    // Note: This returns latest components for effective_date = now
    let now = chrono::Utc::now().date_naive();
    match crate::repositories::payroll_repository::get_employee_salary_components(
        &state.db,
        employee_id,
        now,
    )
    .await
    {
        Ok(components) => {
            // Flatten the tuple (Component, Definition) for response
            let response: Vec<_> = components
                .into_iter()
                .map(|(c, d)| {
                    serde_json::json!({
                        "id": c.id,
                        "employee_id": c.employee_id,
                        "component_definition_id": c.component_definition_id,
                        "amount": c.amount,
                        "effective_date": c.effective_date,
                        "name": d.name,
                        "code": d.code,
                        "is_base_for_ot": d.is_base_for_ot,
                        "is_taxable": d.is_taxable,
                    })
                })
                .collect();
            Json(response).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn assign_employee_salary_component(
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
    Json(req): Json<AssignSalaryComponentRequest>,
) -> impl IntoResponse {
    match crate::repositories::payroll_repository::assign_employee_salary_component(
        &state.db,
        employee_id,
        req.component_definition_id,
        req.amount,
        req.effective_date,
    )
    .await
    {
        Ok(component) => (StatusCode::CREATED, Json(component)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// --- Existing Payroll Handlers ---

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

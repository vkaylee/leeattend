use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::handlers::common::MemberContext;
use crate::models::{
    AssignSalaryComponentRequest, CreateSalaryComponentRequest, Payslip, PayslipSegment,
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

pub async fn get_payroll_settings_history(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
) -> impl IntoResponse {
    match sqlx::query_as!(
        crate::models::PayrollSettings,
        r#"SELECT workspace_id, std_working_day_mode, fixed_std_days, working_days_per_week, hours_per_day, effective_date, created_at as "created_at!", updated_at as "updated_at!" 
           FROM payroll_settings 
           WHERE workspace_id = $1
           ORDER BY effective_date DESC"#,
        workspace_id
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(history) => Json(history).into_response(),
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
    let effective_date = req.effective_date.unwrap_or_else(|| Utc::now().date_naive());
    let current =
        crate::repositories::payroll_repository::get_payroll_settings_at(&state.db, workspace_id, effective_date)
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
        effective_date,
    )
    .await
    {
        Ok(settings) => Json(settings).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

use crate::models::{SetDepartmentPayrollConfigRequest, SetEmployeePayrollConfigRequest};

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
        Ok(configs) => Json(configs).into_response(),
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
        Ok(configs) => Json(configs).into_response(),
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

#[derive(Debug, Deserialize)]
pub struct PayrollQuery {
    pub year: i32,
    pub month: u32,
}

#[derive(Debug, Serialize)]
pub struct PayslipResponse {
    #[serde(flatten)]
    pub payslip: Payslip,
    pub segments: Vec<PayslipSegmentResponse>,
}

#[derive(Debug, Serialize)]
pub struct PayslipSegmentResponse {
    #[serde(flatten)]
    pub segment: PayslipSegment,
    // Add computed/default reason if needed
    pub reason: String,
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
        Ok(payslip) => {
             // Fetch segments
             let segments = crate::repositories::payroll_repository::get_payslip_segments(&state.db, payslip.id)
                .await
                .unwrap_or_default();
             
             let response = PayslipResponse {
                 payslip,
                 segments: segments.into_iter().map(|s| {
                     // Determine reason logic
                     let reason = if s.start_date.day() == 1 { "Standard".to_string() } 
                                  else if s.applied_base_salary > rust_decimal::Decimal::ZERO { "Salary Change".to_string() }
                                  else { "Adjustment".to_string() };
                     
                     PayslipSegmentResponse {
                         segment: s,
                         reason
                     }
                 }).collect()
             };
             Json(response).into_response()
        },
        Err(status) => status.into_response(),
    }
}

use chrono::Datelike;

pub async fn get_employee_payslips(
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
) -> impl IntoResponse {
    match crate::repositories::payroll_repository::get_payslips_by_employee(&state.db, employee_id)
        .await
    {
        Ok(payslips) => {
            let mut responses = Vec::new();
            for p in payslips {
                 let segments = crate::repositories::payroll_repository::get_payslip_segments(&state.db, p.id)
                    .await
                    .unwrap_or_default();
                    
                 responses.push(PayslipResponse {
                     payslip: p,
                     segments: segments.into_iter().map(|s| {
                         // Determine reason logic (simple heuristic)
                         // If it's the first segment and starts on 1st: Standard
                         // Else: Change
                         let reason = if s.start_date.day() == 1 { "Standard".to_string() } 
                                      else { "Salary Change".to_string() }; // MVP heuristic
                         PayslipSegmentResponse {
                             segment: s,
                             reason
                         }
                     }).collect()
                 });
            }
            Json(responses).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
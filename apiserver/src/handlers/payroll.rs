use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

use crate::handlers::common::MemberContext;
use crate::models::{
    AssignSalaryComponentRequest, BatchCalculatePayrollRequest, CreateSalaryComponentRequest,
    Payslip, PayslipSegment, UpdatePayrollSettingsRequest, IncentiveEvaluationResult,
    PayrollIncentivePolicyResponse, CreatePayrollIncentivePolicyRequest,
    PayrollIncentivePolicy, SalaryComponentWithDefinition,
};

pub async fn get_incentive_policies(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
) -> impl IntoResponse {
    match crate::repositories::payroll_repository::get_active_incentive_policies(&state.db, workspace_id)
        .await
    {
        Ok(policies) => {
            let mut response = Vec::new();
            for policy in policies {
                let tiers = crate::repositories::payroll_repository::get_policy_tiers(&state.db, policy.id)
                    .await
                    .unwrap_or_default();
                response.push(PayrollIncentivePolicyResponse {
                    policy,
                    tiers,
                });
            }
            Json(response).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn create_incentive_policy(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, workspace_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<CreatePayrollIncentivePolicyRequest>,
) -> impl IntoResponse {
    let policy_id = Uuid::new_v4();
    let res: Result<sqlx::postgres::PgRow, sqlx::Error> = sqlx::query(
        r#"INSERT INTO payroll_incentive_policies (id, workspace_id, name, base_amount, payout_type, is_active)
           VALUES ($1, $2, $3, $4, $5, TRUE)
           RETURNING id, workspace_id, name, base_amount, payout_type, is_active, created_at"#,
    )
    .bind(policy_id)
    .bind(workspace_id)
    .bind(&req.name)
    .bind(req.base_amount)
    .bind(&req.payout_type)
    .fetch_one(&state.db)
    .await;

    let row = match res {
        Ok(r) => r,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    for tier in &req.tiers {
        let tier_id = Uuid::new_v4();
        let tier_res = sqlx::query(
            r#"INSERT INTO payroll_incentive_tiers (id, policy_id, priority, max_late_early_count, max_unpaid_leave_days, max_sick_leave_days, max_total_leave_days, payout_amount)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        )
        .bind(tier_id)
        .bind(policy_id)
        .bind(tier.priority)
        .bind(tier.max_late_early_count)
        .bind(tier.max_unpaid_leave_days)
        .bind(tier.max_sick_leave_days)
        .bind(tier.max_total_leave_days)
        .bind(tier.payout_amount)
        .execute(&state.db)
        .await;

        if let Err(e) = tier_res {
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    }



    (StatusCode::CREATED, Json(PayrollIncentivePolicy {
        id: row.get("id"),
        workspace_id: row.get("workspace_id"),
        name: row.get("name"),
        base_amount: row.get("base_amount"),
        payout_type: row.get("payout_type"),
        is_active: row.get("is_active"),
        created_at: row.get("created_at"),
    })).into_response()
}

pub async fn delete_incentive_policy(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, policy_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    match sqlx::query("DELETE FROM payroll_incentive_policies WHERE id = $1").bind(policy_id).execute(&state.db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
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
pub struct PayslipSegmentResponse {
    #[serde(flatten)]
    pub segment: PayslipSegment,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct FullPayslipResponse {
    #[serde(flatten)]
    pub payslip: Payslip,
    pub segments: Vec<PayslipSegmentResponse>,
    pub incentives: Vec<IncentiveEvaluationResult>,
    pub salary_components: Vec<SalaryComponentWithDefinition>,
}

pub async fn calculate_batch_payroll(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
    Json(req): Json<BatchCalculatePayrollRequest>,
) -> impl IntoResponse {
    let response = crate::services::payroll_service::PayrollService::calculate_batch_payslips(
        &state.db,
        req.employee_ids,
        workspace_id,
        req.year,
        req.month,
    )
    .await;
    Json(response).into_response()
}

pub async fn get_workspace_payslips(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
    Query(query): Query<PayrollQuery>,
) -> impl IntoResponse {
    let start_date = chrono::NaiveDate::from_ymd_opt(query.year, query.month, 1).unwrap();
    let next_month = if query.month == 12 {
        chrono::NaiveDate::from_ymd_opt(query.year + 1, 1, 1).unwrap()
    } else {
        chrono::NaiveDate::from_ymd_opt(query.year, query.month + 1, 1).unwrap()
    };
    let end_date = next_month.pred_opt().unwrap();

    match crate::repositories::payroll_repository::get_payslips_by_workspace_period(
        &state.db,
        workspace_id,
        start_date,
        end_date,
    )
    .await
    {
        Ok(payslips) => Json(payslips).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn calculate_payroll(
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    crate::utils::debug::DebugContext { mock_date: _mock_date }: crate::utils::debug::DebugContext,
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
             // Fetch segments and incentives
             let segments = crate::repositories::payroll_repository::get_payslip_segments(&state.db, payslip.id)
                .await
                .unwrap_or_default();
             let incentives = crate::repositories::payroll_repository::get_payslip_incentives(&state.db, payslip.id)
                .await
                .unwrap_or_default();

             // Fetch salary components at the period start date
             let salary_components = crate::repositories::payroll_repository::get_employee_salary_components(
                 &state.db,
                 employee_id,
                 payslip.period_start,
             )
             .await
             .unwrap_or_default()
             .into_iter()
             .map(|(component, definition)| SalaryComponentWithDefinition {
                 component,
                 definition,
             })
             .collect();

             let response = FullPayslipResponse {
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
                 }).collect(),
                 incentives,
                 salary_components,
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
                 let incentives = crate::repositories::payroll_repository::get_payslip_incentives(&state.db, p.id)
                    .await
                    .unwrap_or_default();

                 // Fetch salary components at the period start date
                 let salary_components = crate::repositories::payroll_repository::get_employee_salary_components(
                     &state.db,
                     employee_id,
                     p.period_start,
                 )
                 .await
                 .unwrap_or_default()
                 .into_iter()
                 .map(|(component, definition)| SalaryComponentWithDefinition {
                     component,
                     definition,
                 })
                 .collect();

                 responses.push(FullPayslipResponse {
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
                     }).collect(),
                     incentives,
                     salary_components,
                 });
            }
            Json(responses).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
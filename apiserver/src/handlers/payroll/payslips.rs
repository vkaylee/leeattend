use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{
    BatchCalculatePayrollRequest, BatchCalculatePayrollResponse, IncentiveEvaluationResult,
    Payslip, PayslipSegment, SalaryComponentWithDefinition,
};
use crate::AppState;

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
) -> AppResult<Json<BatchCalculatePayrollResponse>> {
    let response = crate::services::payroll_service::PayrollService::calculate_batch_payslips(
        &state.db,
        req.employee_ids,
        workspace_id,
        req.year,
        req.month,
    )
    .await;
    Ok(Json(response))
}

pub async fn get_workspace_payslips(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
    Query(query): Query<PayrollQuery>,
) -> AppResult<Json<Vec<Payslip>>> {
    let start_date = chrono::NaiveDate::from_ymd_opt(query.year, query.month, 1)
        .ok_or_else(|| AppError::Validation("Invalid year/month".to_string()))?;

    let next_month = if query.month == 12 {
        chrono::NaiveDate::from_ymd_opt(query.year + 1, 1, 1)
    } else {
        chrono::NaiveDate::from_ymd_opt(query.year, query.month + 1, 1)
    };

    let end_date = next_month
        .and_then(|d| d.pred_opt())
        .ok_or_else(|| AppError::Validation("Invalid date range".to_string()))?;

    let payslips = crate::repositories::payroll_repository::get_payslips_by_workspace_period(
        &state.db,
        workspace_id,
        start_date,
        end_date,
    )
    .await?;

    Ok(Json(payslips))
}

pub async fn calculate_payroll(
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    crate::utils::debug::DebugContext {
        mock_date: _mock_date,
    }: crate::utils::debug::DebugContext,
    Query(query): Query<PayrollQuery>,
) -> AppResult<Json<FullPayslipResponse>> {
    let payslip = crate::services::payroll_service::PayrollService::calculate_monthly_payslip(
        &state.db,
        employee_id,
        workspace_id,
        query.year,
        query.month,
    )
    .await
    .map_err(|s| AppError::from((s, "Failed to calculate payroll".to_string())))?;

    // Fetch segments and incentives
    let segments =
        crate::repositories::payroll_repository::get_payslip_segments(&state.db, payslip.id)
            .await
            .unwrap_or_default();
    let incentives =
        crate::repositories::payroll_repository::get_payslip_incentives(&state.db, payslip.id)
            .await
            .unwrap_or_default();

    // Fetch salary components at the period start date
    let salary_components =
        crate::repositories::payroll_repository::get_employee_salary_components(
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
        segments: segments
            .into_iter()
            .map(|s| {
                // Determine reason logic
                let reason = if s.start_date.day() == 1 {
                    "Standard".to_string()
                } else if s.applied_base_salary > rust_decimal::Decimal::ZERO {
                    "Salary Change".to_string()
                } else {
                    "Adjustment".to_string()
                };

                PayslipSegmentResponse { segment: s, reason }
            })
            .collect(),
        incentives,
        salary_components,
    };
    Ok(Json(response))
}

pub async fn get_employee_payslips(
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
) -> AppResult<Json<Vec<FullPayslipResponse>>> {
    let payslips =
        crate::repositories::payroll_repository::get_payslips_by_employee(&state.db, employee_id)
            .await?;

    let mut responses = Vec::new();
    for p in payslips {
        let segments =
            crate::repositories::payroll_repository::get_payslip_segments(&state.db, p.id)
                .await
                .unwrap_or_default();
        let incentives =
            crate::repositories::payroll_repository::get_payslip_incentives(&state.db, p.id)
                .await
                .unwrap_or_default();

        // Fetch salary components at the period start date
        let salary_components =
            crate::repositories::payroll_repository::get_employee_salary_components(
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
            segments: segments
                .into_iter()
                .map(|s| {
                    // Determine reason logic (simple heuristic)
                    // If it's the first segment and starts on 1st: Standard
                    // Else: Change
                    let reason = if s.start_date.day() == 1 {
                        "Standard".to_string()
                    } else {
                        "Salary Change".to_string()
                    }; // MVP heuristic
                    PayslipSegmentResponse { segment: s, reason }
                })
                .collect(),
            incentives,
            salary_components,
        });
    }
    Ok(Json(responses))
}

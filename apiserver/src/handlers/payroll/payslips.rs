use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{
    BatchCalculatePayrollRequest, BatchCalculatePayrollResponse, IncentiveEvaluationResult,
    Payslip, PayslipRuleEvaluation, PayslipSegment, SalaryComponentWithDefinition,
};
use crate::utils::i18n::I18nKey;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct PayrollQuery {
    pub year: i32,
    pub month: u32,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PayslipSegmentResponse {
    #[serde(flatten)]
    pub segment: PayslipSegment,
    pub reason: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct FullPayslipResponse {
    #[serde(flatten)]
    pub payslip: Payslip,
    pub segments: Vec<PayslipSegmentResponse>,
    pub incentives: Vec<IncentiveEvaluationResult>,
    pub rule_evaluations: Vec<PayslipRuleEvaluation>,
    pub salary_components: Vec<SalaryComponentWithDefinition>,
}

/// POST /workspaces/{ws_id}/payroll/calculate-batch
#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/payroll/calculate-batch",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = BatchCalculatePayrollRequest,
    responses(
        (status = 200, description = "Calculated batch payroll", body = BatchCalculatePayrollResponse)
    )
)]
pub async fn calculate_batch_payroll(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<BatchCalculatePayrollRequest>,
) -> AppResult<Json<BatchCalculatePayrollResponse>> {
    ctx.require_permission(Permission::PayrollCalculate)?;
    let response = state
        .payroll()
        .service()
        .calculate_batch_payslips(req.employee_ids, ctx.workspace_id, req.year, req.month)
        .await;
    Ok(Json(response))
}

/// GET /workspaces/{ws_id}/payroll/payslips
#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/payroll/payslips",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("year" = i32, Query, description = "Year"),
        ("month" = u32, Query, description = "Month")
    ),
    responses(
        (status = 200, description = "Workspace payslips list", body = [Payslip])
    )
)]
pub async fn get_workspace_payslips(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<PayrollQuery>,
) -> AppResult<Json<Vec<Payslip>>> {
    ctx.require_permission(Permission::PayrollRead)?;
    let start_date = chrono::NaiveDate::from_ymd_opt(query.year, query.month, 1)
        .ok_or_else(|| AppError::validation(I18nKey::InvalidYearMonth))?;

    let next_month = if query.month == 12 {
        chrono::NaiveDate::from_ymd_opt(query.year + 1, 1, 1)
    } else {
        chrono::NaiveDate::from_ymd_opt(query.year, query.month + 1, 1)
    };

    let end_date = next_month
        .and_then(|d| d.pred_opt())
        .ok_or_else(|| AppError::validation(I18nKey::InvalidDateRange))?;

    let payslips = state
        .payroll()
        .repo()
        .get_payslips_by_workspace_period(ctx.workspace_id, start_date, end_date)
        .await?;

    Ok(Json(payslips))
}

/// POST /workspaces/{ws_id}/employees/{employee_id}/calculate-payroll
#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/employees/{employee_id}/calculate-payroll",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employee_id" = Uuid, Path, description = "Employee ID"),
        ("year" = i32, Query, description = "Year"),
        ("month" = u32, Query, description = "Month")
    ),
    responses(
        (status = 200, description = "Calculated monthly payslip details", body = FullPayslipResponse)
    )
)]
pub async fn calculate_payroll(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    crate::utils::debug::DebugContext {
        mock_date: _mock_date,
    }: crate::utils::debug::DebugContext,
    Query(query): Query<PayrollQuery>,
) -> AppResult<Json<FullPayslipResponse>> {
    let (_, employee_id) = params;
    ctx.require_permission(Permission::PayrollCalculate)?;
    let payslip = state
        .payroll()
        .service()
        .calculate_monthly_payslip(employee_id, ctx.workspace_id, query.year, query.month)
        .await?;

    // Fetch segments and incentives
    let segments = state
        .payroll()
        .repo()
        .get_payslip_segments(ctx.workspace_id, payslip.id)
        .await?;
    let incentives = state
        .payroll()
        .repo()
        .get_payslip_incentives(ctx.workspace_id, payslip.id)
        .await?;
    let rule_evaluations = state
        .payroll()
        .repo()
        .get_payslip_rule_evaluations(ctx.workspace_id, payslip.id)
        .await?;

    // Fetch salary components at the period start date
    let salary_components = state
        .payroll()
        .repo()
        .get_employee_salary_components(ctx.workspace_id, employee_id, payslip.period_start)
        .await?
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
        rule_evaluations,
        salary_components,
    };
    Ok(Json(response))
}

/// GET /workspaces/{ws_id}/employees/{employee_id}/payslips
#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/employees/{employee_id}/payslips",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employee_id" = Uuid, Path, description = "Employee ID")
    ),
    responses(
        (status = 200, description = "Employee payslips list", body = [FullPayslipResponse])
    )
)]
pub async fn get_employee_payslips(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<FullPayslipResponse>>> {
    let (_, employee_id) = params;
    ctx.require_permission(Permission::PayrollRead)?;
    let payslips = state
        .payroll()
        .repo()
        .get_payslips_by_employee(ctx.workspace_id, employee_id)
        .await?;

    let mut responses = Vec::new();
    for p in payslips {
        let segments = state
            .payroll()
            .repo()
            .get_payslip_segments(ctx.workspace_id, p.id)
            .await?;
        let incentives = state
            .payroll()
            .repo()
            .get_payslip_incentives(ctx.workspace_id, p.id)
            .await?;
        let rule_evaluations = state
            .payroll()
            .repo()
            .get_payslip_rule_evaluations(ctx.workspace_id, p.id)
            .await?;

        // Fetch salary components at the period start date
        let salary_components = state
            .payroll()
            .repo()
            .get_employee_salary_components(ctx.workspace_id, employee_id, p.period_start)
            .await?
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
            rule_evaluations,
            salary_components,
        });
    }
    Ok(Json(responses))
}

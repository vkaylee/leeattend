use crate::errors::AppResult;
use crate::models::{
    DashboardActionsSummary, DashboardComparisonBasis, DashboardDataQuality, DashboardMetric,
    DashboardMetricContext, DashboardMetricDirection, DashboardMetricFreshness,
    DashboardMetricHealth, DashboardMetricQuantity, DashboardMetricScope, DashboardMetricStatus,
    DashboardMetricTarget, DashboardMetricUnit, DashboardMetricsQuery, DashboardMetricsResponse,
    DashboardQualitySource, DashboardReportingPeriod, DashboardWorkspaceContext,
    NormalizedDashboardMetricsQuery,
};
use crate::repositories::{
    ExecutiveDashboardRequest, ExecutiveDashboardSnapshot, WorkspaceRepository,
    WorkspaceStatsRepository,
};
use crate::utils::tz;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use rust_decimal::Decimal;
use std::collections::BTreeMap;
use std::sync::Arc;
use uuid::Uuid;

/// Workspace-scoped budget targets for cost metrics.
///
/// Resolved from `workspace_config_entries` under the keys
/// `dashboard.budget.overtime_cost` / `dashboard.budget.labor_cost`.
/// `None` means the budget is not configured.
#[derive(Debug, Clone, Default)]
pub struct BudgetConfig {
    pub overtime_cost: Option<Decimal>,
    pub labor_cost: Option<Decimal>,
}

#[allow(
    clippy::implicit_hasher,
    clippy::too_many_lines,
    clippy::too_many_arguments
)]
pub async fn build_dashboard_metrics(
    workspace_repo: &Arc<dyn WorkspaceRepository>,
    stats_repo: &Arc<dyn WorkspaceStatsRepository>,
    auth_token: &crate::diesel_pool::TenantAuthToken,
    query: &DashboardMetricsQuery,
    now: DateTime<Utc>,
    #[allow(clippy::implicit_hasher)] managed_department_ids: Option<
        &std::collections::HashSet<Uuid>,
    >,
    allow_overtime_cost: bool,
    budgets: &BudgetConfig,
) -> AppResult<DashboardMetricsResponse> {
    validate_department_scope_query(query, managed_department_ids)?;
    let workspace_id = auth_token.workspace_id();
    let workspace = workspace_repo
        .get_workspace_by_id(workspace_id)
        .await?
        .ok_or_else(|| {
            crate::errors::AppError::not_found(crate::utils::i18n::I18nKey::WorkspaceNotFound)
        })?;
    let workspace_tz = tz::validate_tz(workspace.timezone())?;
    let normalized = query.normalize(now.with_timezone(&workspace_tz).date_naive())?;
    validate_department_scope(&normalized, managed_department_ids)?;

    let period = DashboardReportingPeriod::new(
        normalized.period.clone(),
        normalized.start_date,
        normalized.end_date,
        workspace_tz.name(),
        now,
    );
    let scope_department_ids = if normalized.department_ids.is_empty() {
        managed_department_ids
            .map(|ids| ids.iter().copied().collect())
            .unwrap_or_default()
    } else {
        normalized.department_ids.clone()
    };
    let scope_kind = if scope_department_ids.is_empty() {
        "workspace"
    } else {
        "department"
    };
    let mut filters = BTreeMap::new();
    filters.insert(
        "department_ids".to_string(),
        join_uuids(&normalized.department_ids),
    );
    filters.insert(
        "location_ids".to_string(),
        join_uuids(&normalized.location_ids),
    );
    filters.insert(
        "business_unit_ids".to_string(),
        join_uuids(&normalized.business_unit_ids),
    );
    filters.insert(
        "manager_ids".to_string(),
        join_uuids(&normalized.manager_ids),
    );
    filters.insert(
        "employment_types".to_string(),
        normalized.employment_types.join(","),
    );
    filters.insert(
        "employee_statuses".to_string(),
        normalized.employee_statuses.join(","),
    );
    filters.insert("shift_ids".to_string(), join_uuids(&normalized.shift_ids));
    filters.insert("top_n".to_string(), normalized.top_n.to_string());
    filters.insert(
        "risk_threshold".to_string(),
        normalized.risk_threshold.clone(),
    );
    let omitted_dimensions = vec![
        "business_unit".to_string(),
        "manager".to_string(),
        "employment_type".to_string(),
        "employee_status".to_string(),
        "shift".to_string(),
    ];
    let comparison = normalized.comparison.clone();
    let context = DashboardMetricContext::new(
        period,
        comparison.clone(),
        filters,
        DashboardMetricScope::new(
            scope_kind,
            scope_department_ids.clone(),
            normalized.location_ids.clone(),
            omitted_dimensions,
        ),
    );

    let (comparison_start, comparison_end) = comparison_period(&normalized);
    let request = ExecutiveDashboardRequest {
        current_start: normalized.start_date,
        current_end: normalized.end_date,
        comparison_start,
        comparison_end,
        department_ids: (!scope_department_ids.is_empty()).then(|| scope_department_ids.clone()),
        location_ids: (!normalized.location_ids.is_empty())
            .then(|| normalized.location_ids.clone()),
        timezone: workspace_tz.name().to_string(),
        allow_overtime_cost,
    };
    let snapshots = stats_repo
        .fetch_executive_dashboard(auth_token, &request)
        .await?;
    let freshness = DashboardMetricFreshness::new(now, Some(0));
    let basis = comparison_basis(&comparison);
    let metrics = build_metrics(
        &snapshots.current,
        &snapshots.comparison,
        &normalized,
        comparison_start,
        comparison_end,
        basis,
        freshness.clone(),
        budgets,
    );
    let data_quality_status = if metrics
        .iter()
        .any(|metric| metric.status() == &DashboardMetricStatus::DataGap)
    {
        "partial"
    } else {
        "complete"
    };
    let currency = snapshots
        .current
        .currency_code
        .as_deref()
        .filter(|value| value.len() == 3)
        .unwrap_or("USD");

    let data_quality_sources = vec![
        DashboardQualitySource::new(
            "employments",
            source_status(snapshots.current.headcount > 0),
            None,
            freshness.clone(),
            Vec::new(),
        ),
        DashboardQualitySource::new(
            "attendance_daily_summaries",
            source_status(snapshots.current.eligible_employee_days > 0),
            None,
            freshness.clone(),
            Vec::new(),
        ),
        DashboardQualitySource::new(
            "scheduled_attendance",
            "data_gap",
            None,
            freshness.clone(),
            vec!["scheduled_attendance_source_unavailable".to_string()],
        ),
        DashboardQualitySource::new(
            "payroll",
            if allow_overtime_cost && snapshots.current.cost_source_available {
                "complete"
            } else {
                "data_gap"
            },
            None,
            freshness,
            if allow_overtime_cost && !snapshots.current.cost_source_available {
                vec!["no_paid_payslips_in_period".to_string()]
            } else {
                Vec::new()
            },
        ),
    ];

    Ok(DashboardMetricsResponse::new(
        "executive-dashboard.v1",
        DashboardWorkspaceContext::new(workspace_tz.name(), currency),
        context,
        metrics,
        Vec::new(),
        DashboardActionsSummary::new(
            snapshots.current.pending_hr_actions.max(0).cast_unsigned(),
            0,
            Vec::new(),
        ),
        DashboardDataQuality::new(data_quality_status, data_quality_sources),
        now,
    ))
}

fn source_status(has_data: bool) -> &'static str {
    if has_data {
        "complete"
    } else {
        "data_gap"
    }
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn build_metrics(
    current: &ExecutiveDashboardSnapshot,
    comparison: &ExecutiveDashboardSnapshot,
    query: &NormalizedDashboardMetricsQuery,
    comparison_start: NaiveDate,
    comparison_end: NaiveDate,
    basis: DashboardComparisonBasis,
    freshness: DashboardMetricFreshness,
    budgets: &BudgetConfig,
) -> Vec<DashboardMetric> {
    let attendance_target = DashboardMetricTarget::new(
        Decimal::from(95),
        crate::models::DashboardTargetOperator::Gte,
    );
    let absence_target = DashboardMetricTarget::new(
        Decimal::from(5),
        crate::models::DashboardTargetOperator::Lte,
    );
    let turnover_target = DashboardMetricTarget::new(
        Decimal::from(15),
        crate::models::DashboardTargetOperator::Lte,
    );
    let overtime_cost_target = budgets.overtime_cost.map(|value| {
        DashboardMetricTarget::new(value, crate::models::DashboardTargetOperator::Lte)
    });
    let labor_cost_target = budgets.labor_cost.map(|value| {
        DashboardMetricTarget::new(value, crate::models::DashboardTargetOperator::Lte)
    });

    let metrics = vec![
        count_metric(
            "headcount",
            current.headcount,
            Some(comparison.headcount),
            DashboardMetricDirection::Neutral,
            "headcount.v1",
            None,
            freshness.clone(),
            query,
            comparison_start,
            comparison_end,
            basis.clone(),
        ),
        rate_metric(
            "attendance_rate",
            current.present_employee_days,
            current.eligible_employee_days,
            comparison,
            |snapshot| {
                (
                    snapshot.present_employee_days,
                    snapshot.eligible_employee_days,
                )
            },
            DashboardMetricDirection::HigherIsBetter,
            "attendance-rate.v1",
            Some(attendance_target),
            freshness.clone(),
            query,
            comparison_start,
            comparison_end,
            basis.clone(),
        ),
        rate_metric(
            "absence_rate",
            current.absent_employee_days,
            current.eligible_employee_days,
            comparison,
            |snapshot| {
                (
                    snapshot.absent_employee_days,
                    snapshot.eligible_employee_days,
                )
            },
            DashboardMetricDirection::LowerIsBetter,
            "absence-rate.v1",
            Some(absence_target),
            freshness.clone(),
            query,
            comparison_start,
            comparison_end,
            basis.clone(),
        ),
        data_gap_metric(
            "late_rate",
            DashboardMetricDirection::LowerIsBetter,
            "late-rate.v1",
            freshness.clone(),
            "scheduled_attendance_source_unavailable",
        ),
        rate_metric(
            "turnover_rate",
            current.termination_count,
            i64::midpoint(current.headcount_start, current.headcount_end),
            comparison,
            |snapshot| {
                (
                    snapshot.termination_count,
                    i64::midpoint(snapshot.headcount_start, snapshot.headcount_end),
                )
            },
            DashboardMetricDirection::LowerIsBetter,
            "turnover-rate.v1",
            Some(turnover_target),
            freshness.clone(),
            query,
            comparison_start,
            comparison_end,
            basis.clone(),
        ),
        // ── Overtime hours ─────────────────────────────────────────────
        hours_metric(
            "system_ot_hours",
            current.system_detected_ot_minutes / 60,
            Some(comparison.system_detected_ot_minutes / 60),
            DashboardMetricDirection::LowerIsBetter,
            "system-ot-hours.v1",
            freshness.clone(),
            query,
            comparison_start,
            comparison_end,
            basis.clone(),
        ),
        hours_metric(
            "approved_ot_hours",
            current.approved_overtime_minutes / 60,
            Some(comparison.approved_overtime_minutes / 60),
            DashboardMetricDirection::LowerIsBetter,
            "approved-ot-hours.v1",
            freshness.clone(),
            query,
            comparison_start,
            comparison_end,
            basis.clone(),
        ),
        hours_metric(
            "unapproved_ot_hours",
            current.unapproved_ot_minutes / 60,
            Some(comparison.unapproved_ot_minutes / 60),
            DashboardMetricDirection::LowerIsBetter,
            "unapproved-ot-hours.v1",
            freshness.clone(),
            query,
            comparison_start,
            comparison_end,
            basis.clone(),
        ),
        ot_approval_rate_metric(
            current,
            comparison,
            freshness.clone(),
            query,
            comparison_start,
            comparison_end,
            basis.clone(),
        ),
        overtime_metric(
            current,
            comparison,
            query,
            comparison_start,
            comparison_end,
            basis.clone(),
            freshness.clone(),
            overtime_cost_target,
        ),
        // ── Labor cost (permission-gated, data_gap when no paid payslips) ──
        ot_cost_per_employee_metric(
            current,
            comparison,
            freshness.clone(),
            query,
            comparison_start,
            comparison_end,
            basis.clone(),
        ),
        currency_metric(
            "labor_cost",
            current.labor_cost,
            comparison.labor_cost,
            DashboardMetricDirection::LowerIsBetter,
            "labor-cost.v1",
            freshness.clone(),
            query,
            comparison_start,
            comparison_end,
            basis.clone(),
            labor_cost_target,
            "no_paid_payslips",
        ),
        currency_metric(
            "payroll_actual",
            current.payroll_actual,
            comparison.payroll_actual,
            DashboardMetricDirection::Neutral,
            "payroll-actual.v1",
            freshness.clone(),
            query,
            comparison_start,
            comparison_end,
            basis.clone(),
            None,
            "no_paid_payslips",
        ),
        data_gap_metric(
            "payroll_forecast",
            DashboardMetricDirection::Neutral,
            "payroll-forecast.v1",
            freshness.clone(),
            "forecast_source_unavailable",
        ),
        budget_variance_metric(
            current,
            comparison,
            budgets,
            freshness.clone(),
            query,
            comparison_start,
            comparison_end,
            basis.clone(),
        ),
        count_metric(
            "pending_hr_actions",
            current.pending_hr_actions,
            Some(comparison.pending_hr_actions),
            DashboardMetricDirection::LowerIsBetter,
            "pending-hr-actions.v1",
            None,
            freshness,
            query,
            comparison_start,
            comparison_end,
            basis,
        ),
    ];
    metrics
}

#[allow(clippy::too_many_arguments)]
fn count_metric(
    key: &'static str,
    value: i64,
    comparison_value: Option<i64>,
    direction: DashboardMetricDirection,
    definition: &'static str,
    target: Option<DashboardMetricTarget>,
    freshness: DashboardMetricFreshness,
    query: &NormalizedDashboardMetricsQuery,
    comparison_start: NaiveDate,
    comparison_end: NaiveDate,
    basis: DashboardComparisonBasis,
) -> DashboardMetric {
    MetricBuilder {
        key,
        value: Some(Decimal::from(value.max(0))),
        unit: DashboardMetricUnit::Count,
        numerator: None,
        denominator: None,
        status: DashboardMetricStatus::Available,
        health: evaluate_health(Decimal::from(value.max(0)), target.as_ref()),
        null_reason: None,
        definition,
        direction,
        freshness,
        quality_flags: Vec::new(),
        comparison: comparison_value.map(|previous| {
            comparison_for(
                basis,
                comparison_start,
                comparison_end,
                Some(Decimal::from(previous.max(0))),
                Some(Decimal::from(value - previous)),
                query,
            )
        }),
        target,
    }
    .build()
}

#[allow(clippy::too_many_arguments)]
fn rate_metric(
    key: &str,
    numerator: i64,
    denominator: i64,
    comparison: &ExecutiveDashboardSnapshot,
    comparison_values: impl Fn(&ExecutiveDashboardSnapshot) -> (i64, i64),
    direction: DashboardMetricDirection,
    definition: &str,
    target: Option<DashboardMetricTarget>,
    freshness: DashboardMetricFreshness,
    query: &NormalizedDashboardMetricsQuery,
    comparison_start: NaiveDate,
    comparison_end: NaiveDate,
    basis: DashboardComparisonBasis,
) -> DashboardMetric {
    if denominator <= 0 {
        return DashboardMetric::new(
            key,
            None,
            DashboardMetricUnit::Percent,
            None,
            None,
            DashboardMetricStatus::NotApplicable,
            DashboardMetricHealth::NotApplicable,
            Some("zero_denominator".to_string()),
            definition,
            direction,
            freshness,
            vec!["zero_denominator".to_string()],
            None,
            target,
        );
    }
    let value = percent(numerator, denominator);
    let (comparison_numerator, comparison_denominator) = comparison_values(comparison);
    let comparison = if matches!(basis, DashboardComparisonBasis::None) {
        None
    } else if comparison_denominator > 0 {
        let previous = percent(comparison_numerator, comparison_denominator);
        Some(comparison_for(
            basis,
            comparison_start,
            comparison_end,
            Some(previous),
            Some(value - previous),
            query,
        ))
    } else {
        None
    };
    DashboardMetric::new(
        key,
        Some(value),
        DashboardMetricUnit::Percent,
        Some(DashboardMetricQuantity::new(
            Decimal::from(numerator.max(0)),
            "employee_days",
        )),
        Some(DashboardMetricQuantity::new(
            Decimal::from(denominator),
            "employee_days",
        )),
        DashboardMetricStatus::Available,
        evaluate_health(value, target.as_ref()),
        None,
        definition,
        direction,
        freshness,
        Vec::new(),
        comparison,
        target,
    )
}

fn data_gap_metric(
    key: &'static str,
    direction: DashboardMetricDirection,
    definition: &'static str,
    freshness: DashboardMetricFreshness,
    reason: &str,
) -> DashboardMetric {
    DashboardMetric::new(
        key,
        None,
        DashboardMetricUnit::Percent,
        None,
        None,
        DashboardMetricStatus::DataGap,
        DashboardMetricHealth::DataGap,
        Some(reason.to_string()),
        definition,
        direction,
        freshness,
        vec![reason.to_string()],
        None,
        None,
    )
}

struct MetricBuilder {
    key: &'static str,
    value: Option<Decimal>,
    unit: DashboardMetricUnit,
    numerator: Option<DashboardMetricQuantity>,
    denominator: Option<DashboardMetricQuantity>,
    status: DashboardMetricStatus,
    health: DashboardMetricHealth,
    null_reason: Option<String>,
    definition: &'static str,
    direction: DashboardMetricDirection,
    freshness: DashboardMetricFreshness,
    quality_flags: Vec<String>,
    comparison: Option<crate::models::DashboardMetricComparison>,
    target: Option<DashboardMetricTarget>,
}

impl MetricBuilder {
    fn build(self) -> DashboardMetric {
        DashboardMetric::new(
            self.key,
            self.value,
            self.unit,
            self.numerator,
            self.denominator,
            self.status,
            self.health,
            self.null_reason,
            self.definition,
            self.direction,
            self.freshness,
            self.quality_flags,
            self.comparison,
            self.target,
        )
    }
}

fn comparison_for(
    basis: DashboardComparisonBasis,
    period_start: NaiveDate,
    period_end: NaiveDate,
    previous: Option<Decimal>,
    delta: Option<Decimal>,
    _query: &NormalizedDashboardMetricsQuery,
) -> crate::models::DashboardMetricComparison {
    let delta_percent = match (previous, delta) {
        (Some(previous), Some(delta)) if previous != Decimal::ZERO => {
            Some(delta / previous * Decimal::from(100))
        }
        _ => None,
    };
    let delta_percentage_points = delta;
    crate::models::DashboardMetricComparison::new(
        basis,
        period_start,
        period_end,
        previous,
        delta,
        delta_percent,
        delta_percentage_points,
        DashboardMetricStatus::Available,
    )
}

fn percent(numerator: i64, denominator: i64) -> Decimal {
    Decimal::from(numerator.max(0)) / Decimal::from(denominator) * Decimal::from(100)
}

#[allow(clippy::too_many_arguments)]
fn overtime_metric(
    current: &ExecutiveDashboardSnapshot,
    comparison: &ExecutiveDashboardSnapshot,
    query: &NormalizedDashboardMetricsQuery,
    comparison_start: NaiveDate,
    comparison_end: NaiveDate,
    basis: DashboardComparisonBasis,
    freshness: DashboardMetricFreshness,
    target: Option<DashboardMetricTarget>,
) -> DashboardMetric {
    // Grid compatibility: the slot resolves `overtime_cost` first, else falls
    // back to `overtime_hours`. Emit `overtime_cost` only when the caller may
    // see it AND a paid/finalized payslip exists in the period; when the caller
    // lacks `PayrollSensitiveRead` the repo leaves the cost as `None` and we
    // fall back to `overtime_hours`.
    let Some(cost) = current.overtime_cost else {
        return hours_metric(
            "overtime_hours",
            current.approved_overtime_minutes / 60,
            Some(comparison.approved_overtime_minutes / 60),
            DashboardMetricDirection::LowerIsBetter,
            "overtime-hours.v1",
            freshness,
            query,
            comparison_start,
            comparison_end,
            basis,
        );
    };
    // Permission granted but no paid/finalized payslips: honest data_gap,
    // not a fabricated zero.
    if !current.cost_source_available {
        return currency_metric(
            "overtime_cost",
            None,
            None,
            DashboardMetricDirection::LowerIsBetter,
            "overtime-cost.v1",
            freshness,
            query,
            comparison_start,
            comparison_end,
            basis,
            target,
            "no_paid_payslips",
        );
    }
    let comparison_value = comparison.overtime_cost;
    let health = evaluate_health(cost, target.as_ref());
    DashboardMetric::new(
        "overtime_cost",
        Some(cost),
        DashboardMetricUnit::Currency,
        None,
        None,
        DashboardMetricStatus::Available,
        health,
        None,
        "overtime-cost.v1",
        DashboardMetricDirection::LowerIsBetter,
        freshness,
        Vec::new(),
        comparison_value.map(|previous| {
            comparison_for(
                basis.clone(),
                comparison_start,
                comparison_end,
                Some(previous),
                Some(cost - previous),
                query,
            )
        }),
        target,
    )
}

/// Approval rate = approved / system-detected, percent. Zero denominator →
/// `not_applicable` with `zero_denominator`.
#[allow(clippy::too_many_arguments)]
fn ot_approval_rate_metric(
    current: &ExecutiveDashboardSnapshot,
    comparison: &ExecutiveDashboardSnapshot,
    freshness: DashboardMetricFreshness,
    query: &NormalizedDashboardMetricsQuery,
    comparison_start: NaiveDate,
    comparison_end: NaiveDate,
    basis: DashboardComparisonBasis,
) -> DashboardMetric {
    if current.system_detected_ot_minutes <= 0 {
        return DashboardMetric::new(
            "ot_approval_rate",
            None,
            DashboardMetricUnit::Percent,
            None,
            None,
            DashboardMetricStatus::NotApplicable,
            DashboardMetricHealth::NotApplicable,
            Some("zero_denominator".to_string()),
            "ot-approval-rate.v1",
            DashboardMetricDirection::HigherIsBetter,
            freshness,
            vec!["zero_denominator".to_string()],
            None,
            None,
        );
    }
    let value = percent(
        current.approved_overtime_minutes,
        current.system_detected_ot_minutes,
    );
    let comparison_value = if comparison.system_detected_ot_minutes > 0 {
        Some(percent(
            comparison.approved_overtime_minutes,
            comparison.system_detected_ot_minutes,
        ))
    } else {
        None
    };
    DashboardMetric::new(
        "ot_approval_rate",
        Some(value),
        DashboardMetricUnit::Percent,
        Some(DashboardMetricQuantity::new(
            Decimal::from(current.approved_overtime_minutes.max(0)),
            "minutes",
        )),
        Some(DashboardMetricQuantity::new(
            Decimal::from(current.system_detected_ot_minutes),
            "minutes",
        )),
        DashboardMetricStatus::Available,
        DashboardMetricHealth::Ok,
        None,
        "ot-approval-rate.v1",
        DashboardMetricDirection::HigherIsBetter,
        freshness,
        Vec::new(),
        comparison_value.map(|previous| {
            comparison_for(
                basis,
                comparison_start,
                comparison_end,
                Some(previous),
                Some(value - previous),
                query,
            )
        }),
        None,
    )
}

/// OT cost / headcount. No cost source → data_gap; headcount 0 → not_applicable.
#[allow(clippy::too_many_arguments)]
fn ot_cost_per_employee_metric(
    current: &ExecutiveDashboardSnapshot,
    comparison: &ExecutiveDashboardSnapshot,
    freshness: DashboardMetricFreshness,
    query: &NormalizedDashboardMetricsQuery,
    comparison_start: NaiveDate,
    comparison_end: NaiveDate,
    basis: DashboardComparisonBasis,
) -> DashboardMetric {
    if !current.cost_source_available {
        return currency_metric(
            "ot_cost_per_employee",
            None,
            None,
            DashboardMetricDirection::LowerIsBetter,
            "ot-cost-per-employee.v1",
            freshness,
            query,
            comparison_start,
            comparison_end,
            basis,
            None,
            "no_paid_payslips",
        );
    }
    if current.headcount <= 0 {
        return DashboardMetric::new(
            "ot_cost_per_employee",
            None,
            DashboardMetricUnit::Currency,
            None,
            None,
            DashboardMetricStatus::NotApplicable,
            DashboardMetricHealth::NotApplicable,
            Some("zero_denominator".to_string()),
            "ot-cost-per-employee.v1",
            DashboardMetricDirection::LowerIsBetter,
            freshness,
            vec!["zero_denominator".to_string()],
            None,
            None,
        );
    }
    let value = current
        .overtime_cost
        .map(|cost| cost / Decimal::from(current.headcount));
    let comparison_value = if comparison.headcount > 0 {
        comparison
            .overtime_cost
            .map(|cost| cost / Decimal::from(comparison.headcount))
    } else {
        None
    };
    currency_metric(
        "ot_cost_per_employee",
        value,
        comparison_value,
        DashboardMetricDirection::LowerIsBetter,
        "ot-cost-per-employee.v1",
        freshness,
        query,
        comparison_start,
        comparison_end,
        basis,
        None,
        "no_paid_payslips",
    )
}

/// Budget variance = labor_cost − budget (currency). Missing budget →
/// data_gap; over-budget (variance > 0) → critical via a `0 Gte` target.
#[allow(clippy::too_many_arguments)]
fn budget_variance_metric(
    current: &ExecutiveDashboardSnapshot,
    comparison: &ExecutiveDashboardSnapshot,
    budgets: &BudgetConfig,
    freshness: DashboardMetricFreshness,
    query: &NormalizedDashboardMetricsQuery,
    comparison_start: NaiveDate,
    comparison_end: NaiveDate,
    basis: DashboardComparisonBasis,
) -> DashboardMetric {
    let Some(budget) = budgets.labor_cost else {
        return currency_metric(
            "budget_variance",
            None,
            None,
            DashboardMetricDirection::LowerIsBetter,
            "budget-variance.v1",
            freshness,
            query,
            comparison_start,
            comparison_end,
            basis,
            None,
            "budget_config_missing",
        );
    };
    if !current.cost_source_available {
        return currency_metric(
            "budget_variance",
            None,
            None,
            DashboardMetricDirection::LowerIsBetter,
            "budget-variance.v1",
            freshness,
            query,
            comparison_start,
            comparison_end,
            basis,
            None,
            "no_paid_payslips",
        );
    }
    let Some(labor_cost) = current.labor_cost else {
        return currency_metric(
            "budget_variance",
            None,
            None,
            DashboardMetricDirection::LowerIsBetter,
            "budget-variance.v1",
            freshness,
            query,
            comparison_start,
            comparison_end,
            basis,
            None,
            "no_paid_payslips",
        );
    };
    let value = labor_cost - budget;
    let comparison_value = comparison.labor_cost.map(|previous| previous - budget);
    let health = evaluate_health(
        value,
        Some(&DashboardMetricTarget::new(
            Decimal::ZERO,
            crate::models::DashboardTargetOperator::Lte,
        )),
    );
    DashboardMetric::new(
        "budget_variance",
        Some(value),
        DashboardMetricUnit::Currency,
        None,
        None,
        DashboardMetricStatus::Available,
        health,
        None,
        "budget-variance.v1",
        DashboardMetricDirection::LowerIsBetter,
        freshness,
        Vec::new(),
        comparison_value.map(|previous| {
            comparison_for(
                basis,
                comparison_start,
                comparison_end,
                Some(previous),
                Some(value - previous),
                query,
            )
        }),
        Some(DashboardMetricTarget::new(
            Decimal::ZERO,
            crate::models::DashboardTargetOperator::Lte,
        )),
    )
}

/// Currency metric that degrades to `data_gap` (with `reason`) when the source
/// is unavailable (no paid/finalized payslips) and to `restricted` when the
/// caller lacks `PayrollSensitiveRead`.
#[allow(clippy::too_many_arguments)]
fn currency_metric(
    key: &'static str,
    value: Option<Decimal>,
    comparison_value: Option<Decimal>,
    direction: DashboardMetricDirection,
    definition: &'static str,
    freshness: DashboardMetricFreshness,
    query: &NormalizedDashboardMetricsQuery,
    comparison_start: NaiveDate,
    comparison_end: NaiveDate,
    basis: DashboardComparisonBasis,
    target: Option<DashboardMetricTarget>,
    reason: &str,
) -> DashboardMetric {
    let (status, health, null_reason) = match value {
        Some(_) => (
            DashboardMetricStatus::Available,
            evaluate_health(value.unwrap_or_default(), target.as_ref()),
            None,
        ),
        None => (
            DashboardMetricStatus::DataGap,
            DashboardMetricHealth::DataGap,
            Some(reason.to_string()),
        ),
    };
    MetricBuilder {
        key,
        value,
        unit: DashboardMetricUnit::Currency,
        numerator: None,
        denominator: None,
        status,
        health,
        null_reason: null_reason.clone(),
        definition,
        direction,
        freshness,
        quality_flags: if null_reason.is_some() {
            vec![reason.to_string()]
        } else {
            Vec::new()
        },
        comparison: comparison_value.map(|previous| {
            comparison_for(
                basis,
                comparison_start,
                comparison_end,
                Some(previous),
                value.map(|current| current - previous),
                query,
            )
        }),
        target,
    }
    .build()
}

#[allow(clippy::too_many_arguments)]
fn hours_metric(
    key: &'static str,
    value: i64,
    comparison_value: Option<i64>,
    direction: DashboardMetricDirection,
    definition: &'static str,
    freshness: DashboardMetricFreshness,
    query: &NormalizedDashboardMetricsQuery,
    comparison_start: NaiveDate,
    comparison_end: NaiveDate,
    basis: DashboardComparisonBasis,
) -> DashboardMetric {
    MetricBuilder {
        key,
        value: Some(Decimal::from(value.max(0))),
        unit: DashboardMetricUnit::Hours,
        numerator: None,
        denominator: None,
        status: DashboardMetricStatus::Available,
        health: DashboardMetricHealth::Ok,
        null_reason: None,
        definition,
        direction,
        freshness,
        quality_flags: Vec::new(),
        comparison: comparison_value.map(|previous| {
            comparison_for(
                basis,
                comparison_start,
                comparison_end,
                Some(Decimal::from(previous.max(0))),
                Some(Decimal::from(value - previous)),
                query,
            )
        }),
        target: None,
    }
    .build()
}

fn evaluate_health(
    value: Decimal,
    target: Option<&DashboardMetricTarget>,
) -> DashboardMetricHealth {
    let Some(target) = target else {
        return DashboardMetricHealth::Ok;
    };
    let threshold = target.value();
    match target.operator() {
        crate::models::DashboardTargetOperator::Gte => {
            if value >= threshold {
                DashboardMetricHealth::Ok
            } else if value >= threshold * Decimal::new(95, 2) {
                DashboardMetricHealth::Warning
            } else {
                DashboardMetricHealth::Critical
            }
        }
        crate::models::DashboardTargetOperator::Gt => {
            if value > threshold {
                DashboardMetricHealth::Ok
            } else {
                DashboardMetricHealth::Warning
            }
        }
        crate::models::DashboardTargetOperator::Lte => {
            if value <= threshold {
                DashboardMetricHealth::Ok
            } else if value <= threshold * Decimal::new(105, 2) {
                DashboardMetricHealth::Warning
            } else {
                DashboardMetricHealth::Critical
            }
        }
        crate::models::DashboardTargetOperator::Lt => {
            if value < threshold {
                DashboardMetricHealth::Ok
            } else {
                DashboardMetricHealth::Warning
            }
        }
        crate::models::DashboardTargetOperator::Eq => {
            if value == threshold {
                DashboardMetricHealth::Ok
            } else {
                DashboardMetricHealth::Warning
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn comparison_period(query: &NormalizedDashboardMetricsQuery) -> (NaiveDate, NaiveDate) {
    let days = query
        .end_date
        .signed_duration_since(query.start_date)
        .num_days();
    match query.comparison.as_str() {
        "same_period_last_year" => (
            query.start_date - Duration::days(365),
            query.end_date - Duration::days(365),
        ),
        "previous_period" => {
            let end = query.start_date - Duration::days(1);
            (end - Duration::days(days), end)
        }
        _ => (query.start_date, query.end_date),
    }
}

fn comparison_basis(value: &str) -> DashboardComparisonBasis {
    match value {
        "previous_period" => DashboardComparisonBasis::PreviousPeriod,
        "same_period_last_year" => DashboardComparisonBasis::SamePeriodLastYear,
        "target" => DashboardComparisonBasis::Target,
        "budget" => DashboardComparisonBasis::Budget,
        "benchmark" => DashboardComparisonBasis::Benchmark,
        _ => DashboardComparisonBasis::None,
    }
}

fn validate_department_scope_query(
    query: &DashboardMetricsQuery,
    managed_department_ids: Option<&std::collections::HashSet<Uuid>>,
) -> AppResult<()> {
    let Some(allowed) = managed_department_ids else {
        return Ok(());
    };
    let Some(department_ids) = query.department_ids.as_ref() else {
        return Ok(());
    };
    let mut requested = department_ids
        .iter()
        .flat_map(|value| value.split(','))
        .filter_map(|value| Uuid::parse_str(value.trim()).ok());
    if requested.any(|id| !allowed.contains(&id)) {
        return Err(crate::errors::AppError::forbidden(
            crate::utils::i18n::I18nKey::Forbidden,
        ));
    }
    Ok(())
}

fn validate_department_scope(
    query: &NormalizedDashboardMetricsQuery,
    managed_department_ids: Option<&std::collections::HashSet<Uuid>>,
) -> AppResult<()> {
    if let Some(allowed) = managed_department_ids {
        if query.department_ids.iter().any(|id| !allowed.contains(id)) {
            return Err(crate::errors::AppError::forbidden(
                crate::utils::i18n::I18nKey::Forbidden,
            ));
        }
    }
    Ok(())
}

fn join_uuids(values: &[Uuid]) -> String {
    values
        .iter()
        .map(Uuid::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn freshness() -> DashboardMetricFreshness {
        DashboardMetricFreshness::new(Utc.with_ymd_and_hms(2026, 8, 13, 9, 0, 0).unwrap(), Some(0))
    }

    fn snapshot() -> ExecutiveDashboardSnapshot {
        ExecutiveDashboardSnapshot {
            headcount: 100,
            eligible_employee_days: 2000,
            present_employee_days: 1900,
            absent_employee_days: 100,
            leave_employee_days: 50,
            termination_count: 4,
            headcount_start: 96,
            headcount_end: 100,
            approved_overtime_minutes: 480,
            system_detected_ot_minutes: 600,
            unapproved_ot_minutes: 120,
            overtime_approval_rate: Some(Decimal::new(80, 2)),
            pending_hr_actions: 7,
            currency_code: Some("USD".to_string()),
            overtime_cost: None,
            labor_cost: None,
            payroll_actual: None,
            cost_source_available: false,
        }
    }

    fn query() -> NormalizedDashboardMetricsQuery {
        NormalizedDashboardMetricsQuery {
            period: "this_month".to_string(),
            comparison: "previous_period".to_string(),
            start_date: NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 8, 13).unwrap(),
            department_ids: Vec::new(),
            location_ids: Vec::new(),
            business_unit_ids: Vec::new(),
            manager_ids: Vec::new(),
            employment_types: Vec::new(),
            employee_statuses: Vec::new(),
            shift_ids: Vec::new(),
            top_n: 10,
            risk_threshold: "watch".to_string(),
        }
    }

    fn build(
        current: &ExecutiveDashboardSnapshot,
        comparison: &ExecutiveDashboardSnapshot,
    ) -> Vec<serde_json::Value> {
        build_with_budget(current, comparison, &BudgetConfig::default())
    }

    fn build_with_budget(
        current: &ExecutiveDashboardSnapshot,
        comparison: &ExecutiveDashboardSnapshot,
        budgets: &BudgetConfig,
    ) -> Vec<serde_json::Value> {
        let q = query();
        build_metrics(
            current,
            comparison,
            &q,
            q.start_date,
            q.end_date,
            DashboardComparisonBasis::PreviousPeriod,
            freshness(),
            budgets,
        )
        .into_iter()
        .map(|metric| serde_json::to_value(metric).unwrap())
        .collect()
    }

    fn metric<'a>(metrics: &'a [serde_json::Value], key: &str) -> &'a serde_json::Value {
        metrics
            .iter()
            .find(|json| json["key"] == key)
            .unwrap_or_else(|| panic!("metric {key} missing"))
    }

    #[test]
    fn emits_metrics_in_fixed_order() {
        let current = snapshot();
        let metrics = build(&current, &current);
        let keys = metrics
            .iter()
            .map(|json| json["key"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            keys,
            vec![
                "headcount",
                "attendance_rate",
                "absence_rate",
                "late_rate",
                "turnover_rate",
                "system_ot_hours",
                "approved_ot_hours",
                "unapproved_ot_hours",
                "ot_approval_rate",
                "overtime_hours",
                "ot_cost_per_employee",
                "labor_cost",
                "payroll_actual",
                "payroll_forecast",
                "budget_variance",
                "pending_hr_actions"
            ]
        );
    }

    #[test]
    fn computes_normalized_attendance_rate() {
        let current = snapshot();
        let metrics = build(&current, &current);
        let attendance = metric(&metrics, "attendance_rate");
        // 1900 / 2000 = 95%
        assert_eq!(attendance["value"], "95.00");
        assert_eq!(attendance["unit"], "percent");
        assert!(attendance["numerator"].is_object());
        assert!(attendance["denominator"].is_object());
    }

    #[test]
    fn zero_denominator_is_not_applicable() {
        let mut current = snapshot();
        current.eligible_employee_days = 0;
        let metrics = build(&current, &current);
        let attendance = metric(&metrics, "attendance_rate");
        assert_eq!(attendance["status"], "not_applicable");
        assert!(attendance["value"].is_null());
        assert_eq!(attendance["null_reason"], "zero_denominator");
    }

    #[test]
    fn late_rate_is_data_gap_without_source() {
        let current = snapshot();
        let metrics = build(&current, &current);
        let late = metric(&metrics, "late_rate");
        assert_eq!(late["status"], "data_gap");
        assert_eq!(late["health"], "data_gap");
        assert!(late["value"].is_null());
    }

    #[test]
    fn overtime_hours_fallback_when_cost_unavailable() {
        let current = snapshot();
        let metrics = build(&current, &current);
        let overtime = metric(&metrics, "overtime_hours");
        assert_eq!(overtime["unit"], "hours");
        // 480 minutes / 60 = 8 hours
        assert_eq!(overtime["value"], "8");
        assert!(metrics.iter().all(|json| json["key"] != "overtime_cost"));
    }

    #[test]
    fn overtime_cost_data_gap_when_no_paid_payslips() {
        let mut current = snapshot();
        // Permission present (cost is Some) but no paid/finalized payslip.
        current.overtime_cost = Some(Decimal::ZERO);
        current.cost_source_available = false;
        let metrics = build(&current, &current);
        let overtime = metric(&metrics, "overtime_cost");
        assert_eq!(overtime["status"], "data_gap");
        assert_eq!(overtime["null_reason"], "no_paid_payslips");
        assert!(overtime["value"].is_null());
    }

    #[test]
    fn overtime_cost_emitted_when_authorized() {
        let mut current = snapshot();
        current.overtime_cost = Some(Decimal::new(1_250, 2));
        current.cost_source_available = true;
        current.labor_cost = Some(Decimal::new(5_000, 2));
        current.payroll_actual = Some(Decimal::new(4_000, 2));
        let mut comparison = snapshot();
        comparison.overtime_cost = Some(Decimal::new(1_000, 2));
        comparison.cost_source_available = true;
        comparison.labor_cost = Some(Decimal::new(4_000, 2));
        comparison.payroll_actual = Some(Decimal::new(3_000, 2));
        let metrics = build(&current, &comparison);
        let overtime = metric(&metrics, "overtime_cost");
        assert_eq!(overtime["unit"], "currency");
        assert_eq!(overtime["value"], "12.50");
        assert!(overtime["comparison"].is_object());
        assert!(metrics.iter().all(|json| json["key"] != "overtime_hours"));
        // Cost metrics surfaced.
        let labor = metric(&metrics, "labor_cost");
        assert_eq!(labor["status"], "available");
        assert_eq!(labor["value"], "50.00");
        let payroll = metric(&metrics, "payroll_actual");
        assert_eq!(payroll["value"], "40.00");
        let per_employee = metric(&metrics, "ot_cost_per_employee");
        assert_eq!(per_employee["value"], "0.1250");
    }

    #[test]
    fn cost_metrics_restricted_when_unauthorized() {
        let mut current = snapshot();
        // Unauthorized: repo leaves cost None and cost_source_available false.
        current.cost_source_available = false;
        let metrics = build(&current, &current);
        let labor = metric(&metrics, "labor_cost");
        assert_eq!(labor["status"], "data_gap");
        assert_eq!(labor["null_reason"], "no_paid_payslips");
    }

    #[test]
    fn approval_rate_uses_system_detected_denominator() {
        let current = snapshot(); // 480 approved / 600 detected = 80%
        let metrics = build(&current, &current);
        let rate = metric(&metrics, "ot_approval_rate");
        assert_eq!(rate["unit"], "percent");
        assert_eq!(rate["value"], "80.00");
        assert_eq!(rate["status"], "available");
    }

    #[test]
    fn approval_rate_zero_denominator_is_not_applicable() {
        let mut current = snapshot();
        current.system_detected_ot_minutes = 0;
        current.approved_overtime_minutes = 0;
        let metrics = build(&current, &current);
        let rate = metric(&metrics, "ot_approval_rate");
        assert_eq!(rate["status"], "not_applicable");
        assert_eq!(rate["null_reason"], "zero_denominator");
        assert!(rate["value"].is_null());
    }

    #[test]
    fn budget_variance_uses_budget_config_and_flags_over_budget() {
        let mut current = snapshot();
        current.cost_source_available = true;
        current.labor_cost = Some(Decimal::new(12_000, 2)); // 120.00
        let comparison = snapshot();
        let budgets = BudgetConfig {
            labor_cost: Some(Decimal::new(10_000, 2)), // 100.00 budget
            ..BudgetConfig::default()
        };
        let metrics = build_with_budget(&current, &comparison, &budgets);
        let variance = metric(&metrics, "budget_variance");
        assert_eq!(variance["status"], "available");
        assert_eq!(variance["value"], "20.00"); // over budget
        assert_eq!(variance["health"], "critical");
        assert!(variance["target"].is_object());
    }

    #[test]
    fn budget_variance_missing_budget_is_data_gap() {
        let current = snapshot();
        let metrics = build(&current, &current);
        let variance = metric(&metrics, "budget_variance");
        assert_eq!(variance["status"], "data_gap");
        assert_eq!(variance["null_reason"], "budget_config_missing");
        assert!(variance["value"].is_null());
    }

    #[test]
    fn payroll_forecast_is_always_data_gap() {
        let current = snapshot();
        let metrics = build(&current, &current);
        let forecast = metric(&metrics, "payroll_forecast");
        assert_eq!(forecast["status"], "data_gap");
        assert_eq!(forecast["null_reason"], "forecast_source_unavailable");
    }

    #[test]
    fn hours_metrics_emit_system_approved_unapproved() {
        let current = snapshot();
        let metrics = build(&current, &current);
        // 600 min / 60 = 10h system, 480/60 = 8h approved, 120/60 = 2h unapproved.
        assert_eq!(metric(&metrics, "system_ot_hours")["value"], "10");
        assert_eq!(metric(&metrics, "approved_ot_hours")["value"], "8");
        assert_eq!(metric(&metrics, "unapproved_ot_hours")["value"], "2");
    }

    #[test]
    fn attendance_target_health_reflects_threshold() {
        let mut current = snapshot();
        // 100 present / 2000 eligible = 5% — far below the 95% target.
        current.present_employee_days = 100;
        let metrics = build(&current, &current);
        let attendance = metric(&metrics, "attendance_rate");
        assert_eq!(attendance["health"], "critical");
        assert!(attendance["target"].is_object());
    }

    #[test]
    fn comparison_period_previous_is_contiguous() {
        let q = query();
        let (start, end) = comparison_period(&q);
        assert_eq!(end, q.start_date - Duration::days(1));
        assert_eq!(
            start,
            end - Duration::days(q.end_date.signed_duration_since(q.start_date).num_days())
        );
    }
}

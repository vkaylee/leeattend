use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DashboardMetricUnit {
    Count,
    Rate,
    Hours,
    Currency,
    Duration,
    Percent,
    PercentagePointDelta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DashboardMetricStatus {
    Available,
    NotApplicable,
    DataGap,
    Restricted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DashboardMetricHealth {
    Ok,
    Warning,
    Critical,
    NotApplicable,
    DataGap,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DashboardMetricDirection {
    HigherIsBetter,
    LowerIsBetter,
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DashboardMetricQuantity {
    value: Decimal,
    unit: String,
}

impl DashboardMetricQuantity {
    pub fn new(value: Decimal, unit: impl Into<String>) -> Self {
        Self {
            value,
            unit: unit.into(),
        }
    }

    pub fn value(&self) -> Decimal {
        self.value
    }

    pub fn unit(&self) -> &str {
        &self.unit
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DashboardMetricFreshness {
    as_of: DateTime<Utc>,
    lag_seconds: Option<u64>,
}

impl DashboardMetricFreshness {
    pub fn new(as_of: DateTime<Utc>, lag_seconds: Option<u64>) -> Self {
        Self { as_of, lag_seconds }
    }

    pub fn as_of(&self) -> DateTime<Utc> {
        self.as_of
    }

    pub fn lag_seconds(&self) -> Option<u64> {
        self.lag_seconds
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DashboardComparisonBasis {
    None,
    PreviousPeriod,
    SamePeriodLastYear,
    Target,
    Budget,
    Benchmark,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum DashboardTargetOperator {
    #[serde(rename = "lt")]
    Lt,
    #[serde(rename = "lte")]
    Lte,
    #[serde(rename = "eq")]
    Eq,
    #[serde(rename = "gte")]
    Gte,
    #[serde(rename = "gt")]
    Gt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DashboardMetricComparison {
    basis: DashboardComparisonBasis,
    period_start: NaiveDate,
    period_end: NaiveDate,
    value: Option<Decimal>,
    delta: Option<Decimal>,
    delta_percent: Option<Decimal>,
    delta_percentage_points: Option<Decimal>,
    status: DashboardMetricStatus,
}

impl DashboardMetricComparison {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        basis: DashboardComparisonBasis,
        period_start: NaiveDate,
        period_end: NaiveDate,
        value: Option<Decimal>,
        delta: Option<Decimal>,
        delta_percent: Option<Decimal>,
        delta_percentage_points: Option<Decimal>,
        status: DashboardMetricStatus,
    ) -> Self {
        Self {
            basis,
            period_start,
            period_end,
            value,
            delta,
            delta_percent,
            delta_percentage_points,
            status,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DashboardMetricTarget {
    value: Decimal,
    operator: DashboardTargetOperator,
}

impl DashboardMetricTarget {
    pub fn new(value: Decimal, operator: DashboardTargetOperator) -> Self {
        Self { value, operator }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DashboardMetric {
    key: String,
    value: Option<Decimal>,
    unit: DashboardMetricUnit,
    numerator: Option<DashboardMetricQuantity>,
    denominator: Option<DashboardMetricQuantity>,
    status: DashboardMetricStatus,
    health: DashboardMetricHealth,
    null_reason: Option<String>,
    definition_version: String,
    direction: DashboardMetricDirection,
    freshness: DashboardMetricFreshness,
    quality_flags: Vec<String>,
    comparison: Option<DashboardMetricComparison>,
    target: Option<DashboardMetricTarget>,
}

impl DashboardMetric {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        key: impl Into<String>,
        value: Option<Decimal>,
        unit: DashboardMetricUnit,
        numerator: Option<DashboardMetricQuantity>,
        denominator: Option<DashboardMetricQuantity>,
        status: DashboardMetricStatus,
        health: DashboardMetricHealth,
        null_reason: Option<String>,
        definition_version: impl Into<String>,
        direction: DashboardMetricDirection,
        freshness: DashboardMetricFreshness,
        quality_flags: Vec<String>,
        comparison: Option<DashboardMetricComparison>,
        target: Option<DashboardMetricTarget>,
    ) -> Self {
        Self {
            key: key.into(),
            value,
            unit,
            numerator,
            denominator,
            status,
            health,
            null_reason,
            definition_version: definition_version.into(),
            direction,
            freshness,
            quality_flags,
            comparison,
            target,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DashboardReportingPeriod {
    key: String,
    start_date: NaiveDate,
    end_date: NaiveDate,
    timezone: String,
    as_of: DateTime<Utc>,
}

impl DashboardReportingPeriod {
    pub fn new(
        key: impl Into<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        timezone: impl Into<String>,
        as_of: DateTime<Utc>,
    ) -> Self {
        Self {
            key: key.into(),
            start_date,
            end_date,
            timezone: timezone.into(),
            as_of,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DashboardMetricScope {
    kind: String,
    department_ids: Vec<uuid::Uuid>,
    location_ids: Vec<uuid::Uuid>,
    omitted_dimensions: Vec<String>,
}

impl DashboardMetricScope {
    pub fn new(
        kind: impl Into<String>,
        department_ids: Vec<uuid::Uuid>,
        location_ids: Vec<uuid::Uuid>,
        omitted_dimensions: Vec<String>,
    ) -> Self {
        Self {
            kind: kind.into(),
            department_ids,
            location_ids,
            omitted_dimensions,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DashboardMetricContext {
    period: DashboardReportingPeriod,
    comparison: String,
    filters: std::collections::BTreeMap<String, String>,
    scope: DashboardMetricScope,
}

impl DashboardMetricContext {
    pub fn new(
        period: DashboardReportingPeriod,
        comparison: impl Into<String>,
        filters: std::collections::BTreeMap<String, String>,
        scope: DashboardMetricScope,
    ) -> Self {
        Self {
            period,
            comparison: comparison.into(),
            filters,
            scope,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DashboardWorkspaceContext {
    timezone: String,
    currency: String,
}

impl DashboardWorkspaceContext {
    pub fn new(timezone: impl Into<String>, currency: impl Into<String>) -> Self {
        Self {
            timezone: timezone.into(),
            currency: currency.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DashboardActionTypeCount {
    #[serde(rename = "type")]
    action_type: String,
    count: u64,
}

impl DashboardActionTypeCount {
    pub fn new(action_type: impl Into<String>, count: u64) -> Self {
        Self {
            action_type: action_type.into(),
            count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DashboardActionsSummary {
    total: u64,
    overdue: u64,
    by_type: Vec<DashboardActionTypeCount>,
}

impl DashboardActionsSummary {
    pub fn new(total: u64, overdue: u64, by_type: Vec<DashboardActionTypeCount>) -> Self {
        Self {
            total,
            overdue,
            by_type,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DashboardQualitySource {
    key: String,
    status: String,
    coverage_ratio: Option<Decimal>,
    freshness: DashboardMetricFreshness,
    warnings: Vec<String>,
}

impl DashboardQualitySource {
    pub fn new(
        key: impl Into<String>,
        status: impl Into<String>,
        coverage_ratio: Option<Decimal>,
        freshness: DashboardMetricFreshness,
        warnings: Vec<String>,
    ) -> Self {
        Self {
            key: key.into(),
            status: status.into(),
            coverage_ratio,
            freshness,
            warnings,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DashboardDataQuality {
    status: String,
    sources: Vec<DashboardQualitySource>,
}

impl DashboardDataQuality {
    pub fn new(status: impl Into<String>, sources: Vec<DashboardQualitySource>) -> Self {
        Self {
            status: status.into(),
            sources,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DashboardMetricsResponse {
    contract_version: String,
    workspace: DashboardWorkspaceContext,
    context: DashboardMetricContext,
    metrics: Vec<DashboardMetric>,
    insights: Vec<DashboardInsight>,
    actions: DashboardActionsSummary,
    data_quality: DashboardDataQuality,
    generated_at: DateTime<Utc>,
}

impl DashboardMetricsResponse {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        contract_version: impl Into<String>,
        workspace: DashboardWorkspaceContext,
        context: DashboardMetricContext,
        metrics: Vec<DashboardMetric>,
        insights: Vec<DashboardInsight>,
        actions: DashboardActionsSummary,
        data_quality: DashboardDataQuality,
        generated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            contract_version: contract_version.into(),
            workspace,
            context,
            metrics,
            insights,
            actions,
            data_quality,
            generated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DashboardInsightType {
    Trend,
    Threshold,
    Concentration,
    DataQuality,
    Workflow,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DashboardInsightSeverity {
    Info,
    Watch,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DashboardInsight {
    id: String,
    #[serde(rename = "type")]
    insight_type: DashboardInsightType,
    severity: DashboardInsightSeverity,
    metric_key: String,
    title_key: String,
    summary_key: String,
    parameters: std::collections::BTreeMap<String, String>,
    requires_permission: Option<String>,
    scope: String,
    dismissible: bool,
}

impl DashboardInsight {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        insight_type: DashboardInsightType,
        severity: DashboardInsightSeverity,
        metric_key: impl Into<String>,
        title_key: impl Into<String>,
        summary_key: impl Into<String>,
        parameters: std::collections::BTreeMap<String, String>,
        requires_permission: Option<String>,
        scope: impl Into<String>,
        dismissible: bool,
    ) -> Self {
        Self {
            id: id.into(),
            insight_type,
            severity,
            metric_key: metric_key.into(),
            title_key: title_key.into(),
            summary_key: summary_key.into(),
            parameters,
            requires_permission,
            scope: scope.into(),
            dismissible,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn sample_metric(status: DashboardMetricStatus) -> DashboardMetric {
        DashboardMetric::new(
            "absence_rate",
            None,
            DashboardMetricUnit::Percent,
            None,
            None,
            status,
            DashboardMetricHealth::DataGap,
            Some("missing_source".to_string()),
            "absence-rate.v1",
            DashboardMetricDirection::LowerIsBetter,
            DashboardMetricFreshness::new(
                Utc.with_ymd_and_hms(2026, 8, 13, 0, 0, 0).unwrap(),
                None,
            ),
            vec!["attendance_coverage_incomplete".to_string()],
            None,
            None,
        )
    }

    #[test]
    fn serializes_structured_quantity_and_separate_delta_units() {
        let quantity = DashboardMetricQuantity::new(Decimal::new(35, 0), "eligible_employee_days");
        let metric = DashboardMetric::new(
            "attendance_rate",
            Some(Decimal::new(942, 1)),
            DashboardMetricUnit::Percent,
            Some(quantity),
            Some(DashboardMetricQuantity::new(
                Decimal::new(1000, 0),
                "eligible_employee_days",
            )),
            DashboardMetricStatus::Available,
            DashboardMetricHealth::Ok,
            None,
            "attendance-rate.v1",
            DashboardMetricDirection::HigherIsBetter,
            DashboardMetricFreshness::new(
                Utc.with_ymd_and_hms(2026, 8, 13, 0, 0, 0).unwrap(),
                Some(120),
            ),
            Vec::new(),
            Some(DashboardMetricComparison::new(
                DashboardComparisonBasis::PreviousPeriod,
                NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 7, 31).unwrap(),
                Some(Decimal::new(930, 1)),
                Some(Decimal::new(12, 1)),
                Some(Decimal::new(129, 2)),
                Some(Decimal::new(12, 1)),
                DashboardMetricStatus::Available,
            )),
            Some(DashboardMetricTarget::new(
                Decimal::new(95, 0),
                DashboardTargetOperator::Gte,
            )),
        );
        let json = serde_json::to_value(metric).unwrap();
        assert_eq!(json["key"], "attendance_rate");
        assert_eq!(json["numerator"]["unit"], "eligible_employee_days");
        assert_eq!(json["comparison"]["delta_percentage_points"], "1.2");
        assert_eq!(json["comparison"]["delta_percent"], "1.29");
    }

    #[test]
    fn preserves_explicit_null_for_unavailable_value() {
        let json = serde_json::to_value(sample_metric(DashboardMetricStatus::DataGap)).unwrap();
        assert!(json["value"].is_null());
        assert!(json["numerator"].is_null());
        assert_eq!(json["status"], "data_gap");
        assert_eq!(json["null_reason"], "missing_source");
    }

    #[test]
    fn restricted_metric_does_not_require_numeric_value() {
        let metric = sample_metric(DashboardMetricStatus::Restricted);
        assert!(metric.value.is_none());
        assert!(metric.numerator.is_none());
        assert!(metric.denominator.is_none());
    }
}

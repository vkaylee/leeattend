use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::custom_field::CustomFieldValueResponse;
use crate::models::dashboard_metrics::DashboardComparisonBasis;
use crate::models::enums::RecruitmentSourceType;

// ── Filter Models ───────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct EmployeeFilterOptions {
    search_query: Option<String>,
    page: i64,
    limit: i64,
    sort_by: Option<String>,
    sort_dir: Option<String>,
    department_ids: Option<Vec<Uuid>>,
    permission: Option<String>,
    filter_ids: Option<Vec<Uuid>>,
    status: Option<String>,
}

impl EmployeeFilterOptions {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        search_query: Option<String>,
        page: i64,
        limit: i64,
        sort_by: Option<String>,
        sort_dir: Option<String>,
        department_ids: Option<Vec<Uuid>>,
        permission: Option<String>,
        filter_ids: Option<Vec<Uuid>>,
    ) -> Self {
        Self {
            search_query,
            page,
            limit,
            sort_by,
            sort_dir,
            department_ids,
            permission,
            filter_ids,
            status: None,
        }
    }

    #[must_use]
    pub fn with_search_query(mut self, q: Option<String>) -> Self {
        self.search_query = q;
        self
    }

    #[must_use]
    pub fn with_page(mut self, p: i64) -> Self {
        self.page = p;
        self
    }

    #[must_use]
    pub fn with_limit(mut self, l: i64) -> Self {
        self.limit = l;
        self
    }

    #[must_use]
    pub fn with_sort_by(mut self, s: Option<String>) -> Self {
        self.sort_by = s;
        self
    }

    #[must_use]
    pub fn with_sort_dir(mut self, d: Option<String>) -> Self {
        self.sort_dir = d;
        self
    }

    #[must_use]
    pub fn with_department_ids(mut self, depts: Option<Vec<Uuid>>) -> Self {
        self.department_ids = depts;
        self
    }

    #[must_use]
    pub fn with_permission(mut self, perm: Option<String>) -> Self {
        self.permission = perm;
        self
    }

    #[must_use]
    pub fn with_filter_ids(mut self, ids: Option<Vec<Uuid>>) -> Self {
        self.filter_ids = ids;
        self
    }

    #[must_use]
    pub fn with_status(mut self, status: Option<String>) -> Self {
        self.status = status;
        self
    }

    pub fn search_query(&self) -> Option<&str> {
        self.search_query.as_deref()
    }

    pub fn page(&self) -> i64 {
        self.page
    }

    pub fn limit(&self) -> i64 {
        self.limit
    }

    pub fn sort_by(&self) -> Option<&str> {
        self.sort_by.as_deref()
    }

    pub fn sort_dir(&self) -> Option<&str> {
        self.sort_dir.as_deref()
    }

    pub fn department_ids(&self) -> Option<&[Uuid]> {
        self.department_ids.as_deref()
    }

    pub fn permission(&self) -> Option<&str> {
        self.permission.as_deref()
    }

    pub fn filter_ids(&self) -> Option<&[Uuid]> {
        self.filter_ids.as_deref()
    }

    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }
}

#[derive(Debug, Clone)]
pub struct EmployeePaginationArgs<'a> {
    filter_ids: Option<&'a Vec<Uuid>>,
    department_id: Option<Uuid>,
    page: i64,
    limit: i64,
    sort_desc: bool,
    active_during: Option<(NaiveDate, NaiveDate)>,
    status_filter: Option<String>,
}

impl<'a> EmployeePaginationArgs<'a> {
    pub fn new(
        filter_ids: Option<&'a Vec<Uuid>>,
        department_id: Option<Uuid>,
        page: i64,
        limit: i64,
        sort_desc: bool,
        active_during: Option<(NaiveDate, NaiveDate)>,
        status_filter: Option<String>,
    ) -> Self {
        Self {
            filter_ids,
            department_id,
            page,
            limit,
            sort_desc,
            active_during,
            status_filter,
        }
    }

    pub fn filter_ids(&self) -> Option<&'a Vec<Uuid>> {
        self.filter_ids
    }

    pub fn department_id(&self) -> Option<Uuid> {
        self.department_id
    }

    pub fn page(&self) -> i64 {
        self.page
    }

    pub fn limit(&self) -> i64 {
        self.limit
    }

    pub fn sort_desc(&self) -> bool {
        self.sort_desc
    }

    pub fn active_during(&self) -> Option<(NaiveDate, NaiveDate)> {
        self.active_during
    }

    pub fn status_filter(&self) -> Option<&str> {
        self.status_filter.as_deref()
    }
}

// ── Recruitment Source Models ───────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct RecruitmentSource {
    id: Uuid,
    workspace_id: Uuid,
    source_type: RecruitmentSourceType,
    name: String,
    referrer_employee_id: Option<Uuid>,
    referral_bonus_amount: Option<Decimal>,
    referral_bonus_paid: bool,
    agency_contact_name: Option<String>,
    agency_contact_email: Option<String>,
    agency_fee_amount: Option<Decimal>,
    agency_fee_paid: bool,
    notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl RecruitmentSource {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        source_type: RecruitmentSourceType,
        name: String,
        referrer_employee_id: Option<Uuid>,
        referral_bonus_amount: Option<Decimal>,
        referral_bonus_paid: bool,
        agency_contact_name: Option<String>,
        agency_contact_email: Option<String>,
        agency_fee_amount: Option<Decimal>,
        agency_fee_paid: bool,
        notes: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            source_type,
            name,
            referrer_employee_id,
            referral_bonus_amount,
            referral_bonus_paid,
            agency_contact_name,
            agency_contact_email,
            agency_fee_amount,
            agency_fee_paid,
            notes,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn source_type(&self) -> &RecruitmentSourceType {
        &self.source_type
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn referrer_employee_id(&self) -> Option<Uuid> {
        self.referrer_employee_id
    }

    pub fn referral_bonus_amount(&self) -> Option<Decimal> {
        self.referral_bonus_amount
    }

    pub fn referral_bonus_paid(&self) -> bool {
        self.referral_bonus_paid
    }

    pub fn agency_contact_name(&self) -> Option<&str> {
        self.agency_contact_name.as_deref()
    }

    pub fn agency_contact_email(&self) -> Option<&str> {
        self.agency_contact_email.as_deref()
    }

    pub fn agency_fee_amount(&self) -> Option<Decimal> {
        self.agency_fee_amount
    }

    pub fn agency_fee_paid(&self) -> bool {
        self.agency_fee_paid
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct CreateRecruitmentSourceRequest {
    pub source_type: RecruitmentSourceType,
    pub name: String,
    pub referrer_employee_id: Option<Uuid>,
    pub referral_bonus_amount: Option<Decimal>,
    pub agency_contact_name: Option<String>,
    pub agency_contact_email: Option<String>,
    pub agency_fee_amount: Option<Decimal>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct UpdateRecruitmentSourceRequest {
    pub name: Option<String>,
    pub referrer_employee_id: Option<Uuid>,
    pub referral_bonus_amount: Option<Decimal>,
    pub referral_bonus_paid: Option<bool>,
    pub agency_contact_name: Option<String>,
    pub agency_contact_email: Option<String>,
    pub agency_fee_amount: Option<Decimal>,
    pub agency_fee_paid: Option<bool>,
    pub notes: Option<String>,
}

/// Dashboard chart data point for recruitment source breakdown
#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct RecruitmentSourceBreakdown {
    pub source_type: String,
    pub source_name: String,
    pub count: i64,
}

// ── Turnover Analytics Models ──────────────────────────────────────

/// Tenure distribution bucket for terminated employees
#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct TurnoverTenureBucket {
    pub bucket: String,
    pub count: i64,
}

/// Turnover breakdown by recruitment source
#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct TurnoverBySource {
    pub source_type: String,
    pub source_name: String,
    pub count: i64,
}

/// Turnover breakdown by department
#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct TurnoverByDepartment {
    pub department_name: String,
    pub count: i64,
}

/// Combined turnover analytics response
#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct TurnoverAnalytics {
    pub tenure_distribution: Vec<TurnoverTenureBucket>,
    pub by_source: Vec<TurnoverBySource>,
    pub by_department: Vec<TurnoverByDepartment>,
    pub total_terminated: i64,
    pub period_start: String,
    pub period_end: String,
    /// Turnover rate with mandatory denominator. `value` is `None` when
    /// `average_headcount == 0` — never a fabricated `0.0`.
    pub turnover_rate: TurnoverRate,
    /// Mean of month-start headcounts across the period (the turnover denominator).
    pub average_headcount: i64,
    pub voluntary_count: i64,
    pub involuntary_count: i64,
    pub unknown_classification_count: i64,
    /// Retention rate = (headcount_start − leavers active at start) / headcount_start × 100.
    /// `None` when headcount_start == 0.
    pub retention_rate: Option<Decimal>,
    /// Leavers with tenure < 90 days / average headcount.
    pub early_attrition: TurnoverRate,
    /// ALWAYS `None` in v1 — no performance-rating data source. See `NO_DATA_SOURCE` warning.
    pub regrettable_attrition: Option<Decimal>,
    pub comparison: Option<TurnoverComparison>,
    pub target: TurnoverTarget,
    /// Empty when the period spans fewer than two months.
    pub monthly_trend: Vec<TurnoverTrendPoint>,
    pub cohort_retention: Vec<CohortRetentionRow>,
    pub termination_reasons: Vec<TurnoverReasonRow>,
    pub classification: Vec<TurnoverClassification>,
    pub by_employment_type: Vec<TurnoverByEmploymentType>,
    pub by_location: Vec<TurnoverByLocation>,
    pub by_manager: Vec<TurnoverByManager>,
    pub warnings: Vec<TurnoverWarning>,
}

impl TurnoverAnalytics {
    /// Empty analytics for a period — used by test mocks. Never fabricates a rate:
    /// the rate keeps a zero denominator so `value` stays `None`.
    pub fn empty(start_date: NaiveDate, end_date: NaiveDate) -> Self {
        Self {
            tenure_distribution: vec![],
            by_source: vec![],
            by_department: vec![],
            total_terminated: 0,
            period_start: start_date.to_string(),
            period_end: end_date.to_string(),
            turnover_rate: TurnoverRate::percent(0, 0),
            average_headcount: 0,
            voluntary_count: 0,
            involuntary_count: 0,
            unknown_classification_count: 0,
            retention_rate: None,
            early_attrition: TurnoverRate::percent(0, 0),
            regrettable_attrition: None,
            comparison: None,
            target: TurnoverTarget {
                value: Decimal::from(15),
                operator: "lte".to_string(),
                source: "default".to_string(),
            },
            monthly_trend: vec![],
            cohort_retention: vec![],
            termination_reasons: vec![],
            classification: vec![],
            by_employment_type: vec![],
            by_location: vec![],
            by_manager: vec![],
            warnings: vec![],
        }
    }
}

/// Turnover rate: `leavers / average_headcount × 100`. Mirrors `RateQuantity`'s
/// zero-denominator rule — `value` is `None` when the denominator is `0`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, utoipa::ToSchema)]
pub struct TurnoverRate {
    pub value: Option<Decimal>,
    pub numerator: i64,
    pub denominator: i64,
    /// `"employees"` — the unit of the numerator/denominator.
    pub unit: String,
}

impl TurnoverRate {
    pub fn percent(numerator: i64, denominator: i64) -> Self {
        let unit = "employees".to_string();
        if denominator <= 0 {
            return Self {
                value: None,
                numerator: numerator.max(0),
                denominator,
                unit,
            };
        }
        let value =
            Decimal::from(numerator.max(0)) / Decimal::from(denominator) * Decimal::from(100);
        Self {
            value: Some(value),
            numerator: numerator.max(0),
            denominator,
            unit,
        }
    }
}

/// A percentage-point comparison against a previous period.
#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct TurnoverComparison {
    pub basis: DashboardComparisonBasis,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub rate: Option<TurnoverRate>,
    /// `current.rate − comparison.rate`, in percentage points.
    pub delta_percentage_points: Option<Decimal>,
}

/// A structured warning surfaced to the UI. `code` is a stable machine key.
#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct TurnoverWarning {
    pub code: String,
    pub message: String,
}

/// Termination-reason breakdown row.
#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct TurnoverReasonRow {
    pub reason: String,
    pub count: i64,
}

/// Voluntary / involuntary / unknown classification row (derived from `termination_reason`).
#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct TurnoverClassification {
    pub classification: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct TurnoverByEmploymentType {
    pub employment_type: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct TurnoverByLocation {
    pub location_name: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct TurnoverByManager {
    pub manager_name: String,
    pub count: i64,
}

/// One month bucket in the monthly turnover trend.
#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct TurnoverTrendPoint {
    pub month: String,
    pub headcount: i64,
    pub leavers: i64,
    pub turnover_rate: Option<TurnoverRate>,
}

/// One (cohort, months-since-hire) cell in the cohort retention matrix.
#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct CohortRetentionRow {
    pub cohort_month: String,
    pub months_since_hire: i64,
    pub cohort_size: i64,
    pub active_count: i64,
    pub retention_rate: Option<Decimal>,
}

/// Turnover target/benchmark. `operator` is a comparison (`lte` for "at most").
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, utoipa::ToSchema)]
pub struct TurnoverTarget {
    pub value: Decimal,
    pub operator: String,
    /// `"workspace_config"` when resolved from `workspace_config_entries`, else `"default"`.
    pub source: String,
}

/// Request struct for the turnover analytics query.
#[derive(Debug, Clone)]
pub struct TurnoverAnalyticsQuery {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub comparison_basis: DashboardComparisonBasis,
    pub target: TurnoverTarget,
}

// ── Recruitment Effectiveness Models ───────────────────────────────

/// One source row in the recruitment effectiveness ranking.
///
/// Never interpret `hires` as a quality signal on its own — retention and cost
/// are the quality dimensions, and every row carries its sample size.
#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct RecruitmentEffectivenessSource {
    /// `None` when the hire has no linked source (null semantics).
    pub source_id: Option<Uuid>,
    /// `"direct"` | `"referral"` | `"agency"` | `"unknown"`.
    pub source_type: String,
    /// `COALESCE(rs.name, 'Unknown source')`.
    pub source_name: String,
    pub hires: i64,
    /// hires / total_hires × 100. `None` when total_hires == 0.
    pub hires_ratio: Option<Decimal>,
    /// Hires still active 90 days post-hire (or not terminated within 90 days).
    pub retained_90d: i64,
    /// retained_90d / hires × 100. `None` when hires == 0.
    pub retention_90d: Option<Decimal>,
    /// Agency fee or referral bonus ÷ hires. `None` for direct sources or hires == 0.
    pub cost_per_hire: Option<Decimal>,
    /// `false` for direct sources (no cost data exists) or when no fee is configured.
    pub cost_per_hire_available: bool,
    /// Terminations of hires-by-this-source within the period.
    pub turnover_in_range: i64,
}

/// One stage of the honest conversion funnel.
///
/// `count: None` + `data_source: None` means the stage has no data source in the
/// system yet (e.g. applicants) — the UI must render this as "data unavailable",
/// never as a fabricated zero.
#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct RecruitmentFunnelStage {
    /// `"applicants"` | `"screening"` | `"interview"` | `"offer"` | `"hired"` | `"retained_90d"`.
    pub stage: String,
    pub count: Option<i64>,
    /// e.g. `Some("hires")`; `None` when no data source exists.
    pub data_source: Option<String>,
    /// stage count / previous stage count × 100. `None` when previous is None or 0.
    pub conversion_rate: Option<Decimal>,
}

/// Overview KPIs for the recruitment effectiveness dashboard.
#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct RecruitmentEffectivenessOverview {
    pub hires: i64,
    /// Weighted average cost per hire across all sources. `None` when no cost data.
    pub average_cost_per_hire: Option<Decimal>,
    pub cost_available: bool,
    pub retained_90d: i64,
    /// retained_90d / hires × 100. `None` when hires == 0.
    pub retention_90d: Option<Decimal>,
    /// Terminations of sourced hires within the period.
    pub turnover_in_range: i64,
    /// Hires in the comparison window. `None` when no comparison requested.
    pub comparison_hires: Option<i64>,
    /// `current.comparison_hires − previous.comparison_hires`. `None` when no comparison.
    pub comparison_hires_delta: Option<i64>,
}

/// One month bucket in the monthly hires trend.
#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct RecruitmentTrendPoint {
    /// `"YYYY-MM"`.
    pub month: String,
    pub hires: i64,
}

/// One unavailable metric — a self-describing data-availability gap.
#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct RecruitmentDataGap {
    /// `"applicants"` | `"time_to_hire"` | `"quality_of_hire"` | ...
    pub metric: String,
    /// i18n key, e.g. `"dashboard.hr.effective_gap_no_applicant_data"`.
    pub reason: String,
}

/// Combined recruitment effectiveness analytics response.
///
/// The period-over-period comparison lives on `overview` (`comparison_hires` /
/// `comparison_hires_delta`) rather than a reused `TurnoverComparison`, which is
/// percentage-point oriented and would overload the hires semantics.
#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct RecruitmentEffectivenessAnalytics {
    pub period_start: String,
    pub period_end: String,
    pub total_hires: i64,
    pub overview: RecruitmentEffectivenessOverview,
    pub funnel: Vec<RecruitmentFunnelStage>,
    pub by_source: Vec<RecruitmentEffectivenessSource>,
    /// Empty when the period spans fewer than two months.
    pub hires_trend: Vec<RecruitmentTrendPoint>,
    /// Structured availability report — every unavailable metric is listed here.
    pub data_gaps: Vec<RecruitmentDataGap>,
    pub warnings: Vec<TurnoverWarning>,
}

impl RecruitmentEffectivenessAnalytics {
    /// Empty analytics for a period — used by test mocks. Never fabricates a rate:
    /// funnel pre-hire stages stay `count: None`, retention stays `None`.
    pub fn empty(start_date: NaiveDate, end_date: NaiveDate) -> Self {
        Self {
            period_start: start_date.to_string(),
            period_end: end_date.to_string(),
            total_hires: 0,
            overview: RecruitmentEffectivenessOverview {
                hires: 0,
                average_cost_per_hire: None,
                cost_available: false,
                retained_90d: 0,
                retention_90d: None,
                turnover_in_range: 0,
                comparison_hires: None,
                comparison_hires_delta: None,
            },
            funnel: vec![
                RecruitmentFunnelStage {
                    stage: "applicants".into(),
                    count: None,
                    data_source: None,
                    conversion_rate: None,
                },
                RecruitmentFunnelStage {
                    stage: "screening".into(),
                    count: None,
                    data_source: None,
                    conversion_rate: None,
                },
                RecruitmentFunnelStage {
                    stage: "interview".into(),
                    count: None,
                    data_source: None,
                    conversion_rate: None,
                },
                RecruitmentFunnelStage {
                    stage: "offer".into(),
                    count: None,
                    data_source: None,
                    conversion_rate: None,
                },
                RecruitmentFunnelStage {
                    stage: "hired".into(),
                    count: Some(0),
                    data_source: Some("hires".into()),
                    conversion_rate: None,
                },
                RecruitmentFunnelStage {
                    stage: "retained_90d".into(),
                    count: Some(0),
                    data_source: Some("hires".into()),
                    conversion_rate: None,
                },
            ],
            by_source: vec![],
            hires_trend: vec![],
            data_gaps: Self::data_gaps(),
            warnings: vec![],
        }
    }

    /// The fixed list of unavailable metrics with their i18n reason keys.
    pub fn data_gaps() -> Vec<RecruitmentDataGap> {
        vec![
            RecruitmentDataGap {
                metric: "applicants".into(),
                reason: "dashboard.hr.effective_gap_no_applicant_data".into(),
            },
            RecruitmentDataGap {
                metric: "screening".into(),
                reason: "dashboard.hr.effective_gap_no_screening_data".into(),
            },
            RecruitmentDataGap {
                metric: "interview".into(),
                reason: "dashboard.hr.effective_gap_no_interview_data".into(),
            },
            RecruitmentDataGap {
                metric: "offer".into(),
                reason: "dashboard.hr.effective_gap_no_offer_data".into(),
            },
            RecruitmentDataGap {
                metric: "offer_acceptance".into(),
                reason: "dashboard.hr.effective_gap_no_offer_acceptance".into(),
            },
            RecruitmentDataGap {
                metric: "time_to_hire".into(),
                reason: "dashboard.hr.effective_gap_no_time_to_hire".into(),
            },
            RecruitmentDataGap {
                metric: "time_to_fill".into(),
                reason: "dashboard.hr.effective_gap_no_time_to_fill".into(),
            },
            RecruitmentDataGap {
                metric: "quality_of_hire".into(),
                reason: "dashboard.hr.effective_gap_no_performance_data".into(),
            },
        ]
    }
}

/// Request struct for the recruitment effectiveness query.
#[derive(Debug, Clone)]
pub struct RecruitmentEffectivenessQuery {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub comparison_basis: DashboardComparisonBasis,
    /// Empty = no department filter.
    pub department_ids: Vec<Uuid>,
    /// Empty = no location filter.
    pub location_ids: Vec<Uuid>,
    /// Empty = no job-role filter (matches `employments.job_title`).
    pub job_roles: Vec<String>,
    /// Empty = no source-type filter (`direct` | `referral` | `agency`).
    pub source_types: Vec<String>,
    /// Empty = no source filter.
    pub source_ids: Vec<Uuid>,
}

// ── Employment / Employee Models ───────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct Employee {
    id: Uuid,
    workspace_id: Uuid,
    user_id: Option<Uuid>,
    full_name: String,
    employee_id_number: String,
    tags: Vec<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    department_name: Option<String>,
    active_employment_id: Option<Uuid>,
    employer_id: Option<Uuid>,
    primary_location_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_fields: Option<Vec<CustomFieldValueResponse>>,
}

impl Employee {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        user_id: Option<Uuid>,
        full_name: String,
        employee_id_number: String,
        tags: Vec<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        department_name: Option<String>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            user_id,
            full_name,
            employee_id_number,
            tags,
            created_at,
            updated_at,
            department_name,
            active_employment_id: None,
            employer_id: None,
            primary_location_id: None,
            custom_fields: None,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn custom_fields(&self) -> Option<&[CustomFieldValueResponse]> {
        self.custom_fields.as_deref()
    }

    #[must_use]
    pub fn with_custom_fields(mut self, custom_fields: Vec<CustomFieldValueResponse>) -> Self {
        self.custom_fields = Some(custom_fields);
        self
    }

    pub fn user_id(&self) -> Option<Uuid> {
        self.user_id
    }

    pub fn full_name(&self) -> &str {
        &self.full_name
    }

    pub fn employee_id_number(&self) -> &str {
        &self.employee_id_number
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn department_name(&self) -> Option<&str> {
        self.department_name.as_deref()
    }

    pub fn active_employment_id(&self) -> Option<Uuid> {
        self.active_employment_id
    }

    pub fn is_active(&self) -> bool {
        self.active_employment_id.is_some()
    }

    pub fn employer_id(&self) -> Option<Uuid> {
        self.employer_id
    }

    pub fn primary_location_id(&self) -> Option<Uuid> {
        self.primary_location_id
    }

    #[must_use]
    pub fn with_employment_info(
        mut self,
        active_employment_id: Option<Uuid>,
        employer_id: Option<Uuid>,
        primary_location_id: Option<Uuid>,
    ) -> Self {
        self.active_employment_id = active_employment_id;
        self.employer_id = employer_id;
        self.primary_location_id = primary_location_id;
        self
    }

    #[cfg(test)]
    pub fn set_full_name(&mut self, full_name: String) {
        self.full_name = full_name;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct AssignmentRuleConditions {
    department_id: Option<Uuid>,
    employee_id_numbers: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    location_id: Option<Uuid>,
    employer_id: Option<Uuid>,
}

impl AssignmentRuleConditions {
    pub fn new(
        department_id: Option<Uuid>,
        employee_id_numbers: Option<Vec<String>>,
        tags: Option<Vec<String>>,
        location_id: Option<Uuid>,
        employer_id: Option<Uuid>,
    ) -> Self {
        Self {
            department_id,
            employee_id_numbers,
            tags,
            location_id,
            employer_id,
        }
    }

    pub fn department_id(&self) -> Option<Uuid> {
        self.department_id
    }

    pub fn employee_id_numbers(&self) -> Option<&[String]> {
        self.employee_id_numbers.as_deref()
    }

    pub fn tags(&self) -> Option<&[String]> {
        self.tags.as_deref()
    }

    pub fn location_id(&self) -> Option<Uuid> {
        self.location_id
    }

    pub fn employer_id(&self) -> Option<Uuid> {
        self.employer_id
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct AssignmentRule {
    id: Uuid,
    workspace_id: Uuid,
    pattern_id: Uuid,
    name: String,
    priority: i32,
    conditions: AssignmentRuleConditions,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
}

impl AssignmentRule {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        pattern_id: Uuid,
        name: String,
        priority: i32,
        conditions: AssignmentRuleConditions,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            pattern_id,
            name,
            priority,
            conditions,
            is_active,
            created_at,
            updated_at,
            start_date,
            end_date,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn pattern_id(&self) -> Uuid {
        self.pattern_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn priority(&self) -> i32 {
        self.priority
    }

    pub fn conditions(&self) -> &AssignmentRuleConditions {
        &self.conditions
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn start_date(&self) -> Option<NaiveDate> {
        self.start_date
    }

    pub fn end_date(&self) -> Option<NaiveDate> {
        self.end_date
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct CreateAssignmentRuleRequest {
    pub pattern_id: Uuid,
    pub name: String,
    pub priority: i32,
    pub conditions: AssignmentRuleConditions,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct UpdateAssignmentRuleRequest {
    pub pattern_id: Option<Uuid>,
    pub name: Option<String>,
    pub priority: Option<i32>,
    pub conditions: Option<AssignmentRuleConditions>,
    pub is_active: Option<bool>,
    pub max_concurrent_workers: Option<i32>,
    #[serde(
        default,
        deserialize_with = "crate::utils::serde_helpers::deserialize_option_option"
    )]
    pub start_date: Option<Option<NaiveDate>>,
    #[serde(
        default,
        deserialize_with = "crate::utils::serde_helpers::deserialize_option_option"
    )]
    pub end_date: Option<Option<NaiveDate>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct RehireEmployeeRequest {
    pub hire_date: NaiveDate,
    pub employer_id: Uuid,
    pub primary_location_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, utoipa::ToSchema)]
#[schema(as = String)]
pub struct SanitizedEmployeeId(String);

impl SanitizedEmployeeId {
    pub fn new(raw: &str) -> Self {
        Self(crate::utils::normalize_employee_id(raw))
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> serde::Deserialize<'de> for SanitizedEmployeeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(SanitizedEmployeeId::new(&s))
    }
}

impl serde::Serialize for SanitizedEmployeeId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl std::fmt::Display for SanitizedEmployeeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for SanitizedEmployeeId {
    fn from(s: String) -> Self {
        SanitizedEmployeeId::new(&s)
    }
}

impl From<&str> for SanitizedEmployeeId {
    fn from(s: &str) -> Self {
        SanitizedEmployeeId::new(s)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct CreateEmployeeRequest {
    pub user_id: Option<Uuid>,
    pub full_name: String,
    pub employee_id_number: SanitizedEmployeeId,
    pub hire_date: NaiveDate,
    pub tags: Option<Vec<String>>,
    pub recruitment_source_id: Option<Uuid>,
    pub employer_id: Uuid,
    pub primary_location_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct UpdateEmployeeRequest {
    #[serde(
        default,
        deserialize_with = "crate::utils::serde_helpers::deserialize_option_option"
    )]
    pub user_id: Option<Option<Uuid>>,
    pub full_name: Option<String>,
    pub employee_id_number: Option<SanitizedEmployeeId>,
    pub tags: Option<Vec<String>>,
    pub max_concurrent_workers: Option<i32>,
    pub recruitment_source_id: Option<Uuid>,
    pub force_transfer: Option<bool>,
}

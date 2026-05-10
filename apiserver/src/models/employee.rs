use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::enums::RecruitmentSourceType;

// ── Filter Models ───────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct EmployeeFilterOptions {
    pub search_query: Option<String>,
    pub page: i64,
    pub limit: i64,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
    pub department_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone)]
pub struct EmployeePaginationArgs<'a> {
    pub filter_ids: Option<&'a Vec<Uuid>>,
    pub department_id: Option<Uuid>,
    pub page: i64,
    pub limit: i64,
    pub sort_desc: bool,
}

// ── Recruitment Source Models ───────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct RecruitmentSource {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub source_type: RecruitmentSourceType,
    pub name: String,
    pub referrer_employee_id: Option<Uuid>,
    pub referral_bonus_amount: Option<Decimal>,
    pub referral_bonus_paid: bool,
    pub agency_contact_name: Option<String>,
    pub agency_contact_email: Option<String>,
    pub agency_fee_amount: Option<Decimal>,
    pub agency_fee_paid: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecruitmentSourceBreakdown {
    pub source_type: String,
    pub source_name: String,
    pub count: i64,
}

// ── Turnover Analytics Models ──────────────────────────────────────

/// Tenure distribution bucket for terminated employees
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TurnoverTenureBucket {
    pub bucket: String,
    pub count: i64,
}

/// Turnover breakdown by recruitment source
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TurnoverBySource {
    pub source_type: String,
    pub source_name: String,
    pub count: i64,
}

/// Turnover breakdown by department
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TurnoverByDepartment {
    pub department_name: String,
    pub count: i64,
}

/// Combined turnover analytics response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TurnoverAnalytics {
    pub tenure_distribution: Vec<TurnoverTenureBucket>,
    pub by_source: Vec<TurnoverBySource>,
    pub by_department: Vec<TurnoverByDepartment>,
    pub total_terminated: i64,
    pub period_start: String,
    pub period_end: String,
}

// ── Employment / Employee Models ───────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Employee {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Option<Uuid>,
    pub full_name: String,
    pub employee_id_number: String,
    pub hire_date: NaiveDate,
    pub termination_date: Option<NaiveDate>,
    pub tags: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub department_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssignmentRuleConditions {
    pub department_id: Option<Uuid>,
    pub employee_id_numbers: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct AssignmentRule {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub pattern_id: Uuid,
    pub name: String,
    pub priority: i32,
    pub conditions: sqlx::types::Json<AssignmentRuleConditions>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateAssignmentRuleRequest {
    pub pattern_id: Uuid,
    pub name: String,
    pub priority: i32,
    pub conditions: AssignmentRuleConditions,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateAssignmentRuleRequest {
    pub pattern_id: Option<Uuid>,
    pub name: Option<String>,
    pub priority: Option<i32>,
    pub conditions: Option<AssignmentRuleConditions>,
    pub is_active: Option<bool>,
    pub max_concurrent_workers: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RehireEmployeeRequest {
    pub hire_date: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateEmployeeRequest {
    pub user_id: Option<Uuid>,
    pub full_name: String,
    pub employee_id_number: SanitizedEmployeeId,
    pub hire_date: NaiveDate,
    pub tags: Option<Vec<String>>,
    pub recruitment_source_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateEmployeeRequest {
    pub user_id: Option<Uuid>,
    pub full_name: Option<String>,
    pub employee_id_number: Option<SanitizedEmployeeId>,
    pub hire_date: Option<NaiveDate>,
    pub termination_date: Option<NaiveDate>,
    pub tags: Option<Vec<String>>,
    pub is_active: Option<bool>,
    pub max_concurrent_workers: Option<i32>,
    pub recruitment_source_id: Option<Uuid>,
}

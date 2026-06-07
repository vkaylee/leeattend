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
    pub permission: Option<String>,
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
}

// ── Employment / Employee Models ───────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct Employee {
    id: Uuid,
    workspace_id: Uuid,
    user_id: Option<Uuid>,
    full_name: String,
    employee_id_number: String,
    original_hire_date: NaiveDate,
    hire_date: NaiveDate,
    termination_date: Option<NaiveDate>,
    tags: Vec<String>,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    department_name: Option<String>,
    primary_location_id: Uuid,
    employer_id: Option<Uuid>,
}

impl Employee {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        user_id: Option<Uuid>,
        full_name: String,
        employee_id_number: String,
        original_hire_date: NaiveDate,
        hire_date: NaiveDate,
        termination_date: Option<NaiveDate>,
        tags: Vec<String>,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        department_name: Option<String>,
        primary_location_id: Uuid,
        employer_id: Option<Uuid>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            user_id,
            full_name,
            employee_id_number,
            original_hire_date,
            hire_date,
            termination_date,
            tags,
            is_active,
            created_at,
            updated_at,
            department_name,
            primary_location_id,
            employer_id,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
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

    pub fn original_hire_date(&self) -> NaiveDate {
        self.original_hire_date
    }

    pub fn hire_date(&self) -> NaiveDate {
        self.hire_date
    }

    pub fn termination_date(&self) -> Option<NaiveDate> {
        self.termination_date
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
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

    pub fn department_name(&self) -> Option<&str> {
        self.department_name.as_deref()
    }

    pub fn primary_location_id(&self) -> Uuid {
        self.primary_location_id
    }

    pub fn employer_id(&self) -> Option<Uuid> {
        self.employer_id
    }

    #[cfg(test)]
    pub fn set_is_active(&mut self, is_active: bool) {
        self.is_active = is_active;
    }

    #[cfg(test)]
    pub fn set_termination_date(&mut self, termination_date: Option<NaiveDate>) {
        self.termination_date = termination_date;
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
}

impl AssignmentRuleConditions {
    pub fn new(
        department_id: Option<Uuid>,
        employee_id_numbers: Option<Vec<String>>,
        tags: Option<Vec<String>>,
    ) -> Self {
        Self {
            department_id,
            employee_id_numbers,
            tags,
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
}

impl AssignmentRule {
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
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct CreateAssignmentRuleRequest {
    pub pattern_id: Uuid,
    pub name: String,
    pub priority: i32,
    pub conditions: AssignmentRuleConditions,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct UpdateAssignmentRuleRequest {
    pub pattern_id: Option<Uuid>,
    pub name: Option<String>,
    pub priority: Option<i32>,
    pub conditions: Option<AssignmentRuleConditions>,
    pub is_active: Option<bool>,
    pub max_concurrent_workers: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct RehireEmployeeRequest {
    pub hire_date: NaiveDate,
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
    pub primary_location_id: Uuid,
    pub employer_id: Option<Uuid>,
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
    pub hire_date: Option<NaiveDate>,
    pub termination_date: Option<NaiveDate>,
    pub primary_location_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
    pub is_active: Option<bool>,
    pub max_concurrent_workers: Option<i32>,
    pub recruitment_source_id: Option<Uuid>,
    pub employer_id: Option<Uuid>,
    pub force_transfer: Option<bool>,
}

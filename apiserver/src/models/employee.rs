use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::custom_field::CustomFieldValueResponse;
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

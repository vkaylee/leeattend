use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::models::Employee;
use crate::models::{NewEmployeeDocumentParams, NewEmployeeProfileParams};
use crate::schema::employees;
use diesel::sql_types::{
    Array, BigInt, Bool, Nullable, Numeric, Text, Timestamptz, Uuid as UuidType,
};

#[derive(QueryableByName)]
pub struct CountRow {
    #[diesel(sql_type = BigInt)]
    pub count: i64,
}

#[derive(Debug, diesel::QueryableByName)]
pub struct EmployeeRow {
    #[diesel(sql_type = UuidType)]
    pub id: Uuid,
    #[diesel(sql_type = UuidType)]
    pub workspace_id: Uuid,
    #[diesel(sql_type = Nullable<UuidType>)]
    pub user_id: Option<Uuid>,
    #[diesel(sql_type = Text)]
    pub full_name: String,
    #[diesel(sql_type = Text)]
    pub employee_id_number: String,
    #[diesel(sql_type = Array<Text>)]
    pub tags: Vec<String>,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = Timestamptz)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = Nullable<Text>)]
    pub department_name: Option<String>,
    #[diesel(sql_type = Nullable<UuidType>)]
    pub primary_location_id: Option<Uuid>,
    #[diesel(sql_type = Nullable<UuidType>)]
    pub employer_id: Option<Uuid>,
    #[diesel(sql_type = Nullable<UuidType>)]
    pub active_employment_id: Option<Uuid>,
}

impl From<EmployeeRow> for Employee {
    fn from(row: EmployeeRow) -> Self {
        Self::new(
            row.id,
            row.workspace_id,
            row.user_id,
            row.full_name,
            row.employee_id_number,
            row.tags,
            row.created_at,
            row.updated_at,
            row.department_name,
        )
        .with_employment_info(
            row.active_employment_id,
            row.employer_id,
            row.primary_location_id,
        )
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = employees)]
pub struct EmployeeEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Option<Uuid>,
    pub employee_id_number: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub tags: Option<Vec<Option<String>>>,
    pub full_name: String,
    pub recruitment_source_id: Option<Uuid>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub hire_date: chrono::NaiveDate,
    pub original_hire_date: chrono::NaiveDate,
    pub termination_date: Option<chrono::NaiveDate>,
    pub is_active: bool,
}

impl From<Employee> for EmployeeEntity {
    fn from(val: Employee) -> Self {
        let hire = val.created_at().date_naive();
        EmployeeEntity {
            id: val.id(),
            workspace_id: val.workspace_id(),
            user_id: val.user_id(),
            employee_id_number: val.employee_id_number().to_string(),
            created_at: Some(val.created_at()),
            updated_at: Some(val.updated_at()),
            tags: Some(val.tags().to_vec().into_iter().map(Some).collect()),
            full_name: val.full_name().to_string(),
            recruitment_source_id: None,
            deleted_at: None,
            is_active: val.is_active(),
            hire_date: hire,
            termination_date: None,
            original_hire_date: hire,
        }
    }
}

impl From<EmployeeEntity> for Employee {
    fn from(val: EmployeeEntity) -> Self {
        Employee::new(
            val.id,
            val.workspace_id,
            val.user_id,
            val.full_name,
            val.employee_id_number,
            val.tags.unwrap_or_default().into_iter().flatten().collect(),
            val.created_at.unwrap_or_else(Utc::now),
            val.updated_at.unwrap_or_else(Utc::now),
            None,
        )
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = employees)]
pub struct UpdateDieselEmployee {
    pub user_id: Option<Option<Uuid>>,
    pub full_name: Option<String>,
    pub employee_id_number: Option<String>,
    pub tags: Option<Vec<Option<String>>>,
    pub updated_at: Option<DateTime<Utc>>,
}

use crate::models::{RecruitmentSource, RecruitmentSourceType};
use crate::schema::sql_types::RecruitmentSourceType as RecruitmentSourceTypeSql;
use crate::schema::{
    employee_documents, employee_emergency_contacts, employee_profiles, recruitment_sources,
};
use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, IsNull, Output, ToSql};
use rust_decimal::Decimal;
use std::io::Write;

impl ToSql<RecruitmentSourceTypeSql, Pg> for RecruitmentSourceType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            RecruitmentSourceType::Direct => out.write_all(b"direct")?,
            RecruitmentSourceType::Referral => out.write_all(b"referral")?,
            RecruitmentSourceType::Agency => out.write_all(b"agency")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<RecruitmentSourceTypeSql, Pg> for RecruitmentSourceType {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"direct" => Ok(RecruitmentSourceType::Direct),
            b"referral" => Ok(RecruitmentSourceType::Referral),
            b"agency" => Ok(RecruitmentSourceType::Agency),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = recruitment_sources)]
pub struct RecruitmentSourceEntity {
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

impl From<RecruitmentSource> for RecruitmentSourceEntity {
    fn from(val: RecruitmentSource) -> Self {
        Self {
            id: val.id(),
            workspace_id: val.workspace_id(),
            source_type: *val.source_type(),
            name: val.name().to_string(),
            referrer_employee_id: val.referrer_employee_id(),
            referral_bonus_amount: val.referral_bonus_amount(),
            referral_bonus_paid: val.referral_bonus_paid(),
            agency_contact_name: val.agency_contact_name().map(String::from),
            agency_contact_email: val.agency_contact_email().map(String::from),
            agency_fee_amount: val.agency_fee_amount(),
            agency_fee_paid: val.agency_fee_paid(),
            notes: val.notes().map(String::from),
            created_at: val.created_at(),
            updated_at: val.updated_at(),
        }
    }
}

impl From<RecruitmentSourceEntity> for RecruitmentSource {
    fn from(val: RecruitmentSourceEntity) -> Self {
        Self::new(
            val.id,
            val.workspace_id,
            val.source_type,
            val.name,
            val.referrer_employee_id,
            val.referral_bonus_amount,
            val.referral_bonus_paid,
            val.agency_contact_name,
            val.agency_contact_email,
            val.agency_fee_amount,
            val.agency_fee_paid,
            val.notes,
            val.created_at,
            val.updated_at,
        )
    }
}

#[derive(Debug, diesel::QueryableByName)]
pub struct RecruitmentSourceBreakdownRow {
    #[diesel(sql_type = Text)]
    pub source_type: String,
    #[diesel(sql_type = Text)]
    pub source_name: String,
    #[diesel(sql_type = BigInt)]
    pub count: i64,
}

#[derive(Debug, diesel::QueryableByName)]
pub struct TenureRow {
    #[diesel(sql_type = Text)]
    pub bucket: String,
    #[diesel(sql_type = BigInt)]
    pub count: i64,
}

#[derive(Debug, diesel::QueryableByName)]
pub struct DeptRow {
    #[diesel(sql_type = Text)]
    pub department_name: String,
    #[diesel(sql_type = BigInt)]
    pub count: i64,
}

#[derive(Debug, diesel::QueryableByName)]
pub struct ClassificationRow {
    #[diesel(sql_type = Text)]
    pub classification: String,
    #[diesel(sql_type = BigInt)]
    pub count: i64,
}

#[derive(Debug, diesel::QueryableByName)]
pub struct ReasonRow {
    #[diesel(sql_type = Text)]
    pub reason: String,
    #[diesel(sql_type = BigInt)]
    pub count: i64,
}

#[derive(Debug, diesel::QueryableByName)]
pub struct EmploymentTypeRow {
    #[diesel(sql_type = Text)]
    pub employment_type: String,
    #[diesel(sql_type = BigInt)]
    pub count: i64,
}

#[derive(Debug, diesel::QueryableByName)]
pub struct LocationRow {
    #[diesel(sql_type = Text)]
    pub location_name: String,
    #[diesel(sql_type = BigInt)]
    pub count: i64,
}

#[derive(Debug, diesel::QueryableByName)]
pub struct ManagerRow {
    #[diesel(sql_type = Text)]
    pub manager_name: String,
    #[diesel(sql_type = BigInt)]
    pub count: i64,
}

#[derive(Debug, diesel::QueryableByName)]
pub struct AvgHeadcountRow {
    #[diesel(sql_type = BigInt)]
    pub n_months: i64,
    #[diesel(sql_type = BigInt)]
    pub average_headcount: i64,
    #[diesel(sql_type = BigInt)]
    pub headcount_start: i64,
}

#[derive(Debug, diesel::QueryableByName)]
pub struct TrendRow {
    #[diesel(sql_type = Text)]
    pub month: String,
    #[diesel(sql_type = BigInt)]
    pub headcount: i64,
    #[diesel(sql_type = BigInt)]
    pub leavers: i64,
}

#[derive(Debug, diesel::QueryableByName)]
pub struct CohortRow {
    #[diesel(sql_type = Text)]
    pub cohort_month: String,
    #[diesel(sql_type = BigInt)]
    pub months_since_hire: i64,
    #[diesel(sql_type = BigInt)]
    pub cohort_size: i64,
    #[diesel(sql_type = BigInt)]
    pub active_count: i64,
}

/// Row for `recruitment_effectiveness_hires.sql` — per-source hires + cost.
#[derive(Debug, diesel::QueryableByName)]
pub struct EffectivenessHiresRow {
    #[diesel(sql_type = Nullable<UuidType>)]
    pub source_id: Option<Uuid>,
    #[diesel(sql_type = Text)]
    pub source_type: String,
    #[diesel(sql_type = Text)]
    pub source_name: String,
    #[diesel(sql_type = BigInt)]
    pub hires: i64,
    #[diesel(sql_type = Nullable<Numeric>)]
    pub total_cost: Option<rust_decimal::Decimal>,
    #[diesel(sql_type = Bool)]
    pub cost_available: bool,
}

/// Row for `recruitment_effectiveness_retention.sql` — per-source retained-at-90d.
#[derive(Debug, diesel::QueryableByName)]
pub struct EffectivenessRetentionRow {
    #[diesel(sql_type = Nullable<UuidType>)]
    pub source_id: Option<Uuid>,
    #[diesel(sql_type = Text)]
    pub source_type: String,
    #[diesel(sql_type = Text)]
    pub source_name: String,
    #[diesel(sql_type = BigInt)]
    pub retained_90d: i64,
}

/// Row for `recruitment_effectiveness_trend.sql` — monthly hires bucket.
#[derive(Debug, diesel::QueryableByName)]
pub struct EffectivenessTrendRow {
    #[diesel(sql_type = Text)]
    pub month: String,
    #[diesel(sql_type = BigInt)]
    pub hires: i64,
}

/// Row for `recruitment_effectiveness_turnover.sql` — per-source terminations in range.
#[derive(Debug, diesel::QueryableByName)]
pub struct EffectivenessTurnoverRow {
    #[diesel(sql_type = Nullable<UuidType>)]
    pub source_id: Option<Uuid>,
    #[diesel(sql_type = Text)]
    pub source_type: String,
    #[diesel(sql_type = Text)]
    pub source_name: String,
    #[diesel(sql_type = BigInt)]
    pub count: i64,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = employee_profiles)]
pub struct EmployeeProfileEntity {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<String>,
    pub national_id: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub bank_branch: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub avatar_file_id: Option<Uuid>,
}

impl From<EmployeeProfileEntity> for crate::models::EmployeeProfile {
    fn from(val: EmployeeProfileEntity) -> Self {
        crate::models::EmployeeProfile::new(NewEmployeeProfileParams {
            id: val.id,
            employee_id: val.employee_id,
            phone: val.phone,
            address: val.address,
            date_of_birth: val.date_of_birth,
            gender: val.gender,
            national_id: val.national_id,
            bank_name: val.bank_name,
            bank_account_number: val.bank_account_number,
            bank_branch: val.bank_branch,
            created_at: val.created_at,
            updated_at: val.updated_at,
        })
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = employee_emergency_contacts)]
pub struct EmployeeEmergencyContactEntity {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub contact_name: String,
    pub relationship: String,
    pub phone: String,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
}

impl From<EmployeeEmergencyContactEntity> for crate::models::EmployeeEmergencyContact {
    fn from(val: EmployeeEmergencyContactEntity) -> Self {
        crate::models::EmployeeEmergencyContact::new(
            val.id,
            val.employee_id,
            val.contact_name,
            val.relationship,
            val.phone,
            val.is_primary,
            val.created_at,
        )
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = employee_documents)]
pub struct EmployeeDocumentEntity {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub workspace_id: Uuid,
    pub document_type: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i32,
    pub mime_type: String,
    pub expires_at: Option<NaiveDate>,
    pub is_active: bool,
    pub uploaded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl From<EmployeeDocumentEntity> for crate::models::EmployeeDocument {
    fn from(val: EmployeeDocumentEntity) -> Self {
        crate::models::EmployeeDocument::new(NewEmployeeDocumentParams {
            id: val.id,
            employee_id: val.employee_id,
            workspace_id: val.workspace_id,
            document_type: val.document_type,
            file_name: val.file_name,
            file_path: val.file_path,
            file_size: val.file_size,
            mime_type: val.mime_type,
            expires_at: val.expires_at,
            is_active: val.is_active,
            uploaded_by: val.uploaded_by,
            created_at: val.created_at,
        })
    }
}

impl From<crate::models::EmployeeDocument> for EmployeeDocumentEntity {
    fn from(val: crate::models::EmployeeDocument) -> Self {
        Self {
            id: val.id(),
            employee_id: val.employee_id(),
            workspace_id: val.workspace_id(),
            document_type: val.document_type().to_string(),
            file_name: val.file_name().to_string(),
            file_path: val.file_path().to_string(),
            file_size: val.file_size(),
            mime_type: val.mime_type().to_string(),
            expires_at: val.expires_at(),
            is_active: val.is_active(),
            uploaded_by: val.uploaded_by(),
            created_at: val.created_at(),
        }
    }
}

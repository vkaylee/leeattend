use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::models::Employee;
use crate::schema::employees;
use diesel::sql_types::{Array, BigInt, Bool, Date, Nullable, Text, Timestamptz, Uuid as UuidType};

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
    #[diesel(sql_type = Date)]
    pub original_hire_date: chrono::NaiveDate,
    #[diesel(sql_type = Date)]
    pub hire_date: chrono::NaiveDate,
    #[diesel(sql_type = Nullable<Date>)]
    pub termination_date: Option<chrono::NaiveDate>,
    #[diesel(sql_type = Array<Text>)]
    pub tags: Vec<String>,
    #[diesel(sql_type = Bool)]
    pub is_active: bool,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = Timestamptz)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = Nullable<Text>)]
    pub department_name: Option<String>,
    #[diesel(sql_type = UuidType)]
    pub primary_location_id: Uuid,
}

impl From<EmployeeRow> for Employee {
    fn from(row: EmployeeRow) -> Self {
        Self {
            id: row.id,
            workspace_id: row.workspace_id,
            user_id: row.user_id,
            full_name: row.full_name,
            employee_id_number: row.employee_id_number,
            original_hire_date: row.original_hire_date,
            hire_date: row.hire_date,
            termination_date: row.termination_date,
            tags: row.tags,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
            department_name: row.department_name,
            primary_location_id: row.primary_location_id,
        }
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = employees)]
pub struct EmployeeEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Option<Uuid>,
    pub employee_id_number: String,
    pub is_active: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub tags: Option<Vec<Option<String>>>,
    pub full_name: String,
    pub recruitment_source_id: Option<Uuid>,
    pub hire_date: NaiveDate,
    pub termination_date: Option<NaiveDate>,
    pub original_hire_date: NaiveDate,
    pub primary_location_id: Uuid,
}

impl From<Employee> for EmployeeEntity {
    fn from(val: Employee) -> Self {
        EmployeeEntity {
            id: val.id,
            workspace_id: val.workspace_id,
            user_id: val.user_id,
            employee_id_number: val.employee_id_number,
            is_active: val.is_active,
            created_at: Some(val.created_at),
            updated_at: Some(val.updated_at),
            tags: Some(val.tags.into_iter().map(Some).collect()),
            full_name: val.full_name,
            recruitment_source_id: None,
            hire_date: val.hire_date,
            termination_date: val.termination_date,
            original_hire_date: val.original_hire_date,
            primary_location_id: val.primary_location_id,
        }
    }
}

impl From<EmployeeEntity> for Employee {
    fn from(val: EmployeeEntity) -> Self {
        Employee {
            id: val.id,
            workspace_id: val.workspace_id,
            user_id: val.user_id,
            full_name: val.full_name,
            employee_id_number: val.employee_id_number,
            original_hire_date: val.original_hire_date,
            hire_date: val.hire_date,
            termination_date: val.termination_date,
            tags: val.tags.unwrap_or_default().into_iter().flatten().collect(),
            is_active: val.is_active,
            created_at: val.created_at.unwrap_or_else(Utc::now),
            updated_at: val.updated_at.unwrap_or_else(Utc::now),
            department_name: None,
            primary_location_id: val.primary_location_id,
        }
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = employees)]
pub struct UpdateDieselEmployee {
    pub user_id: Option<Uuid>,
    pub full_name: Option<String>,
    pub employee_id_number: Option<String>,
    pub tags: Option<Vec<Option<String>>>,
    pub is_active: Option<bool>,
    pub hire_date: Option<NaiveDate>,
    pub termination_date: Option<Option<NaiveDate>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub primary_location_id: Option<Uuid>,
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
            id: val.id,
            workspace_id: val.workspace_id,
            source_type: val.source_type,
            name: val.name,
            referrer_employee_id: val.referrer_employee_id,
            referral_bonus_amount: val.referral_bonus_amount,
            referral_bonus_paid: val.referral_bonus_paid,
            agency_contact_name: val.agency_contact_name,
            agency_contact_email: val.agency_contact_email,
            agency_fee_amount: val.agency_fee_amount,
            agency_fee_paid: val.agency_fee_paid,
            notes: val.notes,
            created_at: val.created_at,
            updated_at: val.updated_at,
        }
    }
}

impl From<RecruitmentSourceEntity> for RecruitmentSource {
    fn from(val: RecruitmentSourceEntity) -> Self {
        Self {
            id: val.id,
            workspace_id: val.workspace_id,
            source_type: val.source_type,
            name: val.name,
            referrer_employee_id: val.referrer_employee_id,
            referral_bonus_amount: val.referral_bonus_amount,
            referral_bonus_paid: val.referral_bonus_paid,
            agency_contact_name: val.agency_contact_name,
            agency_contact_email: val.agency_contact_email,
            agency_fee_amount: val.agency_fee_amount,
            agency_fee_paid: val.agency_fee_paid,
            notes: val.notes,
            created_at: val.created_at,
            updated_at: val.updated_at,
        }
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
}

impl From<EmployeeProfileEntity> for crate::models::EmployeeProfile {
    fn from(val: EmployeeProfileEntity) -> Self {
        Self {
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
        }
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
        Self {
            id: val.id,
            employee_id: val.employee_id,
            contact_name: val.contact_name,
            relationship: val.relationship,
            phone: val.phone,
            is_primary: val.is_primary,
            created_at: val.created_at,
        }
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
        Self {
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
        }
    }
}

impl From<crate::models::EmployeeDocument> for EmployeeDocumentEntity {
    fn from(val: crate::models::EmployeeDocument) -> Self {
        Self {
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
        }
    }
}

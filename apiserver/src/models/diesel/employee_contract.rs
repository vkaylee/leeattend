use chrono::{DateTime, NaiveDate, Utc};
use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::prelude::*;
use diesel::serialize::{self, IsNull, Output, ToSql};
use std::io::Write;
use uuid::Uuid;

use crate::models::contract::EmployeeContract;
use crate::models::enums::{ContractStatus, ContractType};
use crate::schema::employee_contracts;
use crate::schema::sql_types::{ContractStatusEnum, ContractTypeEnum};

impl ToSql<ContractTypeEnum, Pg> for ContractType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            ContractType::Probation => out.write_all(b"probation")?,
            ContractType::Training => out.write_all(b"training")?,
            ContractType::FixedTerm => out.write_all(b"fixed_term")?,
            ContractType::Indefinite => out.write_all(b"indefinite")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<ContractTypeEnum, Pg> for ContractType {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"probation" => Ok(ContractType::Probation),
            b"training" => Ok(ContractType::Training),
            b"fixed_term" => Ok(ContractType::FixedTerm),
            b"indefinite" => Ok(ContractType::Indefinite),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl ToSql<ContractStatusEnum, Pg> for ContractStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            ContractStatus::Draft => out.write_all(b"draft")?,
            ContractStatus::Active => out.write_all(b"active")?,
            ContractStatus::Expired => out.write_all(b"expired")?,
            ContractStatus::Terminated => out.write_all(b"terminated")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<ContractStatusEnum, Pg> for ContractStatus {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"draft" => Ok(ContractStatus::Draft),
            b"active" => Ok(ContractStatus::Active),
            b"expired" => Ok(ContractStatus::Expired),
            b"terminated" => Ok(ContractStatus::Terminated),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = employee_contracts)]
pub struct EmployeeContractEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub contract_number: Option<String>,
    pub contract_type: ContractType,
    pub status: ContractStatus,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub document_id: Option<Uuid>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub employment_id: Uuid,
}

impl From<EmployeeContractEntity> for EmployeeContract {
    fn from(val: EmployeeContractEntity) -> Self {
        EmployeeContract::new(
            val.id,
            val.workspace_id,
            val.employment_id,
            val.contract_number,
            val.contract_type,
            val.status,
            val.start_date,
            val.end_date,
            val.document_id,
            val.notes,
            val.created_at,
            val.updated_at,
        )
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = employee_contracts)]
pub struct UpdateDieselEmployeeContract {
    pub contract_number: Option<String>,
    pub contract_type: Option<ContractType>,
    pub status: Option<ContractStatus>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<Option<NaiveDate>>,
    pub document_id: Option<Option<Uuid>>,
    pub notes: Option<Option<String>>,
    pub updated_at: Option<DateTime<Utc>>,
}

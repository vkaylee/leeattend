use crate::schema::employee_assignments;
use chrono::{NaiveDate, DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use std::io::Write;
use crate::schema::sql_types::AssignmentStatus;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    strum_macros::Display,
    strum_macros::EnumString,
    Serialize,
    Deserialize,
    utoipa::ToSchema,
    diesel::expression::AsExpression,
    diesel::deserialize::FromSqlRow,
)]
#[diesel(sql_type = AssignmentStatus)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AssignmentStatusEnum {
    Active,
    Completed,
    Cancelled,
}

impl ToSql<AssignmentStatus, Pg> for AssignmentStatusEnum {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            AssignmentStatusEnum::Active => out.write_all(b"active")?,
            AssignmentStatusEnum::Completed => out.write_all(b"completed")?,
            AssignmentStatusEnum::Cancelled => out.write_all(b"cancelled")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<AssignmentStatus, Pg> for AssignmentStatusEnum {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"active" => Ok(AssignmentStatusEnum::Active),
            b"completed" => Ok(AssignmentStatusEnum::Completed),
            b"cancelled" => Ok(AssignmentStatusEnum::Cancelled),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = employee_assignments)]
pub struct EmployeeAssignment {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employee_id: Uuid,
    pub home_location_id: Uuid,
    pub host_location_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub reason: Option<String>,
    pub status: AssignmentStatusEnum,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = employee_assignments)]
pub struct NewEmployeeAssignment {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employee_id: Uuid,
    pub home_location_id: Uuid,
    pub host_location_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub reason: Option<String>,
    pub status: AssignmentStatusEnum,
}

use chrono::{DateTime, Utc};
use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::prelude::*;
use diesel::serialize::{self, IsNull, Output, ToSql};
use std::io::Write;
use uuid::Uuid;

use crate::models::{AdjustmentStatus, AttendanceAdjustmentRequest};
use crate::schema::attendance_adjustment_requests;
use crate::schema::sql_types::AdjustmentStatusType;

impl ToSql<AdjustmentStatusType, Pg> for AdjustmentStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            AdjustmentStatus::Pending => out.write_all(b"pending")?,
            AdjustmentStatus::Approved => out.write_all(b"approved")?,
            AdjustmentStatus::Rejected => out.write_all(b"rejected")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<AdjustmentStatusType, Pg> for AdjustmentStatus {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"pending" => Ok(AdjustmentStatus::Pending),
            b"approved" => Ok(AdjustmentStatus::Approved),
            b"rejected" => Ok(AdjustmentStatus::Rejected),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = attendance_adjustment_requests)]
pub struct AttendanceAdjustmentEntity {
    pub id: Uuid,
    pub time_entry_id: Option<Uuid>,
    pub requested_punch_in_time: DateTime<Utc>,
    pub requested_punch_out_time: Option<DateTime<Utc>>,
    pub reason: Option<String>,
    pub status: AdjustmentStatus,
    pub requested_by_user_id: Option<Uuid>,
    pub approved_by_user_id: Option<Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_break_overtime: bool,
    pub employment_id: Uuid,
    pub attachment_file_ids: Option<Vec<Option<Uuid>>>,
}

impl From<AttendanceAdjustmentEntity> for AttendanceAdjustmentRequest {
    fn from(val: AttendanceAdjustmentEntity) -> Self {
        AttendanceAdjustmentRequest::new(
            val.id,
            val.employment_id,
            val.time_entry_id,
            val.requested_punch_in_time,
            val.requested_punch_out_time,
            val.is_break_overtime,
            val.reason,
            val.status,
            val.requested_by_user_id,
            val.approved_by_user_id,
            val.created_at.unwrap_or_else(Utc::now),
            val.updated_at.unwrap_or_else(Utc::now),
            val.attachment_file_ids,
        )
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = attendance_adjustment_requests)]
pub struct NewAttendanceAdjustmentEntity {
    pub id: Uuid,
    pub employment_id: Uuid,
    pub time_entry_id: Option<Uuid>,
    pub requested_punch_in_time: DateTime<Utc>,
    pub requested_punch_out_time: Option<DateTime<Utc>>,
    pub is_break_overtime: bool,
    pub reason: Option<String>,
    pub status: AdjustmentStatus,
    pub requested_by_user_id: Option<Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub attachment_file_ids: Option<Vec<Option<Uuid>>>,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = attendance_adjustment_requests)]
pub struct UpdateAttendanceAdjustmentEntity {
    pub status: Option<AdjustmentStatus>,
    pub approved_by_user_id: Option<Option<Uuid>>,
    pub updated_at: Option<DateTime<Utc>>,
}

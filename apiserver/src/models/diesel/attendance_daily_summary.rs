use chrono::{DateTime, NaiveDate, Utc};
use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::prelude::*;
use diesel::serialize::{self, IsNull, Output, ToSql};
use std::io::Write;
use uuid::Uuid;

use crate::models::{AttendanceDailySummary, OtStatus};
use crate::schema::attendance_daily_summaries;
use crate::schema::sql_types::OtStatusType;

impl ToSql<OtStatusType, Pg> for OtStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            OtStatus::None => out.write_all(b"none")?,
            OtStatus::PendingEmployee => out.write_all(b"pending_employee")?,
            OtStatus::PendingManager => out.write_all(b"pending_manager")?,
            OtStatus::Approved => out.write_all(b"approved")?,
            OtStatus::Rejected => out.write_all(b"rejected")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<OtStatusType, Pg> for OtStatus {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"none" => Ok(OtStatus::None),
            b"pending_employee" => Ok(OtStatus::PendingEmployee),
            b"pending_manager" => Ok(OtStatus::PendingManager),
            b"approved" => Ok(OtStatus::Approved),
            b"rejected" => Ok(OtStatus::Rejected),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = attendance_daily_summaries)]
pub struct AttendanceDailySummaryEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub date: NaiveDate,
    pub actual_work_minutes: i32,
    pub system_detected_ot_minutes: i32,
    pub claimed_ot_minutes: i32,
    pub approved_ot_minutes: i32,
    pub ot_status: OtStatus,
    pub ot_reason: Option<String>,
    pub manager_note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub claim_entered_by_user_id: Option<Uuid>,
    pub claim_entered_at: Option<DateTime<Utc>>,
    pub paper_form_photo_url: Option<String>,
    pub entered_via: Option<String>,
    pub is_holiday: bool,
    pub holiday_name: Option<String>,
    pub employment_id: Uuid,
    pub policy_version_id: Option<Uuid>,
    pub effective_rules_snapshot: Option<serde_json::Value>,
    pub is_superseded: bool,
    pub superseded_by_id: Option<Uuid>,
    pub superseded_at: Option<DateTime<Utc>>,
}

impl From<AttendanceDailySummaryEntity> for AttendanceDailySummary {
    fn from(val: AttendanceDailySummaryEntity) -> Self {
        AttendanceDailySummary {
            id: val.id,
            workspace_id: val.workspace_id,
            employment_id: val.employment_id,
            date: val.date,
            actual_work_minutes: val.actual_work_minutes,
            system_detected_ot_minutes: val.system_detected_ot_minutes,
            claimed_ot_minutes: val.claimed_ot_minutes,
            approved_ot_minutes: val.approved_ot_minutes,
            ot_status: val.ot_status,
            ot_reason: val.ot_reason,
            manager_note: val.manager_note,
            claim_entered_by_user_id: val.claim_entered_by_user_id,
            claim_entered_at: val.claim_entered_at,
            paper_form_photo_url: val.paper_form_photo_url,
            entered_via: val.entered_via,
            is_holiday: val.is_holiday,
            holiday_name: val.holiday_name,
            created_at: val.created_at,
            updated_at: val.updated_at,
            policy_version_id: val.policy_version_id,
            effective_rules_snapshot: val.effective_rules_snapshot,
            is_superseded: val.is_superseded,
            superseded_by_id: val.superseded_by_id,
            superseded_at: val.superseded_at,
        }
    }
}

use crate::models::enums::OtRequestStatus;
use crate::schema::{overtime_request_employees, overtime_requests};
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use diesel::{
    deserialize::{self, FromSql},
    pg::{Pg, PgValue},
    prelude::*,
    serialize::{self, IsNull, Output, ToSql},
};
use serde::{Deserialize, Serialize};
use std::io::Write;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, ToSchema)]
#[diesel(table_name = overtime_requests)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OvertimeRequest {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub requested_by_user_id: Uuid,
    pub department_id: Option<Uuid>,
    pub planned_date: NaiveDate,
    pub planned_start_time: Option<NaiveTime>,
    pub planned_end_time: Option<NaiveTime>,
    pub planned_minutes: i32,
    pub reason: String,
    pub status: OtRequestStatus,
    pub workflow_id: Option<Uuid>,
    pub current_step: i32,
    pub quota_warning: Option<serde_json::Value>,
    pub reconciled_at: Option<DateTime<Utc>>,
    pub reconciled_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, ToSchema)]
#[diesel(table_name = overtime_request_employees)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OvertimeRequestEmployee {
    pub id: Uuid,
    pub overtime_request_id: Uuid,
    pub employee_id: Uuid,
    pub consent_status: String,
    pub consent_at: Option<DateTime<Utc>>,
    pub decline_reason: Option<String>,
    pub actual_ot_minutes: Option<i32>,
    pub matched_summary_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl ToSql<crate::schema::sql_types::OtRequestStatusType, Pg> for OtRequestStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let s = match *self {
            OtRequestStatus::Draft => "draft",
            OtRequestStatus::PendingApproval => "pending_approval",
            OtRequestStatus::Approved => "approved",
            OtRequestStatus::PartiallyAccepted => "partially_accepted",
            OtRequestStatus::Active => "active",
            OtRequestStatus::Completed => "completed",
            OtRequestStatus::Cancelled => "cancelled",
            OtRequestStatus::Rejected => "rejected",
        };
        out.write_all(s.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::OtRequestStatusType, Pg> for OtRequestStatus {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"draft" => Ok(OtRequestStatus::Draft),
            b"pending_approval" => Ok(OtRequestStatus::PendingApproval),
            b"approved" => Ok(OtRequestStatus::Approved),
            b"partially_accepted" => Ok(OtRequestStatus::PartiallyAccepted),
            b"active" => Ok(OtRequestStatus::Active),
            b"completed" => Ok(OtRequestStatus::Completed),
            b"cancelled" => Ok(OtRequestStatus::Cancelled),
            b"rejected" => Ok(OtRequestStatus::Rejected),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

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
    id: Uuid,
    workspace_id: Uuid,
    requested_by_user_id: Uuid,
    department_id: Option<Uuid>,
    planned_date: NaiveDate,
    planned_start_time: Option<NaiveTime>,
    planned_end_time: Option<NaiveTime>,
    planned_minutes: i32,
    reason: String,
    status: OtRequestStatus,
    workflow_id: Option<Uuid>,
    current_step: i32,
    quota_warning: Option<serde_json::Value>,
    reconciled_at: Option<DateTime<Utc>>,
    reconciled_by: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl OvertimeRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        requested_by_user_id: Uuid,
        department_id: Option<Uuid>,
        planned_date: NaiveDate,
        planned_start_time: Option<NaiveTime>,
        planned_end_time: Option<NaiveTime>,
        planned_minutes: i32,
        reason: String,
        status: OtRequestStatus,
        workflow_id: Option<Uuid>,
        current_step: i32,
        quota_warning: Option<serde_json::Value>,
        reconciled_at: Option<DateTime<Utc>>,
        reconciled_by: Option<Uuid>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            requested_by_user_id,
            department_id,
            planned_date,
            planned_start_time,
            planned_end_time,
            planned_minutes,
            reason,
            status,
            workflow_id,
            current_step,
            quota_warning,
            reconciled_at,
            reconciled_by,
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

    pub fn set_workspace_id(&mut self, val: Uuid) {
        self.workspace_id = val;
    }

    pub fn requested_by_user_id(&self) -> Uuid {
        self.requested_by_user_id
    }

    pub fn department_id(&self) -> Option<Uuid> {
        self.department_id
    }

    pub fn planned_date(&self) -> NaiveDate {
        self.planned_date
    }

    pub fn planned_start_time(&self) -> Option<NaiveTime> {
        self.planned_start_time
    }

    pub fn planned_end_time(&self) -> Option<NaiveTime> {
        self.planned_end_time
    }

    pub fn planned_minutes(&self) -> i32 {
        self.planned_minutes
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }

    pub fn status(&self) -> OtRequestStatus {
        self.status
    }

    pub fn set_status(&mut self, val: OtRequestStatus) {
        self.status = val;
    }

    pub fn workflow_id(&self) -> Option<Uuid> {
        self.workflow_id
    }

    pub fn current_step(&self) -> i32 {
        self.current_step
    }

    pub fn set_current_step(&mut self, val: i32) {
        self.current_step = val;
    }

    pub fn quota_warning(&self) -> Option<&serde_json::Value> {
        self.quota_warning.as_ref()
    }

    pub fn reconciled_at(&self) -> Option<DateTime<Utc>> {
        self.reconciled_at
    }

    pub fn reconciled_by(&self) -> Option<Uuid> {
        self.reconciled_by
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, ToSchema)]
#[diesel(table_name = overtime_request_employees)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OvertimeRequestEmployee {
    id: Uuid,
    overtime_request_id: Uuid,
    consent_status: String,
    consent_at: Option<DateTime<Utc>>,
    decline_reason: Option<String>,
    actual_ot_minutes: Option<i32>,
    matched_summary_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    employment_id: Uuid,
}

impl OvertimeRequestEmployee {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        overtime_request_id: Uuid,
        employment_id: Uuid,
        consent_status: String,
        consent_at: Option<DateTime<Utc>>,
        decline_reason: Option<String>,
        actual_ot_minutes: Option<i32>,
        matched_summary_id: Option<Uuid>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            overtime_request_id,
            consent_status,
            consent_at,
            decline_reason,
            actual_ot_minutes,
            matched_summary_id,
            created_at,
            employment_id,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn overtime_request_id(&self) -> Uuid {
        self.overtime_request_id
    }

    pub fn set_overtime_request_id(&mut self, val: Uuid) {
        self.overtime_request_id = val;
    }

    pub fn employment_id(&self) -> Uuid {
        self.employment_id
    }

    pub fn consent_status(&self) -> &str {
        &self.consent_status
    }

    pub fn consent_at(&self) -> Option<DateTime<Utc>> {
        self.consent_at
    }

    pub fn decline_reason(&self) -> Option<&str> {
        self.decline_reason.as_deref()
    }

    pub fn actual_ot_minutes(&self) -> Option<i32> {
        self.actual_ot_minutes
    }

    pub fn matched_summary_id(&self) -> Option<Uuid> {
        self.matched_summary_id
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
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

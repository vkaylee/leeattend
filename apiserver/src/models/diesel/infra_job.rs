use crate::schema::{infra_job_events, infra_job_records};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = infra_job_records)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InfraJobRecord {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub parent_job_id: Option<Uuid>,
    pub task_type: String,
    pub priority: String,
    pub state: String,
    pub attempt_count: i32,
    pub lease_owner: Option<String>,
    pub lease_token: Option<Uuid>,
    pub lease_expires_at: Option<DateTime<Utc>>,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub request_id: Option<Uuid>,
    pub trace_id: Option<String>,
    pub payload_hash: Option<String>,
    pub error_code: Option<String>,
    pub error_summary: Option<String>,
    pub created_at: DateTime<Utc>,
    pub queued_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = infra_job_records)]
pub struct NewInfraJobRecord {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub parent_job_id: Option<Uuid>,
    pub task_type: String,
    pub priority: String,
    pub state: String,
    pub attempt_count: i32,
    pub lease_owner: Option<String>,
    pub lease_token: Option<Uuid>,
    pub lease_expires_at: Option<DateTime<Utc>>,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub request_id: Option<Uuid>,
    pub trace_id: Option<String>,
    pub payload_hash: Option<String>,
    pub error_code: Option<String>,
    pub error_summary: Option<String>,
    pub created_at: DateTime<Utc>,
    pub queued_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = infra_job_events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InfraJobEvent {
    pub id: Uuid,
    pub job_id: Uuid,
    pub workspace_id: Uuid,
    pub event_type: String,
    pub from_state: Option<String>,
    pub to_state: String,
    pub attempt_count: i32,
    pub lease_token: Option<Uuid>,
    pub worker_id: Option<String>,
    pub request_id: Option<Uuid>,
    pub trace_id: Option<String>,
    pub reason_code: Option<String>,
    pub reason_summary: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = infra_job_events)]
pub struct NewInfraJobEvent {
    pub id: Uuid,
    pub job_id: Uuid,
    pub workspace_id: Uuid,
    pub event_type: String,
    pub from_state: Option<String>,
    pub to_state: String,
    pub attempt_count: i32,
    pub lease_token: Option<Uuid>,
    pub worker_id: Option<String>,
    pub request_id: Option<Uuid>,
    pub trace_id: Option<String>,
    pub reason_code: Option<String>,
    pub reason_summary: Option<String>,
    pub created_at: DateTime<Utc>,
}

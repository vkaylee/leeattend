use crate::errors::AppError;
use crate::models::diesel::infra_job::{InfraJobEvent, InfraJobRecord};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CreateInfraJobParams {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub parent_job_id: Option<Uuid>,
    pub task_type: String,
    pub priority: String,
    pub payload_hash: Option<String>,
    pub request_id: Option<Uuid>,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TransitionInfraJobParams {
    pub job_id: Uuid,
    pub workspace_id: Uuid,
    pub expected_state: Option<String>,
    pub to_state: String,
    pub event_type: String,
    pub lease_token: Option<Uuid>,
    pub lease_owner: Option<String>,
    pub lease_expires_at: Option<DateTime<Utc>>,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub worker_id: Option<String>,
    pub request_id: Option<Uuid>,
    pub trace_id: Option<String>,
    pub reason_code: Option<String>,
    pub reason_summary: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InfraJobCursor {
    pub created_at: DateTime<Utc>,
    pub id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct ListInfraJobsParams {
    pub cursor: Option<InfraJobCursor>,
    pub limit: i64,
    pub state: Option<String>,
    pub task_type: Option<String>,
    pub job_id: Option<Uuid>,
    pub search: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InfraJobPage {
    pub jobs: Vec<InfraJobRecord>,
    pub next_cursor: Option<InfraJobCursor>,
}

#[async_trait]
pub trait InfraJobRepository: Send + Sync {
    async fn create(&self, params: CreateInfraJobParams) -> Result<InfraJobRecord, AppError>;

    async fn transition(
        &self,
        params: TransitionInfraJobParams,
    ) -> Result<InfraJobRecord, AppError>;

    async fn get(
        &self,
        workspace_id: Uuid,
        job_id: Uuid,
    ) -> Result<Option<InfraJobRecord>, AppError>;

    async fn list_global(&self, params: ListInfraJobsParams) -> Result<InfraJobPage, AppError>;

    async fn list_events(
        &self,
        workspace_id: Uuid,
        job_id: Uuid,
        limit: i64,
    ) -> Result<Vec<InfraJobEvent>, AppError>;

    async fn list_events_bulk(
        &self,
        workspace_id: Option<Uuid>,
        job_ids: &[Uuid],
        total_limit: i64,
    ) -> Result<Vec<InfraJobEvent>, AppError>;
}

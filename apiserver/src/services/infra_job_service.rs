use crate::errors::AppError;
use crate::models::diesel::infra_job::{InfraJobEvent, InfraJobRecord};
use crate::queue::{Job, JobQueue};
use crate::repositories::infra_job_repository::{
    CreateInfraJobParams, InfraJobCursor, InfraJobPage, InfraJobRepository, ListInfraJobsParams,
    TransitionInfraJobParams,
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct InfraJobService {
    repository: Arc<dyn InfraJobRepository>,
}

impl InfraJobService {
    pub fn new(repository: Arc<dyn InfraJobRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_before_dispatch(
        &self,
        params: CreateInfraJobParams,
    ) -> Result<InfraJobRecord, AppError> {
        let job_id = params.id;
        let job = self.repository.create(params).await?;
        tracing::info!(job_id = %job_id, "infra_job_created");
        Ok(job)
    }

    /// Persist a job before publishing it to Redis, then record the queue
    /// acknowledgement. A dispatch failure is made explicit as `lost` so it
    /// cannot be mistaken for an ordinary empty queue.
    pub async fn enqueue_durable(&self, queue: &JobQueue, job: Job) -> Result<(), AppError> {
        let request_id = job
            .trace_context
            .as_deref()
            .and_then(|value| Uuid::parse_str(value).ok());
        let trace_id = job.trace_context.clone();
        let task_type = enum_name(&job.task_type)?;
        let priority = enum_name(&job.priority)?;
        let job_id = job.job_id;
        let workspace_id = job.workspace_id;

        self.create_before_dispatch(CreateInfraJobParams {
            id: job_id,
            workspace_id,
            parent_job_id: Some(job.parent_job_id),
            task_type,
            priority,
            payload_hash: None,
            request_id,
            trace_id: trace_id.clone(),
        })
        .await?;

        if let Err(error) = queue.enqueue_single(job).await {
            let reason_summary = "Redis dispatch failed".to_string();
            tracing::error!(
                job_id = %job_id,
                workspace_id = %workspace_id,
                error = %error,
                "infra_job_dispatch_failed"
            );
            if let Err(transition_error) = self
                .transition(TransitionInfraJobParams {
                    job_id,
                    workspace_id,
                    expected_state: Some("dispatching".to_string()),
                    to_state: "lost".to_string(),
                    event_type: "dispatch_failed".to_string(),
                    lease_token: None,
                    lease_owner: None,
                    lease_expires_at: None,
                    next_attempt_at: None,
                    worker_id: None,
                    request_id,
                    trace_id,
                    reason_code: Some("queue_dispatch_failed".to_string()),
                    reason_summary: Some(reason_summary.clone()),
                })
                .await
            {
                tracing::error!(
                    job_id = %job_id,
                    workspace_id = %workspace_id,
                    error = %transition_error,
                    "infra_job_dispatch_failure_transition_failed"
                );
            }
            return Err(AppError::Internal(reason_summary));
        }

        self.mark_queued(job_id, workspace_id, request_id, trace_id)
            .await?;
        Ok(())
    }

    pub async fn mark_queued(
        &self,
        job_id: Uuid,
        workspace_id: Uuid,
        request_id: Option<Uuid>,
        trace_id: Option<String>,
    ) -> Result<InfraJobRecord, AppError> {
        self.repository
            .transition(TransitionInfraJobParams {
                job_id,
                workspace_id,
                expected_state: Some("dispatching".to_string()),
                to_state: "queued".to_string(),
                event_type: "queued".to_string(),
                lease_token: None,
                lease_owner: None,
                lease_expires_at: None,
                next_attempt_at: None,
                worker_id: None,
                request_id,
                trace_id,
                reason_code: None,
                reason_summary: None,
            })
            .await
    }

    pub async fn transition(
        &self,
        params: TransitionInfraJobParams,
    ) -> Result<InfraJobRecord, AppError> {
        self.repository.transition(params).await
    }

    pub async fn get(
        &self,
        workspace_id: Uuid,
        job_id: Uuid,
    ) -> Result<Option<InfraJobRecord>, AppError> {
        self.repository.get(workspace_id, job_id).await
    }

    pub async fn list_global(&self, params: ListInfraJobsParams) -> Result<InfraJobPage, AppError> {
        self.repository.list_global(params).await
    }

    pub async fn list_events(
        &self,
        workspace_id: Uuid,
        job_id: Uuid,
        limit: i64,
    ) -> Result<Vec<InfraJobEvent>, AppError> {
        self.repository
            .list_events(workspace_id, job_id, limit)
            .await
    }

    pub async fn list_events_bulk(
        &self,
        workspace_id: Option<Uuid>,
        job_ids: &[Uuid],
        total_limit: i64,
    ) -> Result<Vec<InfraJobEvent>, AppError> {
        self.repository
            .list_events_bulk(workspace_id, job_ids, total_limit)
            .await
    }

    pub fn cursor_token(cursor: &InfraJobCursor) -> String {
        format!("{}:{}", cursor.created_at.timestamp_micros(), cursor.id)
    }

    pub fn parse_cursor_token(token: &str) -> Result<InfraJobCursor, AppError> {
        let (timestamp, id) = token
            .split_once(':')
            .ok_or_else(|| AppError::validation(crate::utils::i18n::I18nKey::Validation))?;
        let micros = timestamp.parse::<i64>().map_err(|_parse_error| {
            AppError::validation(crate::utils::i18n::I18nKey::Validation)
        })?;
        let created_at = chrono::DateTime::<chrono::Utc>::from_timestamp_micros(micros)
            .ok_or_else(|| AppError::validation(crate::utils::i18n::I18nKey::Validation))?;
        let id = Uuid::parse_str(id).map_err(|_parse_error| {
            AppError::validation(crate::utils::i18n::I18nKey::Validation)
        })?;
        Ok(InfraJobCursor { created_at, id })
    }
}

fn enum_name<T: serde::Serialize>(value: &T) -> Result<String, AppError> {
    let serialized = serde_json::to_value(value)
        .map_err(|error| AppError::Internal(format!("Failed to serialize job enum: {error}")))?;
    serialized
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| AppError::Internal("Job enum serialized to a non-string value".to_string()))
}

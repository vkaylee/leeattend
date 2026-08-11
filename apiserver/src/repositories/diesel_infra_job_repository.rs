use crate::diesel_pool::{DieselPools, TransactionConn};
use crate::errors::AppError;
use crate::models::diesel::infra_job::{
    InfraJobEvent, InfraJobRecord, NewInfraJobEvent, NewInfraJobRecord,
};
use crate::repositories::infra_job_repository::{
    CreateInfraJobParams, InfraJobCursor, InfraJobMonitorData, InfraJobPage, InfraJobRepository,
    ListInfraJobsParams, TransitionInfraJobParams,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

#[derive(Clone)]
pub struct DieselInfraJobRepository {
    pool: DieselPools,
}

impl DieselInfraJobRepository {
    pub fn new(pool: DieselPools) -> Self {
        Self { pool }
    }

    async fn api_conn(&self) -> Result<TransactionConn, AppError> {
        Ok(TransactionConn(self.pool.api.get().await.map_err(|e| {
            AppError::Internal(format!("Failed to get system DB connection: {e}"))
        })?))
    }
}

#[async_trait]
impl InfraJobRepository for DieselInfraJobRepository {
    async fn create(&self, params: CreateInfraJobParams) -> Result<InfraJobRecord, AppError> {
        let mut conn = self.api_conn().await?;
        let now = Utc::now();
        let job_id = params.id;
        let workspace_id = params.workspace_id;
        let task_type = params.task_type;
        let priority = params.priority;
        let parent_job_id = params.parent_job_id;
        let payload_hash = params.payload_hash;
        let request_id = params.request_id;
        let trace_id = params.trace_id;

        conn.transaction::<_, AppError, _>(|conn| {
            Box::pin(async move {
                let new_job = NewInfraJobRecord {
                    id: job_id,
                    workspace_id,
                    parent_job_id,
                    task_type,
                    priority,
                    state: "dispatching".to_string(),
                    attempt_count: 0,
                    lease_owner: None,
                    lease_token: None,
                    lease_expires_at: None,
                    next_attempt_at: None,
                    request_id,
                    trace_id: trace_id.clone(),
                    payload_hash,
                    error_code: None,
                    error_summary: None,
                    created_at: now,
                    queued_at: None,
                    started_at: None,
                    finished_at: None,
                    updated_at: now,
                };

                let job = diesel::insert_into(crate::schema::infra_job_records::table)
                    .values(&new_job)
                    .get_result::<InfraJobRecord>(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("Failed to create infra job: {e}")))?;

                let event = NewInfraJobEvent {
                    id: Uuid::new_v4(),
                    job_id,
                    workspace_id,
                    event_type: "created".to_string(),
                    from_state: None,
                    to_state: "dispatching".to_string(),
                    attempt_count: 0,
                    lease_token: None,
                    worker_id: None,
                    request_id,
                    trace_id,
                    reason_code: None,
                    reason_summary: None,
                    created_at: now,
                };

                diesel::insert_into(crate::schema::infra_job_events::table)
                    .values(&event)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| {
                        AppError::Internal(format!(
                            "Failed to append infra job creation event: {e}"
                        ))
                    })?;

                Ok(job)
            })
        })
        .await
    }

    #[allow(clippy::too_many_lines)]
    async fn transition(
        &self,
        params: TransitionInfraJobParams,
    ) -> Result<InfraJobRecord, AppError> {
        let mut conn = self.api_conn().await?;
        let now = Utc::now();
        let job_id = params.job_id;
        let workspace_id = params.workspace_id;
        let to_state = params.to_state.clone();
        let event_type = params.event_type.clone();

        conn.transaction::<_, AppError, _>(|conn| {
            Box::pin(async move {
                let current = crate::schema::infra_job_records::table
                    .filter(crate::schema::infra_job_records::id.eq(job_id))
                    .filter(crate::schema::infra_job_records::workspace_id.eq(workspace_id))
                    .for_update()
                    .first::<InfraJobRecord>(&mut *conn)
                    .await
                    .map_err(|e| match e {
                        diesel::result::Error::NotFound => {
                            AppError::Internal(format!("Infra job {job_id} was not found"))
                        }
                        other => AppError::Internal(format!(
                            "Failed to load infra job {job_id}: {other}"
                        )),
                    })?;

                if let Some(expected_state) = params.expected_state.as_deref() {
                    if current.state != expected_state {
                        return Err(AppError::Conflict(
                            crate::errors::LocalizedMessage::Key(
                                crate::utils::i18n::I18nKey::Conflict,
                            ),
                            Some(serde_json::json!({
                                "job_id": job_id,
                                "expected_state": expected_state,
                                "actual_state": current.state,
                            })),
                        ));
                    }
                }

                if current.state == "processing" && params.lease_token != current.lease_token {
                    return Err(AppError::Conflict(
                        crate::errors::LocalizedMessage::Key(crate::utils::i18n::I18nKey::Conflict),
                        Some(serde_json::json!({ "job_id": job_id, "reason": "stale_lease" })),
                    ));
                }

                if !is_allowed_transition(&current.state, &to_state) {
                    return Err(AppError::Validation(crate::errors::LocalizedMessage::Key(
                        crate::utils::i18n::I18nKey::Validation,
                    )));
                }

                let attempt_count = current.attempt_count + i32::from(to_state == "processing");
                let queued_at = if to_state == "queued" {
                    Some(now)
                } else {
                    current.queued_at
                };
                let started_at = if to_state == "processing" {
                    Some(now)
                } else {
                    current.started_at
                };
                let finished_at = if matches!(
                    to_state.as_str(),
                    "completed" | "failed" | "dead_letter" | "lost"
                ) {
                    Some(now)
                } else {
                    current.finished_at
                };
                let reason_summary = params
                    .reason_summary
                    .map(|summary| summary.chars().take(2_000).collect::<String>());

                diesel::update(
                    crate::schema::infra_job_records::table
                        .filter(crate::schema::infra_job_records::id.eq(job_id))
                        .filter(crate::schema::infra_job_records::workspace_id.eq(workspace_id)),
                )
                .set((
                    crate::schema::infra_job_records::state.eq(to_state.clone()),
                    crate::schema::infra_job_records::attempt_count.eq(attempt_count),
                    crate::schema::infra_job_records::lease_owner.eq(params.lease_owner.clone()),
                    crate::schema::infra_job_records::lease_token.eq(params.lease_token),
                    crate::schema::infra_job_records::lease_expires_at.eq(params.lease_expires_at),
                    crate::schema::infra_job_records::next_attempt_at.eq(params.next_attempt_at),
                    crate::schema::infra_job_records::error_code.eq(params.reason_code.clone()),
                    crate::schema::infra_job_records::error_summary.eq(reason_summary.clone()),
                    crate::schema::infra_job_records::queued_at.eq(queued_at),
                    crate::schema::infra_job_records::started_at.eq(started_at),
                    crate::schema::infra_job_records::finished_at.eq(finished_at),
                    crate::schema::infra_job_records::updated_at.eq(now),
                ))
                .execute(&mut *conn)
                .await
                .map_err(|e| {
                    AppError::Internal(format!("Failed to transition infra job {job_id}: {e}"))
                })?;

                diesel::insert_into(crate::schema::infra_job_events::table)
                    .values(NewInfraJobEvent {
                        id: Uuid::new_v4(),
                        job_id,
                        workspace_id,
                        event_type,
                        from_state: Some(current.state),
                        to_state,
                        attempt_count,
                        lease_token: params.lease_token,
                        worker_id: params.worker_id,
                        request_id: params.request_id,
                        trace_id: params.trace_id,
                        reason_code: params.reason_code,
                        reason_summary,
                        created_at: now,
                    })
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| {
                        AppError::Internal(format!("Failed to append infra job event: {e}"))
                    })?;

                crate::schema::infra_job_records::table
                    .filter(crate::schema::infra_job_records::id.eq(job_id))
                    .filter(crate::schema::infra_job_records::workspace_id.eq(workspace_id))
                    .first::<InfraJobRecord>(&mut *conn)
                    .await
                    .map_err(|e| {
                        AppError::Internal(format!("Failed to reload infra job {job_id}: {e}"))
                    })
            })
        })
        .await
    }

    async fn get(
        &self,
        workspace_id: Uuid,
        job_id: Uuid,
    ) -> Result<Option<InfraJobRecord>, AppError> {
        let mut conn = self.api_conn().await?;
        crate::schema::infra_job_records::table
            .filter(crate::schema::infra_job_records::id.eq(job_id))
            .filter(crate::schema::infra_job_records::workspace_id.eq(workspace_id))
            .first::<InfraJobRecord>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("Failed to read infra job {job_id}: {e}")))
    }

    async fn list_global(&self, params: ListInfraJobsParams) -> Result<InfraJobPage, AppError> {
        let mut conn = self.api_conn().await?;
        let bounded_limit = params.limit.clamp(1, 100);
        let bounded_limit_usize = usize::try_from(bounded_limit).map_err(|_conversion_error| {
            AppError::Internal("Invalid durable monitor query limit".to_string())
        })?;
        let records = crate::schema::infra_job_records::table.into_boxed();
        let mut query = records;

        if let Some(cursor) = params.cursor {
            query = query.filter(
                crate::schema::infra_job_records::created_at
                    .lt(cursor.created_at)
                    .or(crate::schema::infra_job_records::created_at
                        .eq(cursor.created_at)
                        .and(crate::schema::infra_job_records::id.lt(cursor.id))),
            );
        }
        if let Some(state) = params.state {
            query = query.filter(crate::schema::infra_job_records::state.eq(state));
        }
        if let Some(task_type) = params.task_type {
            query = query.filter(crate::schema::infra_job_records::task_type.eq(task_type));
        }
        if let Some(job_id) = params.job_id {
            query = query.filter(crate::schema::infra_job_records::id.eq(job_id));
        }
        if let Some(search) = params.search {
            query = query.filter(
                crate::schema::infra_job_records::task_type
                    .eq(search.clone())
                    .or(crate::schema::infra_job_records::error_code.eq(search)),
            );
        }

        let mut jobs = query
            .order((
                crate::schema::infra_job_records::created_at.desc(),
                crate::schema::infra_job_records::id.desc(),
            ))
            .limit(bounded_limit + 1)
            .load::<InfraJobRecord>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to list global infra jobs: {e}")))?;

        let next_cursor = if jobs.len() > bounded_limit_usize {
            let cursor_job = jobs.get(bounded_limit_usize - 1).cloned().ok_or_else(|| {
                AppError::Internal("Missing global infra job cursor row".to_string())
            })?;
            jobs.truncate(bounded_limit_usize);
            Some(InfraJobCursor {
                created_at: cursor_job.created_at,
                id: cursor_job.id,
            })
        } else {
            None
        };

        Ok(InfraJobPage { jobs, next_cursor })
    }

    async fn list_events(
        &self,
        workspace_id: Uuid,
        job_id: Uuid,
        limit: i64,
    ) -> Result<Vec<InfraJobEvent>, AppError> {
        let mut conn = self.api_conn().await?;
        let bounded_limit = limit.clamp(1, 200);
        crate::schema::infra_job_events::table
            .filter(crate::schema::infra_job_events::job_id.eq(job_id))
            .filter(crate::schema::infra_job_events::workspace_id.eq(workspace_id))
            .order((
                crate::schema::infra_job_events::created_at.asc(),
                crate::schema::infra_job_events::id.asc(),
            ))
            .limit(bounded_limit)
            .load::<InfraJobEvent>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to list infra job events: {e}")))
    }

    async fn list_events_bulk(
        &self,
        workspace_id: Option<Uuid>,
        job_ids: &[Uuid],
        total_limit: i64,
    ) -> Result<Vec<InfraJobEvent>, AppError> {
        if job_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut conn = self.api_conn().await?;
        let bounded_limit = total_limit.clamp(1, 1_000);
        let mut query = crate::schema::infra_job_events::table
            .filter(crate::schema::infra_job_events::job_id.eq_any(job_ids))
            .into_boxed();
        if let Some(workspace_id) = workspace_id {
            query = query.filter(crate::schema::infra_job_events::workspace_id.eq(workspace_id));
        }

        query
            .order((
                crate::schema::infra_job_events::job_id.asc(),
                crate::schema::infra_job_events::created_at.asc(),
                crate::schema::infra_job_events::id.asc(),
            ))
            .limit(bounded_limit)
            .load::<InfraJobEvent>(&mut *conn)
            .await
            .map_err(|e| {
                AppError::Internal(format!("Failed to list infra job events in bulk: {e}"))
            })
    }

    async fn load_monitor_data(
        &self,
        since: DateTime<Utc>,
        limit: i64,
    ) -> Result<InfraJobMonitorData, AppError> {
        let mut conn = self.api_conn().await?;
        let bounded_limit = limit.clamp(1, 20_000);
        let bounded_limit_usize = usize::try_from(bounded_limit).map_err(|error| {
            AppError::Internal(format!("Invalid monitor aggregation limit: {error}"))
        })?;

        let mut records = crate::schema::infra_job_records::table
            .order(crate::schema::infra_job_records::updated_at.desc())
            .limit(bounded_limit + 1)
            .load::<InfraJobRecord>(&mut *conn)
            .await
            .map_err(|error| {
                AppError::Internal(format!("Failed to load monitor job records: {error}"))
            })?;
        let records_truncated = records.len() > bounded_limit_usize;
        if records_truncated {
            records.truncate(bounded_limit_usize);
        }

        let mut events = crate::schema::infra_job_events::table
            .filter(crate::schema::infra_job_events::created_at.ge(since))
            .order((
                crate::schema::infra_job_events::created_at.asc(),
                crate::schema::infra_job_events::id.asc(),
            ))
            .limit(bounded_limit + 1)
            .load::<InfraJobEvent>(&mut *conn)
            .await
            .map_err(|error| {
                AppError::Internal(format!("Failed to load monitor job events: {error}"))
            })?;
        let events_truncated = events.len() > bounded_limit_usize;
        if events_truncated {
            events.truncate(bounded_limit_usize);
        }

        Ok(InfraJobMonitorData {
            records,
            events,
            truncated: records_truncated || events_truncated,
        })
    }
}

fn is_allowed_transition(from: &str, to: &str) -> bool {
    matches!(
        (from, to),
        ("dispatching" | "retryable", "queued")
            | (
                "dispatching" | "queued" | "processing" | "retryable",
                "lost"
            )
            | ("queued", "processing")
            | (
                "processing",
                "completed" | "failed" | "retryable" | "dead_letter"
            )
    )
}

#[cfg(test)]
mod tests {
    use super::is_allowed_transition;

    #[test]
    fn allows_only_declared_forward_lifecycle_transitions() {
        let allowed = [
            ("dispatching", "queued"),
            ("dispatching", "lost"),
            ("queued", "processing"),
            ("queued", "lost"),
            ("processing", "completed"),
            ("processing", "failed"),
            ("processing", "retryable"),
            ("processing", "dead_letter"),
            ("processing", "lost"),
            ("retryable", "queued"),
            ("retryable", "lost"),
        ];

        for (from, to) in allowed {
            assert!(is_allowed_transition(from, to), "{from} -> {to}");
        }
    }

    #[test]
    fn rejects_terminal_reprocessing_and_unknown_transitions() {
        let rejected = [
            ("dispatching", "processing"),
            ("queued", "completed"),
            ("processing", "queued"),
            ("completed", "queued"),
            ("failed", "retryable"),
            ("dead_letter", "queued"),
            ("lost", "completed"),
            ("unknown", "queued"),
        ];

        for (from, to) in rejected {
            assert!(!is_allowed_transition(from, to), "{from} -> {to}");
        }
    }
}

//! Leave Accrual Job Handler
//!
//! Processes leave accrual calculations as a background job.
//! Manages JobMetadata lifecycle: Pending → Processing → Completed/Failed.

use crate::queue::{Job, JobStatus};
use crate::services::leave_accrual_service::LeaveAccrualService;
use chrono::Utc;

pub struct AccrualHandler;

impl AccrualHandler {
    /// Handle a leave accrual job.
    /// Updates JobMetadata status throughout the lifecycle.
    pub async fn handle(
        queue: &std::sync::Arc<dyn crate::repositories::QueueRepository>,
        job: &Job,
        deps: &crate::worker::deps::WorkerDeps,
    ) -> Result<(), String> {
        let payload = job.accrual_payload().ok_or("Missing accrual payload")?;

        // Update metadata → Processing
        if let Ok(Some(mut meta)) = queue.get_job_metadata(job.parent_job_id).await {
            meta.status = JobStatus::Processing;
            let _ = queue.update_job_metadata(&meta).await;
        }

        tracing::info!(
            workspace_id = %job.workspace_id,
            date = %payload.date,
            force = payload.force,
            "Starting leave accrual processing"
        );

        let accrual_service = LeaveAccrualService::new(
            std::sync::Arc::clone(&deps.employee_repo),
            std::sync::Arc::clone(&deps.department_repo),
            std::sync::Arc::clone(&deps.leave_policy_repo),
            std::sync::Arc::clone(&deps.leave_accrual_log_repo),
            std::sync::Arc::clone(&deps.time_entry_repo),
            std::sync::Arc::clone(&deps.leave_request_repo),
            std::sync::Arc::clone(&deps.daily_summary_repo),
            std::sync::Arc::clone(&deps.contract_repo),
        );
        match accrual_service
            .process_workspace_accruals(job.workspace_id, payload.date, payload.force)
            .await
        {
            Ok(logs) => {
                tracing::info!(
                    workspace_id = %job.workspace_id,
                    accrual_count = logs.len(),
                    "Leave accrual processing completed"
                );

                // Update metadata → Completed
                if let Ok(Some(mut meta)) = queue.get_job_metadata(job.parent_job_id).await {
                    meta.status = JobStatus::Completed;
                    meta.completed_batches = 1;
                    meta.completed_at = Some(Utc::now());
                    let _ = queue.update_job_metadata(&meta).await;
                }

                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Accrual processing failed: {e}");

                // Update metadata → Failed
                if let Ok(Some(mut meta)) = queue.get_job_metadata(job.parent_job_id).await {
                    meta.status = JobStatus::Failed;
                    meta.failed_batches = 1;
                    meta.completed_at = Some(Utc::now());
                    meta.error_message = Some(error_msg.clone());
                    let _ = queue.update_job_metadata(&meta).await;
                }

                Err(error_msg)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queue::{AccrualPayload, JobPayload, JobPriority, TaskType};
    use crate::worker::test_utils::MockWorkerDepsBuilder;
    use chrono::NaiveDate;
    use std::sync::Arc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_accrual_handler_success() {
        let mut queue = crate::repositories::queue_repository::MockQueueRepository::new();
        let ws_id = Uuid::new_v4();
        let job_id = Uuid::new_v4();
        let parent_job_id = Uuid::new_v4();

        // 1. Queue expects status to move to Processing
        queue
            .expect_get_job_metadata()
            .with(mockall::predicate::eq(parent_job_id))
            .times(1)
            .returning(move |id| {
                Ok(Some(crate::queue::JobMetadata {
                    job_id: id,
                    status: JobStatus::Pending,
                    total_batches: 1,
                    completed_batches: 0,
                    failed_batches: 0,
                    workspace_id: ws_id,
                    requester_id: Uuid::new_v4(),
                    report_type: None,
                    date_range: None,
                    created_at: Utc::now(),
                    completed_at: None,
                    error_message: None,
                    task_type: None,
                    payload: None,
                }))
            });

        queue
            .expect_update_job_metadata()
            .withf(|meta| meta.status == JobStatus::Processing)
            .times(1)
            .returning(|_| Ok(()));

        // 2. Queue expects status to move to Completed
        queue
            .expect_get_job_metadata()
            .with(mockall::predicate::eq(parent_job_id))
            .times(1)
            .returning(move |id| {
                Ok(Some(crate::queue::JobMetadata {
                    job_id: id,
                    status: JobStatus::Processing,
                    total_batches: 1,
                    completed_batches: 0,
                    failed_batches: 0,
                    workspace_id: ws_id,
                    requester_id: Uuid::new_v4(),
                    report_type: None,
                    date_range: None,
                    created_at: Utc::now(),
                    completed_at: None,
                    error_message: None,
                    task_type: None,
                    payload: None,
                }))
            });

        queue
            .expect_update_job_metadata()
            .withf(|meta| meta.status == JobStatus::Completed && meta.completed_batches == 1)
            .times(1)
            .returning(|_| Ok(()));

        let queue_arc: Arc<dyn crate::repositories::QueueRepository> = Arc::new(queue);

        // 3. Mock Repositories to return empty so accrual loop finishes immediately
        let mut accrual_log_repo =
            crate::repositories::leave_accrual_log_repository::MockLeaveAccrualLogRepository::new();
        accrual_log_repo
            .expect_get_workspace_timezone()
            .returning(|_| Ok("UTC".to_string()));

        let mut employee_repo = crate::repositories::employee::MockEmployeeRepository::new();
        employee_repo
            .expect_get_active_employees()
            .returning(|_| Ok(vec![]));

        let mut builder = MockWorkerDepsBuilder::new();
        builder.leave_accrual_log_repo = accrual_log_repo;
        builder.employee_repo = employee_repo;
        let deps = builder.build();

        let job = Job {
            job_id,
            parent_job_id,
            batch_index: 0,
            task_type: TaskType::LeaveAccrual,
            workspace_id: ws_id,
            payload: JobPayload::LeaveAccrual(AccrualPayload {
                date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                force: false,
            }),
            priority: JobPriority::Normal,
            created_at: Utc::now(),
            retry_count: 0,
            max_retries: 3,
        };

        let result = AccrualHandler::handle(&queue_arc, &job, &deps).await;
        assert!(result.is_ok(), "Handler failed: {:?}", result.err());
    }
}

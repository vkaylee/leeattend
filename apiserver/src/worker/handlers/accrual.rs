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

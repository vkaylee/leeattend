//! Bulk Termination Job Handler

use crate::queue::{Job, JobStatus};
use crate::repositories::QueueRepository;
use crate::worker::WorkerDeps;
use std::sync::Arc;

pub struct BulkTerminationHandler;

impl BulkTerminationHandler {
    pub async fn handle(
        queue: &Arc<dyn QueueRepository>,
        job: &Job,
        deps: &WorkerDeps,
    ) -> Result<(), String> {
        let payload = job
            .bulk_termination_payload()
            .ok_or_else(|| "Missing bulk_termination payload".to_string())?;

        tracing::info!(
            job_id = %job.job_id,
            workspace_id = %job.workspace_id,
            rows_count = payload.rows.len(),
            "Starting bulk termination job"
        );

        let parent_job_id = job.parent_job_id;

        // 1. Update metadata → Processing
        if let Ok(Some(mut meta)) = queue.get_job_metadata(parent_job_id).await {
            if meta.status == JobStatus::Pending {
                meta.status = JobStatus::Processing;
                let _ = queue.update_job_metadata(&meta).await;
            }
        }

        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(job.workspace_id);

        let employee_service = crate::services::employee_service::EmployeeService::new(
            Arc::clone(&deps.employee_repo),
            Arc::clone(&deps.employment_repo),
            Arc::clone(&deps.department_repo),
            Arc::clone(&deps.user_repo),
            Arc::clone(&deps.staging_repo),
            Arc::clone(&deps.leave_balance_repo),
        );

        let mut completed_count = 0;
        let mut failed_count = 0;
        let mut first_error = None;

        for row in &payload.rows {
            match employee_service
                .terminate_employment(&auth_token, row.employment_id, row.termination_date)
                .await
            {
                Ok(_) => {
                    completed_count += 1;
                    let _ = queue.increment_completed(parent_job_id).await;
                }
                Err(e) => {
                    failed_count += 1;
                    let _ = queue.increment_failed(parent_job_id).await;
                    let err_msg = format!(
                        "Failed to terminate employment {}: {:?}",
                        row.employment_id, e
                    );
                    tracing::error!(
                        job_id = %job.job_id,
                        employment_id = %row.employment_id,
                        error = ?e,
                        "Employment termination failed inside bulk worker"
                    );
                    if first_error.is_none() {
                        first_error = Some(err_msg);
                    }
                }
            }
        }

        // 2. Update job metadata final state
        if let Ok(Some(mut meta)) = queue.get_job_metadata(parent_job_id).await {
            meta.completed_batches = completed_count;
            meta.failed_batches = failed_count;
            meta.completed_at = Some(chrono::Utc::now());

            if failed_count > 0 {
                meta.status = JobStatus::Failed;
                meta.error_message = first_error.clone();
            } else {
                meta.status = JobStatus::Completed;
            }

            let _ = queue.update_job_metadata(&meta).await;
        }

        if let Some(err) = first_error {
            return Err(err);
        }

        Ok(())
    }
}
